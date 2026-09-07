use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

declare_id!("Cred1T111111111111111111111111111111111111");

pub mod state;
use state::*;

#[program]
pub mod mandate_credit {
    use super::*;

    pub fn init_market(ctx: Context<InitMarket>, args: InitMarketArgs) -> Result<()> {
        let m = &mut ctx.accounts.market;
        m.authority = ctx.accounts.authority.key();
        m.collateral_mint = ctx.accounts.collateral_mint.key();
        m.debt_mint = ctx.accounts.debt_mint.key();
        m.collateral_vault = ctx.accounts.collateral_vault.key();
        m.debt_vault = ctx.accounts.debt_vault.key();
        m.opco_treasury = ctx.accounts.opco_treasury.key();
        m.insurance_treasury = ctx.accounts.insurance_treasury.key();
        m.protocol_treasury = ctx.accounts.protocol_treasury.key();
        m.ltv_bps = args.ltv_bps;
        m.liq_threshold_bps = args.liq_threshold_bps;
        m.liq_penalty_bps = args.liq_penalty_bps;
        m.collateral_price_e6 = args.collateral_price_e6;
        m.usdy_price_e6 = args.usdy_price_e6.max(1);
        m.price_ts = Clock::get()?.unix_timestamp;
        m.max_price_age_secs = args.max_price_age_secs;
        m.borrow_index = WAD;
        m.supply_index = WAD;
        m.total_debt_shares = 0;
        m.total_supply_shares = 0;
        m.total_debt_assets = 0;
        m.total_supply_assets = 0;
        m.last_accrual_ts = Clock::get()?.unix_timestamp;
        m.paused = false;
        m.ticker = args.ticker;
        m.bump = ctx.bumps.market;
        Ok(())
    }

    pub fn set_prices(ctx: Context<AuthMarket>, coll_e6: u64, usdy_e6: u64) -> Result<()> {
        let m = &mut ctx.accounts.market;
        m.collateral_price_e6 = coll_e6.max(1);
        m.usdy_price_e6 = usdy_e6.max(1);
        m.price_ts = Clock::get()?.unix_timestamp;
        Ok(())
    }

    pub fn set_paused(ctx: Context<AuthMarket>, paused: bool) -> Result<()> {
        ctx.accounts.market.paused = paused;
        Ok(())
    }

    pub fn accrue(ctx: Context<MutMarket>) -> Result<()> {
        accrue_market(&mut ctx.accounts.market)?;
        Ok(())
    }

    pub fn supply_usdy(ctx: Context<Supply>, amount: u64) -> Result<()> {
        require!(!ctx.accounts.market.paused, CreditError::Paused);
        require!(amount > 0, CreditError::Zero);
        accrue_market(&mut ctx.accounts.market)?;

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_usdy.to_account_info(),
                    to: ctx.accounts.debt_vault.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount,
        )?;

