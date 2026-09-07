use anchor_lang::prelude::*;

/// Cache of official Ondo + Pyth prices. Keeper / crank writes here.
#[account]
pub struct OndoOracle {
    pub market: Pubkey,
    pub authority: Pubkey,
    pub pyth_usdy_feed: Pubkey,
    pub collat_usd_e6: u64,
    pub usdy_usd_e6: u64,
    pub collat_ts: i64,
    pub usdy_ts: i64,
    pub max_age_secs: i64,
    pub bump: u8,
}

impl OndoOracle {
    pub const SIZE: usize = 32 * 3 + 8 * 4 + 8 + 1 + 16;

    pub fn collat_fresh(&self, now: i64) -> bool {
        now.saturating_sub(self.collat_ts) <= self.max_age_secs && self.collat_usd_e6 > 0
    }

    pub fn usdy_fresh(&self, now: i64) -> bool {
        now.saturating_sub(self.usdy_ts) <= self.max_age_secs && self.usdy_usd_e6 > 0
    }
}

/// Best-effort read of a Pyth price-update account.
/// Layout-tolerant: looks for a plausible i64 price + i32 expo near the tail of
/// the first 256 bytes. Keeper path is the source of truth for SPYon.
pub fn try_read_pyth_e6(data: &[u8]) -> Option<(u64, i64)> {
    if data.len() < 64 {
        return None;
    }
    // Pyth v2 price update commonly stores price as i64 then expo i32.
    // We scan 8-byte aligned windows for expo in [-12, 0] and price > 0.
    for off in (0..data.len().saturating_sub(16)).step_by(8) {
        if off + 12 > data.len() {
            break;
        }
        let price = i64::from_le_bytes(data[off..off + 8].try_into().ok()?);
        let expo = i32::from_le_bytes(data[off + 8..off + 12].try_into().ok()?);
        if price <= 0 || expo > 0 || expo < -12 {
            continue;
        }
        let e6 = scale_to_e6(price, expo)?;
        if e6 > 100_000 && e6 < 50_000_000 {
            // USDY should sit roughly $0.10–$50 in e6 after scale? NAV ~1.14e6
            return Some((e6, Clock::get().ok()?.unix_timestamp));
        }
        if e6 >= 500_000 && e6 <= 3_000_000 {
            return Some((e6, Clock::get().ok()?.unix_timestamp));
        }
    }
    None
}

fn scale_to_e6(price: i64, expo: i32) -> Option<u64> {
    let p = price as i128;
    let target = expo + 6;
    let scaled = if target >= 0 {
        p.checked_mul(10i128.pow(target as u32))?
    } else {
        p.checked_div(10i128.pow((-target) as u32))?
    };
    if scaled <= 0 {
        return None;
    }
    u64::try_from(scaled).ok()
}
