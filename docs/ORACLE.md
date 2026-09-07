# Oracle

Ondo named **Chainlink** the official oracle for tokenized stocks (`SPYon`, `QQQon`, `TSLAon`, …). Those production feeds are documented live on **Ethereum** (Euler). A public Chainlink SVM feed address for `SPYon` is not in Ondo’s Solana address book yet.

Mandate on Solana therefore uses a two-leg official stack:

| Leg | Source | Why |
|---|---|
| USDY / USD | Pyth `USDY/USD` (Ondo–Pyth partnership, 65+ chains including Solana) | On-chain pull account |
| `SPYon` / USD | Ondo Global Markets **primaryMarket** price (`api.gm.ondo.finance`) posted on-chain | Same number mint/redeem uses; includes total-return / dividends |

Do **not** price collateral off a thin Solana DEX TWAP.

## On-chain accounts

`OndoOracle` PDA `["ondo_oracle", market]`

| Field | Meaning |
|---|---|
| `collat_usd_e6` | last Ondo primary market price * 1e6 |
| `usdy_usd_e6` | last Pyth or Ondo USDY NAV * 1e6 |
| `collat_ts` / `usdy_ts` | unix publish time |
| `pyth_usdy_feed` | Pyth price-update account |
| `max_age_secs` | default 120 |

`refresh_prices` requires both legs fresh before borrow / withdraw collateral / liquidate.

## CLI

```bash
# push Ondo GM + optional Pyth hermes into the cache (keeper)
npx tsx cli/src/index.ts oracle-sync --market <MARKET_PDA> --symbol SPYon

# one-ticker market
npx tsx cli/src/index.ts init-market --ticker SPYon \
  --collateral k18WJUULWheRkSpSquYGdNNmtuE2Vbw1hpuUi92ondo \
  --debt A1KLoBrKBde8Ty9qtNQUtq3C2ortoC3u7twggz7sEto6
```

USDY Solana mint: `A1KLoBrKBde8Ty9qtNQUtq3C2ortoC3u7twggz7sEto6`  
SPYon Solana mint: `k18WJUULWheRkSpSquYGdNNmtuE2Vbw1hpuUi92ondo`
