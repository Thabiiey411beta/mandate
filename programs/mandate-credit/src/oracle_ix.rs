use anchor_lang::prelude::*;
use crate::oracle::{try_read_pyth_e6, OndoOracle};
use crate::state::Market;
use crate::CreditError;

#[derive(Accounts)]
pub struct InitOracle<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(has_one = authority)]
    pub market: Account<'info, Market>,
    #[account(
        init,
        payer = authority,
        space = 8 + OndoOracle::SIZE,
        seeds = [b"ondo_oracle", market.key().as_ref()],
        bump
    )]
    pub oracle: Account<'info, OndoOracle>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RefreshPrices<'info> {
    pub authority: Signer<'info>,
    #[account(mut, has_one = authority)]
    pub market: Account<'info, Market>,
    #[account(mut, seeds = [b"ondo_oracle", market.key().as_ref()], bump = oracle.bump)]
    pub oracle: Account<'info, OndoOracle>,
    /// CHECK: Pyth USDY price-update account (optional; pass oracle if unused)
    pub pyth_usdy: UncheckedAccount<'info>,
}

pub fn init_oracle(ctx: Context<InitOracle>, pyth_usdy_feed: Pubkey, max_age_secs: i64) -> Result<()> {
    let o = &mut ctx.accounts.oracle;
    o.market = ctx.accounts.market.key();
    o.authority = ctx.accounts.authority.key();
    o.pyth_usdy_feed = pyth_usdy_feed;
    o.collat_usd_e6 = 0;
    o.usdy_usd_e6 = 0;
    o.collat_ts = 0;
    o.usdy_ts = 0;
    o.max_age_secs = max_age_secs.max(30);
    o.bump = ctx.bumps.oracle;
    ctx.accounts.market.oracle = o.key();
    ctx.accounts.market.max_price_age_secs = o.max_age_secs;
    Ok(())
}

/// `collat_e6` is the Ondo GM primaryMarket price (keeper). USDY prefers Pyth.
pub fn refresh_prices(ctx: Context<RefreshPrices>, collat_e6: u64, usdy_fallback_e6: u64) -> Result<()> {
    require!(collat_e6 > 0, CreditError::Zero);
    let now = Clock::get()?.unix_timestamp;
    let o = &mut ctx.accounts.oracle;
    o.collat_usd_e6 = collat_e6;
    o.collat_ts = now;

    let mut usdy = usdy_fallback_e6;
    if ctx.accounts.pyth_usdy.key() == o.pyth_usdy_feed {
        if let Some((px, ts)) = try_read_pyth_e6(&ctx.accounts.pyth_usdy.data.borrow()) {
            usdy = px;
            o.usdy_ts = ts;
        }
    }
    require!(usdy > 0, CreditError::StaleOracle);
    o.usdy_usd_e6 = usdy;
    if o.usdy_ts == 0 {
        o.usdy_ts = now;
    }

    let m = &mut ctx.accounts.market;
    m.collateral_price_e6 = o.collat_usd_e6;
    m.usdy_price_e6 = o.usdy_usd_e6;
    m.price_ts = now.min(o.collat_ts).min(o.usdy_ts);
    require!(o.collat_fresh(now) && o.usdy_fresh(now), CreditError::StaleOracle);
    Ok(())
}
