# Mandate

Solana-first collective allocation protocol on Ondo tokenized stocks.

Repo: https://github.com/Thabiiey411beta/mandate

## Layout

```
programs/mandate          epoch vault + ONDO rebate
programs/mandate-credit   isolated SPYon / USDY lending (separate NAV)
cli                       open-epoch
apps/web                  Next.js devnet vote UI
docs                      specs
```

## Credit sleeve (new)

Isolated market: post `SPYon`, borrow `USDY`. Not mixed into `oFUND`.

Genesis: LTV 60%, liq 70%, penalty 4%. Borrow floor = USDY APY + 75 bps.  
Interest: 25% protocol take, then 50/25/25 OpCo / insurance / ONDO.  
Same-slot borrow → re-deposit of that ticker is rejected (anti-loop).

See [docs/CREDIT_SLEEVE.md](docs/CREDIT_SLEEVE.md) and [docs/FEE_SPLIT.md](docs/FEE_SPLIT.md).

## Status

Scaffold. Placeholder program ids. Not audited. Do not deposit mainnet funds.
