use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

declare_id!("MandatE11111111111111111111111111111111111");

pub mod state;
use state::*;

#[program]
pub mod mandate {
    use super::*;

    pub fn initialize_vault(
        ctx: Context<InitializeVault>,
        args: InitializeVaultArgs,
    ) -> Result<()> {
        let v = &mut ctx.accounts.vault;
        v.authority = ctx.accounts.authority.key();
        v.executor = args.executor;
        v.usdy_mint = ctx.accounts.usdy_mint.key();
        v.share_mint = ctx.accounts.share_mint.key();
        v.idle_treasury = ctx.accounts.idle_treasury.key();
        v.nav_usd_per_usdy_e6 = args.nav_usd_per_usdy_e6;
        v.idle_mode = IdleMode::HoldUsdy;
        v.management_fee_bps = args.management_fee_bps;
        v.max_name_weight_bps = args.max_name_weight_bps;
        v.quorum_bps = args.quorum_bps;
        v.min_quorum_notional_e6 = args.min_quorum_notional_e6;
        v.epoch_length_secs = args.epoch_length_secs;
        v.vote_length_secs = args.vote_length_secs;
        v.commit_length_secs = args.commit_length_secs;
        v.paused = false;
        v.bump = ctx.bumps.vault;
        v.current_epoch = 0;
        v.total_shares = 0;
        Ok(())
    }

    pub fn open_epoch(ctx: Context<OpenEpoch>, universe: Vec<Pubkey>) -> Result<()> {
        require!(!ctx.accounts.vault.paused, MandateError::Paused);
        require!(universe.len() <= MAX_BALLOT, MandateError::BallotTooLarge);
        require!(universe.len() > 0, MandateError::EmptyUniverse);

        let now = Clock::get()?.unix_timestamp;
        let v = &mut ctx.accounts.vault;
        v.current_epoch = v.current_epoch.checked_add(1).ok_or(MandateError::Overflow)?;

        let e = &mut ctx.accounts.epoch;
        e.vault = v.key();
        e.index = v.current_epoch;
        e.phase_start = now;
        e.commit_end = now.saturating_add(v.commit_length_secs);
        e.vote_end = now
            .saturating_add(v.commit_length_secs)
            .saturating_add(v.vote_length_secs);
        e.epoch_end = now.saturating_add(v.epoch_length_secs);
        e.universe = universe;
        e.weights = vec![0u64; e.universe.len()];
        e.voted_power = 0;
        e.total_power = 0;
        e.winner_idx = None;
        e.executed = false;
        e.bump = ctx.bumps.epoch;
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64, lock_mult_bps: u16) -> Result<()> {
        require!(!ctx.accounts.vault.paused, MandateError::Paused);
        require!(amount > 0, MandateError::ZeroAmount);
        require!(
            matches!(lock_mult_bps, 100 | 110 | 125 | 150),
            MandateError::BadLock
        );

        let now = Clock::get()?.unix_timestamp;
        let epoch = &mut ctx.accounts.epoch;
        require!(now < epoch.commit_end, MandateError::NotCommitPhase);

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_usdy.to_account_info(),
                    to: ctx.accounts.idle_treasury.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount,
        )?;

        let vault = &mut ctx.accounts.vault;
        let pos = &mut ctx.accounts.position;

        let nav = vault.nav_usd_per_usdy_e6.max(1);
        let value_e6 = amount.saturating_mul(nav);
        // share price = total_assets / total_shares; seed 1:1 on first deposit
        let shares = if vault.total_shares == 0 {
            amount
        } else {
            // conservative: shares ~ amount (NAV accrual handled in exchange rate later)
            amount
        };

        pos.owner = ctx.accounts.user.key();
        pos.vault = vault.key();
        pos.shares = pos.shares.saturating_add(shares);
        pos.vote_power = pos.vote_power.saturating_add(
            amount.saturating_mul(lock_mult_bps as u64) / 100,
        );
        pos.lock_mult_bps = lock_mult_bps;
        pos.epoch_index = epoch.index;
        pos.voted = false;