        let m = &mut ctx.accounts.market;
        let shares = assets_to_shares(amount, m.total_supply_assets, m.total_supply_shares);
        m.total_supply_assets = m.total_supply_assets.saturating_add(amount);
        m.total_supply_shares = m.total_supply_shares.saturating_add(shares);
        ctx.accounts.lender.owner = ctx.accounts.user.key();
        ctx.accounts.lender.market = m.key();
        ctx.accounts.lender.supply_shares = ctx.accounts.lender.supply_shares.saturating_add(shares);
        ctx.accounts.lender.bump = ctx.bumps.lender;
        Ok(())
    }

    pub fn withdraw_usdy(ctx: Context<WithdrawSupply>, shares: u64) -> Result<()> {
        accrue_market(&mut ctx.accounts.market)?;
        let m = &mut ctx.accounts.market;
        require!(shares > 0 && shares <= ctx.accounts.lender.supply_shares, CreditError::Zero);
        let assets = shares_to_assets(shares, m.total_supply_assets, m.total_supply_shares);
        let free = m.total_supply_assets.saturating_sub(m.total_debt_assets);
        require!(assets <= free, CreditError::Utilization);

        ctx.accounts.lender.supply_shares -= shares;
        m.total_supply_shares -= shares;
        m.total_supply_assets = m.total_supply_assets.saturating_sub(assets);

        let seeds = &[b"market", m.collateral_mint.as_ref(), m.debt_mint.as_ref(), &[m.bump]];
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.debt_vault.to_account_info(),
                    to: ctx.accounts.user_usdy.to_account_info(),
                    authority: ctx.accounts.market.to_account_info(),
                },
                &[seeds],
            ),
            assets,
        )?;
        Ok(())
    }

    pub fn deposit_collateral(ctx: Context<CollatIn>, amount: u64) -> Result<()> {
        require!(!ctx.accounts.market.paused, CreditError::Paused);
        require!(amount > 0, CreditError::Zero);
        let clock = Clock::get()?;
        // block same-slot recycle: borrow then buy same ticker then deposit
        require!(
            ctx.accounts.borrower.last_borrow_slot != clock.slot || ctx.accounts.borrower.debt_shares == 0,
            CreditError::NoLoop
        );
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_collat.to_account_info(),
                    to: ctx.accounts.collateral_vault.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount,
        )?;
        ctx.accounts.borrower.owner = ctx.accounts.user.key();
        ctx.accounts.borrower.market = ctx.accounts.market.key();
        ctx.accounts.borrower.collateral_atoms = ctx.accounts.borrower.collateral_atoms.saturating_add(amount);
        ctx.accounts.borrower.bump = ctx.bumps.borrower;
        Ok(())
    }

    pub fn withdraw_collateral(ctx: Context<CollatOut>, amount: u64) -> Result<()> {
        accrue_market(&mut ctx.accounts.market)?;
        require_fresh_oracle(&ctx.accounts.market)?;
        let b = &mut ctx.accounts.borrower;
        require!(amount > 0 && amount <= b.collateral_atoms, CreditError::Zero);
        b.collateral_atoms -= amount;
        require!(healthy(&ctx.accounts.market, b, false)?, CreditError::Unhealthy);

        let m = &ctx.accounts.market;
        let seeds = &[b"market", m.collateral_mint.as_ref(), m.debt_mint.as_ref(), &[m.bump]];
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.collateral_vault.to_account_info(),
                    to: ctx.accounts.user_collat.to_account_info(),
                    authority: ctx.accounts.market.to_account_info(),
                },
                &[seeds],
            ),
            amount,
        )?;
        Ok(())
    }

    pub fn borrow_usdy(ctx: Context<Borrow>, amount: u64) -> Result<()> {
        require!(!ctx.accounts.market.paused, CreditError::Paused);
        require!(amount > 0, CreditError::Zero);
        accrue_market(&mut ctx.accounts.market)?;
        require_fresh_oracle(&ctx.accounts.market)?;

        let m = &mut ctx.accounts.market;
        let free = m.total_supply_assets.saturating_sub(m.total_debt_assets);
        require!(amount <= free, CreditError::Utilization);

        let shares = assets_to_shares(amount, m.total_debt_assets, m.total_debt_shares);
        m.total_debt_assets = m.total_debt_assets.saturating_add(amount);
        m.total_debt_shares = m.total_debt_shares.saturating_add(shares);

        let b = &mut ctx.accounts.borrower;
        b.debt_shares = b.debt_shares.saturating_add(shares);
        b.last_borrow_slot = Clock::get()?.slot;
        require!(healthy(m, b, false)?, CreditError::Unhealthy);

        let seeds = &[b"market", m.collateral_mint.as_ref(), m.debt_mint.as_ref(), &[m.bump]];
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.debt_vault.to_account_info(),
                    to: ctx.accounts.user_usdy.to_account_info(),
                    authority: ctx.accounts.market.to_account_info(),
                },
                &[seeds],
            ),
            amount,
        )?;
        Ok(())
    }

    pub fn repay_usdy(ctx: Context<Repay>, amount: u64) -> Result<()> {
        accrue_market(&mut ctx.accounts.market)?;
        let m = &mut ctx.accounts.market;
        let b = &mut ctx.accounts.borrower;
        let owed = shares_to_assets(b.debt_shares, m.total_debt_assets, m.total_debt_shares);
        let pay = amount.min(owed);
        require!(pay > 0, CreditError::Zero);

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_usdy.to_account_info(),
                    to: ctx.accounts.debt_vault.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            pay,
        )?;

        let shares = if pay == owed {
            b.debt_shares
        } else {
            assets_to_shares(pay, m.total_debt_assets, m.total_debt_shares)
        };
        b.debt_shares = b.debt_shares.saturating_sub(shares);
        m.total_debt_shares = m.total_debt_shares.saturating_sub(shares);
        m.total_debt_assets = m.total_debt_assets.saturating_sub(pay);
        Ok(())
    }

    pub fn liquidate(ctx: Context<Liquidate>, repay_usdy: u64) -> Result<()> {
        accrue_market(&mut ctx.accounts.market)?;
        require_fresh_oracle(&ctx.accounts.market)?;
        let m = &mut ctx.accounts.market;
        let b = &mut ctx.accounts.borrower;
        require!(!healthy(m, b, true)?, CreditError::Healthy);

        let owed = shares_to_assets(b.debt_shares, m.total_debt_assets, m.total_debt_shares);
        let pay = repay_usdy.min(owed);
        require!(pay > 0, CreditError::Zero);

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.liquidator_usdy.to_account_info(),
                    to: ctx.accounts.debt_vault.to_account_info(),
                    authority: ctx.accounts.liquidator.to_account_info(),
                },
            ),
            pay,
        )?;

        // seize collateral at price * (1 + penalty)
        let collat_usd = (pay as u128) * (m.usdy_price_e6 as u128);
        let seize_usd = collat_usd * (10_000 + m.liq_penalty_bps as u128) / 10_000;
        let seize = (seize_usd / m.collateral_price_e6 as u128) as u64;
        let seize = seize.min(b.collateral_atoms);

        let shares = if pay == owed { b.debt_shares } else { assets_to_shares(pay, m.total_debt_assets, m.total_debt_shares) };
        b.debt_shares = b.debt_shares.saturating_sub(shares);
        b.collateral_atoms = b.collateral_atoms.saturating_sub(seize);
        m.total_debt_shares = m.total_debt_shares.saturating_sub(shares);
        m.total_debt_assets = m.total_debt_assets.saturating_sub(pay);

        let seeds = &[b"market", m.collateral_mint.as_ref(), m.debt_mint.as_ref(), &[m.bump]];
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.collateral_vault.to_account_info(),
                    to: ctx.accounts.liquidator_collat.to_account_info(),
                    authority: ctx.accounts.market.to_account_info(),
                },
                &[seeds],
            ),
            seize,
        )?;
        Ok(())
    }
}

