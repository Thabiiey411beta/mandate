# Mandate epoch voting spec (Solana v0)

Full design notes live in the product conversation. This file is what the program implements.

## Calendar (7d default)

| Phase | Slots / time |
|---|---|
| Commit | days 0–4 |
| Ballot freeze | day 4 |
| Vote | days 4–6 |
| Tally + execute | day 6–7 |

Unix timestamps on `Epoch` account, not slot math, so guardians can align to UTC weeks.

## Voting power

```
vote_power = deposited_usdy_atoms * lock_mult / 100
lock_mult: liquid=100, 30d=110, 90d=125, 180d=180
per-wallet cap: 10% of epoch power
```

ONDO stake does not multiply this.

## Winner

Plurality of weight. If winner fails mintability or cap, runner-up then cash.
Quorum: 15% of power AND min notional (param).

## Execution

`executor` role calls `execute_epoch` with an Ondo fill receipt hash. v0 does not CPI into Ondo mint; the legal wrapper mints and the program records balances. v1 can add a checked adapter.
