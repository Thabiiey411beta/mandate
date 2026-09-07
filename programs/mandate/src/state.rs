use anchor_lang::prelude::*;

pub const MAX_BALLOT: usize = 12;
/// $5 ONDO (e6 USD) per $100 NAV → 100% rebate. k = 0.05 → k_e6 = 50_000.
pub const REBATE_K_E6: u64 = 50_000;
pub const UNSTAKE_COOLDOWN_SECS: i64 = 7 * 24 * 60 * 60;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum IdleMode {
    HoldUsdy = 0,
    SupplyIsolated = 1,
}

#[account]
pub struct Vault {
    pub authority: Pubkey,
    pub executor: Pubkey,
    pub usdy_mint: Pubkey,
    pub ondo_mint: Pubkey,
    pub share_mint: Pubkey,
    pub idle_treasury: Pubkey,
    pub ondo_treasury: Pubkey,
    pub nav_usd_per_usdy_e6: u64,
    pub ondo_usd_e6: u64,
    pub idle_mode: IdleMode,
    pub management_fee_bps: u16,
    pub max_name_weight_bps: u16,
    pub quorum_bps: u16,
    pub min_quorum_notional_e6: u64,
    pub epoch_length_secs: i64,
    pub vote_length_secs: i64,
    pub commit_length_secs: i64,
    pub current_epoch: u64,
    pub total_shares: u64,
    pub paused: bool,
    pub bump: u8,
}

impl Vault {
    pub const SIZE: usize = 32 * 7 + 8 * 2 + 1 + 2 * 3 + 8 + 8 * 3 + 8 + 8 + 1 + 1 + 64;
}

#[account]
pub struct Epoch {
    pub vault: Pubkey,
    pub index: u64,
    pub phase_start: i64,
    pub commit_end: i64,
    pub vote_end: i64,
    pub epoch_end: i64,
    pub universe: Vec<Pubkey>,
    pub labels: Vec<String>,
    pub weights: Vec<u64>,
    pub voted_power: u64,
    pub total_power: u64,
    pub winner_idx: Option<u8>,
    pub executed: bool,
    pub bump: u8,
}

impl Epoch {
    pub const SIZE: usize = 32 + 8 + 8 * 4 + 4 + 32 * MAX_BALLOT + 4 + 16 * MAX_BALLOT + 4
        + 8 * MAX_BALLOT + 8 + 8 + 2 + 1 + 1 + 64;
}

#[account]
pub struct Position {
    pub owner: Pubkey,
    pub vault: Pubkey,
    pub shares: u64,
    pub vote_power: u64,
    pub lock_mult_bps: u16,
    pub epoch_index: u64,
    pub voted: bool,
}

impl Position {
    pub const SIZE: usize = 32 + 32 + 8 + 8 + 2 + 8 + 1 + 16;
}

#[account]
pub struct RebateAccount {
    pub owner: Pubkey,
    pub vault: Pubkey,
    pub staked_ondo: u64,
    pub pending_unstake: u64,
    pub unstake_available_ts: i64,
    pub accrued_rebate_usdy: u64,
    pub last_fee_epoch: u64,
    pub bump: u8,
}

impl RebateAccount {
    pub const SIZE: usize = 32 + 32 + 8 + 8 + 8 + 8 + 8 + 1 + 16;

    /// rebate_bps = min(10000, staked_ondo_usd / (k * user_nav) * 10000)
    pub fn rebate_bps(staked_ondo: u64, ondo_usd_e6: u64, user_nav_e6: u64) -> u16 {
        if staked_ondo == 0 || user_nav_e6 == 0 || ondo_usd_e6 == 0 {
            return 0;
        }
        let staked_usd = (staked_ondo as u128).saturating_mul(ondo_usd_e6 as u128);
        let denom = (user_nav_e6 as u128).saturating_mul(REBATE_K_E6 as u128);
        if denom == 0 {
            return 0;
        }
        let bps = staked_usd.saturating_mul(10_000) / denom;
        u16::try_from(bps.min(10_000)).unwrap_or(10_000)
    }
}
