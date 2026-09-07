use anchor_lang::prelude::*;

pub const SPY_LTV_BPS: u16 = 6_000;
pub const SPY_LIQ_BPS: u16 = 7_000;
pub const LIQ_PENALTY_BPS: u16 = 400;
pub const USDY_NAV_APY_BPS: u16 = 355;
pub const BORROW_FLOOR_SPREAD_BPS: u16 = 75;
pub const KINK_UTIL_BPS: u16 = 5_000;
pub const KINK_SLOPE_BPS: u16 = 300;
pub const JUMP_SLOPE_BPS: u16 = 1_200;

pub const OPCO_BPS: u16 = 5_000;
pub const INSURANCE_BPS: u16 = 2_500;
pub const PROTOCOL_BPS: u16 = 2_500;

pub const WAD: u128 = 1_000_000_000_000;

#[account]
pub struct Market {
    pub authority: Pubkey,
    pub collateral_mint: Pubkey,
    pub debt_mint: Pubkey,
    pub collateral_vault: Pubkey,
    pub debt_vault: Pubkey,
    pub opco_treasury: Pubkey,
    pub insurance_treasury: Pubkey,
    pub protocol_treasury: Pubkey,
    pub oracle: Pubkey,
    pub ltv_bps: u16,
    pub liq_threshold_bps: u16,
    pub liq_penalty_bps: u16,
    pub collateral_price_e6: u64,
    pub usdy_price_e6: u64,
    pub price_ts: i64,
    pub max_price_age_secs: i64,
    pub borrow_index: u128,
    pub supply_index: u128,
    pub total_debt_shares: u64,
    pub total_supply_shares: u64,
    pub total_debt_assets: u64,
    pub total_supply_assets: u64,
    pub pending_opco: u64,
    pub pending_insurance: u64,
    pub pending_protocol: u64,
    pub last_accrual_ts: i64,
    pub paused: bool,
    pub ticker: [u8; 12],
    pub bump: u8,
}

impl Market {
    pub const SIZE: usize = 32 * 9 + 2 * 3 + 8 * 6 + 16 * 2 + 8 * 7 + 1 + 12 + 1 + 64;

    pub fn utilization_bps(&self) -> u16 {
        if self.total_supply_assets == 0 {
            return 0;
        }
        let u = (self.total_debt_assets as u128 * 10_000) / self.total_supply_assets as u128;
        u.min(10_000) as u16
    }

    pub fn borrow_apr_bps(&self) -> u16 {
        let floor = USDY_NAV_APY_BPS.saturating_add(BORROW_FLOOR_SPREAD_BPS);
        let util = self.utilization_bps();
        let curve = if util <= KINK_UTIL_BPS {
            (util as u32 * KINK_SLOPE_BPS as u32) / KINK_UTIL_BPS as u32
        } else {
            let extra = util.saturating_sub(KINK_UTIL_BPS) as u32;
            KINK_SLOPE_BPS as u32 + (extra * JUMP_SLOPE_BPS as u32) / (10_000 - KINK_UTIL_BPS) as u32
        };
        floor.saturating_add(curve as u16)
    }
}

#[account]
pub struct Lender {
    pub owner: Pubkey,
    pub market: Pubkey,
    pub supply_shares: u64,
    pub bump: u8,
}

impl Lender {
    pub const SIZE: usize = 32 + 32 + 8 + 1 + 16;
}

#[account]
pub struct Borrower {
    pub owner: Pubkey,
    pub market: Pubkey,
    pub collateral_atoms: u64,
    pub debt_shares: u64,
    pub last_borrow_slot: u64,
    pub bump: u8,
}

impl Borrower {
    pub const SIZE: usize = 32 + 32 + 8 + 8 + 8 + 1 + 16;
}
