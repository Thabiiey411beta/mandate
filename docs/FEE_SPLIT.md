# Mandate fee split

Version: 0.1  
Status: design spec  
Scope: who gets paid, in what order, and what is forbidden.

## Principle

Fees exist so the **legal wrapper can stay alive** and so aligned holders get a defined claim.  
They do **not** exist to harvest idle USDY yield or to widen Ondo mint spreads.

If a dollar of revenue would make the product worse than buying `NVDAon` directly, it is not protocol revenue. It is a leak of trust.

## Revenue that is allowed

| Line | Default | Paid by |
|---|---|---|
| Management | 75 bps / yr on vault NAV | all `oFUND` |
| Performance | 10% of NAV gain vs high-water, crystallize end of epoch | all `oFUND` |
| Credit interest | borrow APR on USDY drawn vs `XXXon` | borrowers |
| Origination | 0–25 bps on new borrows | borrowers |
| Liquidation penalty | 3–5% of repaid principal | unhealthy credit positions |
| Slashed proposal bonds | 100% of slash | proposers who fail checks |

## Revenue that is forbidden

- Skimming USDY / T-bill NAV while cash is idle in an epoch
- Marking up Ondo mint/redeem vs the quoted fill
- Points, emissions, or “protocol owned liquidity” funded by depositors
- Silent performance fee on unrealized NAV during an open epoch

## Waterfall (every crystallization)

Apply **after** user-level ONDO rebate has reduced that user’s fee bill.

```
gross_fee
  → user rebate (ONDO stake)     # not protocol income
  → net_fee
       50%  OpCo / legal wrapper
       25%  Insurance + liquidation backstop
       25%  Protocol (ONDO: buyback or staker reward — DAO switch)
```

### Caps so the business still exists

- Max rebate per user: **80%** of their fee (not 100%).  
  Whale with max ONDO still pays 20% of schedule = 15 bps management at a 75 bps sticker.
- Protocol bucket cannot move above 35% without a 14-day timelock + guardian.
- OpCo bucket cannot fall below 40% while the wrapper is the regulated executor.
- Insurance target: 2% of credit outstanding + 0.3% of vault NAV. Overflow of insurance goes to Protocol.

### Worked example

$40m vault NAV, blended after rebate = 45 bps effective management.  
Annual management = $180,000.

| Bucket | $ |
|---|---|
| OpCo | 90,000 |
| Insurance | 45,000 |
| Protocol / ONDO | 45,000 |

Add credit interest only if the lending sleeve is live (see CREDIT_SLEEVE.md). At $10m borrows × 4% net interest × 50/25/25 same split.

## Rebate vs split (do not double-count)

1. Compute contractual fee for the user.  
2. Apply `rebate_bps` (capped 8,000).  
3. Remaining `net_fee` hits the waterfall.  
4. ONDO stakers may *also* receive the Protocol bucket. That is a second, explicit flow — document it. Do not hide a third skim.

Formula stays:

```
rebate_bps = min(8000, staked_ondo_usd / (0.05 * user_nav_usd) * 10000)
```

## Credit-sleeve fees (when enabled)

| Param | Default |
|---|---|
| USDY borrow APR | max(USDY NAV APY + 75 bps, utilization curve) |
| Protocol share of interest | same 50/25/25 waterfall |
| Reserve factor already inside insurance 25% | do not add a second reserve |
| Liquidation penalty | 4%, of which 2% insurance, 1% liquidator, 1% OpCo |

Borrow APR must stay **above** USDY’s embedded T-bill yield. Otherwise borrowers loop USDY to farm the same yield the collateral already funds and the book is hollow.

## Settlement rail

- Fees accrue in **USDY**.  
- Protocol bucket: USDY held, then periodic buy-and-lock or buy-and-distribute ONDO (DAO param). No daily market-buy spam.  
- OpCo bucket: USDY to the wrapper treasury ATA.  
- Insurance: dedicated PDA, never the authority’s pocket.

## Switch order (do not skip)

1. OpCo can invoice and receive 50%.  
2. Insurance PDA live + reporting.  
3. Only then enable Protocol/ONDO distribution.  
4. Credit sleeve fees follow the same waterfall from day one of borrows.

## What “profitable” means

Mandate is profitable when:

`OpCo + insurance accretion > legal + executor + audit + RPC`

and depositors still beat “buy the stock token yourself” by coordination + idle USDY + credit optionality.

It is not profitable because ONDO went up.
