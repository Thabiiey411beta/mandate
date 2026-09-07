# ONDO rebate accounts

PDA: `["rebate", vault, user]`

| Field | Meaning |
|---|---|
| `staked_ondo` | ONDO locked for fee rebate |
| `pending_unstake` | in 7-day cooldown |
| `accrued_rebate_usdy` | fee credits waiting to be paid |

Formula (same as spec):

```
rebate_bps = min(10000, staked_ondo_usd / (0.05 * user_nav_usd) * 10000)
```

`$5` of ONDO per `$100` of vault NAV → 100% management/performance fee rebate.

Stake does **not** change stock-pick vote power.

Instructions: `init_rebate`, `stake_ondo`, `request_unstake`, `complete_unstake`, `harvest_rebate`.
