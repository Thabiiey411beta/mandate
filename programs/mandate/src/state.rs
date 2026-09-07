use anchor_lang::prelude::*;

pub const MAX_BALLOT: usize = 12;

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
    pub share_mint: Pubkey,
    pub idle_treasury: Pubkey,
    pub nav_usd_per_usdy_e6: u64,
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
    pub const SIZE: usize = 32 * 5 + 8 + 1 + 2 * 3 + 8 + 8 * 3 + 8 + 8 + 1 + 1 + 64;
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
    pub weights: Vec<u64>,
    pub voted_power: u64,
    pub total_power: u64,
    pub winner_idx: Option<u8>,
    pub executed: bool,
    pub bump: u8,
}

impl Epoch {
    pub const SIZE: usize = 32 + 8 + 8 * 4 + 4 + 32 * MAX_BALLOT + 4 + 8 * MAX_BALLOT + 8 + 8 + 2 + 1 + 1 + 32;
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