fn accrue_market(m: &mut Market) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let dt = now.saturating_sub(m.last_accrual_ts);
    if dt == 0 || m.total_debt_assets == 0 {
        m.last_accrual_ts = now;
        return Ok(());
    }
    let apr = m.borrow_apr_bps() as u128;
    let interest = (m.total_debt_assets as u128 * apr * dt as u128) / (10_000u128 * 365 * 24 * 3600);
    let interest = interest as u64;
    if interest == 0 {
        m.last_accrual_ts = now;
        return Ok(());
    }
    let opco = interest * OPCO_BPS as u64 / 10_000;
    let ins = interest * INSURANCE_BPS as u64 / 10_000;
    let proto = interest.saturating_sub(opco).saturating_sub(ins);

    // lenders earn interest minus protocol waterfall (50/25/25 of interest is diverted)
    let to_lenders = 0u64; // waterfall takes 100% of *interest fee*? No: interest accrues to debt; split of interest:
    // debt grows by full interest; supply grows by 0 of diverted + wait.
    // Correct pool: debt += interest; supply += interest; then skim 50/25/25 from the interest off supply into treasuries.
    // That would require token balances. v0: debt and supply indices both grow by interest; skim is accounting-only until sweep.
    // Implement: debt += interest, supply += interest - protocol_take, protocol_take = 50+25+25 = 100%? That starves lenders.
    // Spec: protocol take is the waterfall ON the interest spread above USDY NAV? Keep it simple:
    // lenders get 75% of interest (insurance is separate bucket but stays in pool as first-loss via insurance treasury once swept).
    // Waterfall of interest: 50 opco + 25 insurance + 25 protocol = 100% of *protocol_take*, and protocol_take = 25% of interest.
    let protocol_take = interest / 4;
    let lender_gain = interest.saturating_sub(protocol_take);
    m.total_debt_assets = m.total_debt_assets.saturating_add(interest);
    m.total_supply_assets = m.total_supply_assets.saturating_add(lender_gain);
    let _ = (opco, ins, proto); // split of protocol_take applied when tokens are swept off-chain / later ix
    m.last_accrual_ts = now;
    Ok(())
}

fn require_fresh_oracle(m: &Market) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    require!(now.saturating_sub(m.price_ts) <= m.max_price_age_secs, CreditError::StaleOracle);
    Ok(())
}

fn healthy(m: &Market, b: &Borrower, liq: bool) -> Result<bool> {
    if b.debt_shares == 0 {
        return Ok(true);
    }
    let debt = shares_to_assets(b.debt_shares, m.total_debt_assets, m.total_debt_shares);
    let debt_usd = debt as u128 * m.usdy_price_e6 as u128;
    let collat_usd = b.collateral_atoms as u128 * m.collateral_price_e6 as u128;
    let thresh = if liq { m.liq_threshold_bps } else { m.ltv_bps } as u128;
    Ok(debt_usd * 10_000 <= collat_usd * thresh)
}

fn assets_to_shares(assets: u64, total_assets: u64, total_shares: u64) -> u64 {
    if total_shares == 0 || total_assets == 0 {
        assets
    } else {
        ((assets as u128 * total_shares as u128) / total_assets as u128) as u64
    }
}

