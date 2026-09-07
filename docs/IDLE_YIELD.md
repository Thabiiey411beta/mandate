# Idle cash: safest yield on USDY (Solana)

USDY (Ondo US Dollar Yield) is a non-rebasing note. Yield shows up as **NAV appreciation**, not extra tokens. Current official APY is about **3.55%**, collateralization ~105%, reserves almost entirely short US Treasuries.

Solana mint: `A1KLoBrKBde8Ty9qtNQUtq3C2ortoC3u7twggz7sEto6`

## What “sound money, passive” actually means here

Users are waiting 4–6 days each epoch. The job of idle cash is:

1. Not lose the dollar.
2. Keep mint/redeem optionality into Ondo stocks.
3. Earn *something real* (T-bills), not farm emissions.

That stack, ranked.

### Tier 0 — Hold USDY (default, v0)

**How it works:** vault keeps unallocated capital as USDY. Price per token rises with Treasury coupon minus fees.

**Why it is the best risk-adjusted sleeve for this product**
- Same issuer family as the stocks you will mint (`USDon` / OGM).
- No smart-contract leverage, no LP inventory, no borrower default.
- Bankruptcy-remote note + daily/monthly attestations.
- Instantly convertible toward stock mints when T4 hits (via USDon/USDC path).
- Yield is *already* passive. No harvest tx.

**Yield:** ~3.5% APY as of Sep 2026. Not exciting. That is the point.

**Risks that remain:** Ondo/issuer/custodian, USDY transfer allow/block lists, NAV vs secondary DEX premium/discount on Solana (~$3M USDY-USDC pool — do not size exits through it).

**Mandate policy:** 100% of idle cash in Tier 0 until a guardian vote + audit lifts the cap.

### Tier 1 — Isolated lend USDY, never borrow (later)

Only if a Solana venue lists USDY as **isolated** collateral or a supply-only reserve with:
- no recursive looping in the vault,
- withdrawable inside the execute window (same day),
- oracle = Ondo NAV, not a thin DEX TWAP,
- hard cap ≤ 25% of idle cash in v1 experiments.

Kamino is the deepest Solana lender and already takes tokenized stocks as collateral elsewhere. That does **not** automatically make a USDY supply market safe. Treat each reserve as its own diligence item.

Expected extra yield: 0–2% over NAV if anyone actually borrows USDY. Often the market will pay *less* than just holding USDY, in which case Tier 1 is strictly worse.

### Tier 2 — Forbidden in the core vault

- USDY/USDC CLMM (impermanent loss vs a rising NAV token).
- Looping USDY to buy more USDY or stocks.
- Pendle / Exponent PTs (duration + venue risk during a 7-day epoch).
- Basis trades, perps, points programs.
- Bridging idle cash off Solana every epoch.

Those can live in a *separate* “alpha sleeve” product with its own share class. Mixing them into `oFUND` contaminates the mandate.

## Implementation in this repo

`IdlePolicy` on the `Vault` account:

```
IdleMode::HoldUsdy          // v0
IdleMode::SupplyIsolated    // disabled until allowlisted program id set
```

`accrue_nav` reads an official USDY NAV price account (placeholder Pyth/Ondo feed) so `oFUND` exchange rate rises even when no deposits happen.

## User-facing copy (honest)

> While your vote runs, cash sits in Ondo USDY. You earn the same short-Treasury yield USDY holders earn. We do not farm it.

Do not advertise “8% stable yield” on idle capital. That is how this product dies.