        vault.total_shares = vault.total_shares.saturating_add(shares);
        epoch.total_power = epoch.total_power.saturating_add(pos.vote_power);
        let _ = value_e6;
        Ok(())
    }

    pub fn cast_vote(ctx: Context<CastVote>, weights: Vec<u64>) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        let epoch = &mut ctx.accounts.epoch;
        require!(now >= epoch.commit_end && now < epoch.vote_end, MandateError::NotVotePhase);
        require!(weights.len() == epoch.universe.len(), MandateError::WeightMismatch);

        let pos = &mut ctx.accounts.position;
        require!(!pos.voted, MandateError::AlreadyVoted);
        require!(pos.epoch_index == epoch.index, MandateError::WrongEpoch);

        let sum: u64 = weights.iter().sum();
        require!(sum > 0, MandateError::ZeroAmount);

        // scale weights to this user's vote power
        for (i, w) in weights.iter().enumerate() {
            let part = (*w as u128)
                .saturating_mul(pos.vote_power as u128)
                / (sum as u128);
            epoch.weights[i] = epoch.weights[i].saturating_add(part as u64);
        }
        epoch.voted_power = epoch.voted_power.saturating_add(pos.vote_power);
        pos.voted = true;
        Ok(())
    }

    pub fn tally(ctx: Context<Tally>) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        let vault = &ctx.accounts.vault;
        let epoch = &mut ctx.accounts.epoch;
        require!(now >= epoch.vote_end, MandateError::VoteOpen);
        require!(!epoch.executed, MandateError::AlreadyExecuted);

        let quorum_ok = epoch.total_power > 0
            && epoch.voted_power.saturating_mul(10_000) / epoch.total_power >= vault.quorum_bps as u64;
        require!(quorum_ok, MandateError::NoQuorum);

        let mut best_i = 0usize;
        let mut best = 0u64;
        for (i, w) in epoch.weights.iter().enumerate() {
            if *w > best {
                best = *w;
                best_i = i;
            }
        }
        epoch.winner_idx = Some(best_i as u8);
        Ok(())
    }

    /// Executor reports an Ondo fill. v0 trusts the executor role;
    /// production must verify mint receipt / adapter CPI.
    pub fn execute_epoch(
        ctx: Context<ExecuteEpoch>,
        _fill_hash: [u8; 32],
        new_nav_e6: u64,
    ) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        require!(ctx.accounts.executor.key() == vault.executor, MandateError::BadExecutor);
        let epoch = &mut ctx.accounts.epoch;
        require!(epoch.winner_idx.is_some(), MandateError::NotTallied);
        require!(!epoch.executed, MandateError::AlreadyExecuted);
        require!(vault.idle_mode == IdleMode::HoldUsdy, MandateError::IdleModeBlocked);
        epoch.executed = true;
        vault.nav_usd_per_usdy_e6 = new_nav_e6.max(1);
        Ok(())
    }

    pub fn set_paused(ctx: Context<Auth>, paused: bool) -> Result<()> {
        ctx.accounts.vault.paused = paused;
        Ok(())
    }

    pub fn update_nav(ctx: Context<Auth>, nav_e6: u64) -> Result<()> {
        ctx.accounts.vault.nav_usd_per_usdy_e6 = nav_e6.max(1);
        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitializeVaultArgs {
    pub executor: Pubkey,
    pub nav_usd_per_usdy_e6: u64,
    pub management_fee_bps: u16,
    pub max_name_weight_bps: u16,
    pub quorum_bps: u16,
    pub min_quorum_notional_e6: u64,
    pub epoch_length_secs: i64,
    pub vote_length_secs: i64,
    pub commit_length_secs: i64,
}

#[derive(Accounts)]
pub struct InitializeVault<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        space = 8 + Vault::SIZE,
        seeds = [b"vault", authority.key().as_ref()],
        bump
    )]
    pub vault: Account<'info, Vault>,
    pub usdy_mint: Account<'info, Mint>,
    pub share_mint: Account<'info, Mint>,
    pub idle_treasury: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct OpenEpoch<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut, has_one = authority)]
    pub vault: Account<'info, Vault>,
    #[account(
        init,
        payer = authority,
        space = 8 + Epoch::SIZE,
        seeds = [b"epoch", vault.key().as_ref(), &(vault.current_epoch + 1).to_le_bytes()],
        bump
    )]
    pub epoch: Account<'info, Epoch>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub vault: Account<'info, Vault>,
    #[account(mut, has_one = vault)]
    pub epoch: Account<'info, Epoch>,
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + Position::SIZE,
        seeds = [b"pos", vault.key().as_ref(), epoch.index.to_le_bytes().as_ref(), user.key().as_ref()],
        bump
    )]
    pub position: Account<'info, Position>,
    #[account(mut)]
    pub user_usdy: Account<'info, TokenAccount>,
    #[account(mut, address = vault.idle_treasury)]
    pub idle_treasury: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CastVote<'info> {
    pub user: Signer<'info>,
    #[account(mut)]
    pub epoch: Account<'info, Epoch>,
    #[account(mut, has_one = owner)]
    pub position: Account<'info, Position>,
    /// CHECK: owner of position
    pub owner: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct Tally<'info> {
    pub vault: Account<'info, Vault>,
    #[account(mut, has_one = vault)]
    pub epoch: Account<'info, Epoch>,
}

#[derive(Accounts)]
pub struct ExecuteEpoch<'info> {
    pub executor: Signer<'info>,
    #[account(mut)]
    pub vault: Account<'info, Vault>,
    #[account(mut, has_one = vault)]
    pub epoch: Account<'info, Epoch>,
}

#[derive(Accounts)]
pub struct Auth<'info> {
    pub authority: Signer<'info>,
    #[account(mut, has_one = authority)]
    pub vault: Account<'info, Vault>,
}

#[error_code]
pub enum MandateError {
    #[msg("vault paused")]
    Paused,
    #[msg("ballot too large")]
    BallotTooLarge,
    #[msg("empty universe")]
    EmptyUniverse,
    #[msg("zero amount")]
    ZeroAmount,
    #[msg("bad lock multiplier")]
    BadLock,
    #[msg("not commit phase")]
    NotCommitPhase,
    #[msg("not vote phase")]
    NotVotePhase,
    #[msg("weight mismatch")]
    WeightMismatch,
    #[msg("already voted")]
    AlreadyVoted,
    #[msg("wrong epoch")]
    WrongEpoch,
    #[msg("vote still open")]
    VoteOpen,
    #[msg("already executed")]
    AlreadyExecuted,
    #[msg("quorum missed")]
    NoQuorum,
    #[msg("bad executor")]
    BadExecutor,
    #[msg("not tallied")]
    NotTallied,
    #[msg("idle mode blocked")]
    IdleModeBlocked,
    #[msg("overflow")]
    Overflow,
}