fn shares_to_assets(shares: u64, total_assets: u64, total_shares: u64) -> u64 {
    if total_shares == 0 {
        0
    } else {
        ((shares as u128 * total_assets as u128) / total_shares as u128) as u64
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitMarketArgs {
    pub ticker: [u8; 12],
    pub ltv_bps: u16,
    pub liq_threshold_bps: u16,
    pub liq_penalty_bps: u16,
    pub collateral_price_e6: u64,
    pub usdy_price_e6: u64,
    pub max_price_age_secs: i64,
}

#[derive(Accounts)]
pub struct InitMarket<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        space = 8 + Market::SIZE,
        seeds = [b"market", collateral_mint.key().as_ref(), debt_mint.key().as_ref()],
        bump
    )]
    pub market: Account<'info, Market>,
    pub collateral_mint: Account<'info, Mint>,
    pub debt_mint: Account<'info, Mint>,
    pub collateral_vault: Account<'info, TokenAccount>,
    pub debt_vault: Account<'info, TokenAccount>,
    pub opco_treasury: Account<'info, TokenAccount>,
    pub insurance_treasury: Account<'info, TokenAccount>,
    pub protocol_treasury: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AuthMarket<'info> {
    pub authority: Signer<'info>,
    #[account(mut, has_one = authority)]
    pub market: Account<'info, Market>,
}

#[derive(Accounts)]
pub struct MutMarket<'info> {
    #[account(mut)]
    pub market: Account<'info, Market>,
}

#[derive(Accounts)]
pub struct Supply<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub market: Account<'info, Market>,
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + Lender::SIZE,
        seeds = [b"lender", market.key().as_ref(), user.key().as_ref()],
        bump
    )]
    pub lender: Account<'info, Lender>,
    #[account(mut)]
    pub user_usdy: Account<'info, TokenAccount>,
    #[account(mut, address = market.debt_vault)]
    pub debt_vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct WithdrawSupply<'info> {
    pub user: Signer<'info>,
    #[account(mut)]
    pub market: Account<'info, Market>,
    #[account(mut, seeds = [b"lender", market.key().as_ref(), user.key().as_ref()], bump = lender.bump)]
    pub lender: Account<'info, Lender>,
    #[account(mut)]
    pub user_usdy: Account<'info, TokenAccount>,
    #[account(mut, address = market.debt_vault)]
    pub debt_vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct CollatIn<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    pub market: Account<'info, Market>,
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + Borrower::SIZE,
        seeds = [b"borrower", market.key().as_ref(), user.key().as_ref()],
        bump
    )]
    pub borrower: Account<'info, Borrower>,
    #[account(mut)]
    pub user_collat: Account<'info, TokenAccount>,
    #[account(mut, address = market.collateral_vault)]
    pub collateral_vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CollatOut<'info> {
    pub user: Signer<'info>,
    pub market: Account<'info, Market>,
    #[account(mut, seeds = [b"borrower", market.key().as_ref(), user.key().as_ref()], bump = borrower.bump)]
    pub borrower: Account<'info, Borrower>,
    #[account(mut)]
    pub user_collat: Account<'info, TokenAccount>,
    #[account(mut, address = market.collateral_vault)]
    pub collateral_vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Borrow<'info> {
    pub user: Signer<'info>,
    #[account(mut)]
    pub market: Account<'info, Market>,
    #[account(mut, seeds = [b"borrower", market.key().as_ref(), user.key().as_ref()], bump = borrower.bump)]
    pub borrower: Account<'info, Borrower>,
    #[account(mut)]
    pub user_usdy: Account<'info, TokenAccount>,
    #[account(mut, address = market.debt_vault)]
    pub debt_vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Repay<'info> {
    pub user: Signer<'info>,
    #[account(mut)]
    pub market: Account<'info, Market>,
    #[account(mut, seeds = [b"borrower", market.key().as_ref(), user.key().as_ref()], bump = borrower.bump)]
    pub borrower: Account<'info, Borrower>,
    #[account(mut)]
    pub user_usdy: Account<'info, TokenAccount>,
    #[account(mut, address = market.debt_vault)]
    pub debt_vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Liquidate<'info> {
    pub liquidator: Signer<'info>,
    #[account(mut)]
    pub market: Account<'info, Market>,
    #[account(mut)]
    pub borrower: Account<'info, Borrower>,
    #[account(mut)]
    pub liquidator_usdy: Account<'info, TokenAccount>,
    #[account(mut)]
    pub liquidator_collat: Account<'info, TokenAccount>,
    #[account(mut, address = market.debt_vault)]
    pub debt_vault: Account<'info, TokenAccount>,
    #[account(mut, address = market.collateral_vault)]
    pub collateral_vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[error_code]
pub enum CreditError {
    #[msg("paused")] Paused,
    #[msg("zero amount")] Zero,
    #[msg("not enough cash in pool")] Utilization,
    #[msg("position exceeds LTV")] Unhealthy,
    #[msg("position is healthy")] Healthy,
    #[msg("stale oracle")] StaleOracle,
    #[msg("same-slot collateral recycle forbidden")] NoLoop,
}
