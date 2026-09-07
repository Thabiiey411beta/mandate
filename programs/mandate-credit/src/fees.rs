use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::*;

pub fn apply_interest(m: &mut Market, interest: u64) {
    let take = interest / 4;
    let opco = take * OPCO_BPS as u64 / 10_000;
    let ins = take * INSURANCE_BPS as u64 / 10_000;
    let proto = take.saturating_sub(opco).saturating_sub(ins);
    m.pending_opco = m.pending_opco.saturating_add(opco);
    m.pending_insurance = m.pending_insurance.saturating_add(ins);
    m.pending_protocol = m.pending_protocol.saturating_add(proto);
    m.total_debt_assets = m.total_debt_assets.saturating_add(interest);
    m.total_supply_assets = m.total_supply_assets.saturating_add(interest.saturating_sub(take));
}

#[derive(Accounts)]
pub struct SweepFees<'info> {
    pub authority: Signer<'info>,
    #[account(mut, has_one = authority)]
    pub market: Account<'info, Market>,
    #[account(mut, address = market.debt_vault)]
    pub debt_vault: Account<'info, TokenAccount>,
    #[account(mut, address = market.opco_treasury)]
    pub opco_treasury: Account<'info, TokenAccount>,
    #[account(mut, address = market.insurance_treasury)]
    pub insurance_treasury: Account<'info, TokenAccount>,
    #[account(mut, address = market.protocol_treasury)]
    pub protocol_treasury: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

pub fn sweep(ctx: Context<SweepFees>) -> Result<()> {
    let m = &mut ctx.accounts.market;
    let parts = [
        (m.pending_opco, ctx.accounts.opco_treasury.to_account_info()),
        (m.pending_insurance, ctx.accounts.insurance_treasury.to_account_info()),
        (m.pending_protocol, ctx.accounts.protocol_treasury.to_account_info()),
    ];
    let seeds = &[
        b"market",
        m.collateral_mint.as_ref(),
        m.debt_mint.as_ref(),
        &[m.bump],
    ];
    for (amt, dest) in parts {
        if amt == 0 {
            continue;
        }
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.debt_vault.to_account_info(),
                    to: dest,
                    authority: m.to_account_info(),
                },
                &[seeds],
            ),
            amt,
        )?;
    }
    m.pending_opco = 0;
    m.pending_insurance = 0;
    m.pending_protocol = 0;
    Ok(())
}
