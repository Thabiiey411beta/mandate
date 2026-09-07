# Mandate

**On-chain asset management on Solana, built on Ondo tokenized stocks.**

Users commit capital, vote which stock the vault buys, and hold a pro-rata claim (`oFUND`). Idle cash sits in **USDY** and earns short-Treasury NAV. Optionally, holders post `SPYon` as isolated collateral and borrow USDY without selling the equity.

Repo: https://github.com/Thabiiey411beta/mandate

This is not a broker and not an Ondo fork. Ondo issues `XXXon` and USDY. Mandate coordinates allocation, idle cash, fees, and a separate credit book.

---

## What the protocol is

Two products, two NAVs, one dashboard.

### 1. Epoch vault (`programs/mandate`)

Weekly (default) cycle:

1. **Commit** — deposit USDY (or USDC swapped to USDY). Idle sleeve = hold USDY. No farm.
2. **Vote** — capital-weighted, optional lock multiplier. `$ONDO` does **not** pick the stock.
3. **Tally** — plurality, 15% quorum, name caps.
4. **Execute** — legal wrapper mints the winning `XXXon` via Ondo. Program records the fill.

Share token `oFUND` is a claim on the basket + residual USDY, minus fees after ONDO rebate.

### 2. Credit sleeve (`programs/mandate-credit`)

Isolated market: post `SPYon`, borrow USDY.

- LTV 60% / liq 70% / penalty 4%
- Borrow APR floor = USDY NAV APY + 75 bps
- Same-slot recycle of the borrow into more of the same ticker is rejected
- Losses stay in this program. They never hit `oFUND`

### 3. Fee split (after user rebate, cap 80%)

```
net_fee → 50% OpCo / 25% insurance / 25% protocol (ONDO)
```

Credit interest: 25% protocol take (same split), 75% to USDY lenders.

Forbidden: skimming idle USDY yield, marking up Ondo mint/redeem.

---

## Why Solana first

Epoch voting is many small transactions. Fees must stay negligible. USDY and Ondo stocks already exist on Solana mainnet.

**Devnet limitation (read this once):**  
Real `SPYon` and `USDY` balances live on **mainnet**. Solana devnet cannot hold those tokens. Devnet testing therefore:

- Uses **live Ondo / public-market prices** (real numbers, not mocks)
- Uses **devnet SPL mints** created by `scripts/devnet-bootstrap.mjs` so wallets can deposit, vote, and borrow
- Treats those mints as stand-ins whose oracle is the official `SPYon` / USDY USD price

When you move to mainnet, swap mint addresses to:

| Asset | Solana mainnet mint |
|---|---|
| USDY | `A1KLoBrKBde8Ty9qtNQUtq3C2ortoC3u7twggz7sEto6` |
| SPYon | `k18WJUULWheRkSpSquYGdNNmtuE2Vbw1hpuUi92ondo` |

---

## Oracle

Ondo named **Chainlink** the official stock oracle. Those feeds are production on Ethereum. No public Chainlink SVM `SPYon` address is in Ondo docs yet.

Solana pricing stack:

| Asset | Source |
|---|---|
| SPYon | Ondo GM `primaryMarket` if `ONDO_API_KEY` is set; else live **SPY** cash market (Yahoo / Stooq) × Ondo total-return is approximated by the underlying until the key is set |
| USDY | Pyth USDY/USD when reachable; else last attested NAV from ondo.finance (~$1.14 band) fetched by the keeper |

Keeper: `apps/keeper` writes prices on-chain via `refresh_prices` once programs are deployed. The Next.js app reads the same HTTP sources so the UI never shows hardcoded $100 sliders as if they were NAV.

See [docs/ORACLE.md](docs/ORACLE.md).

---

## Repo map

```
programs/mandate          epoch vault, ONDO rebate
programs/mandate-credit   isolated SPYon / USDY credit
cli                       pdas, open-epoch, init-market, oracle-sync
apps/web                  Next.js vote + live prices + credit panel
apps/keeper               crank: pull live prices, log payload for refresh_prices
scripts/devnet-bootstrap.mjs   create devnet USDY + SPYon mints + config.json
docs/                     specs
```

---

## Devnet bring-up (operator machine)

Requires: Solana CLI, Anchor 0.31, Node 20, a funded devnet wallet (`solana airdrop 2`).

```bash
solana config set --url https://api.devnet.solana.com

# 1. Real program keypairs (not the dummy MandatE1111 ids)
solana-keygen new -o target/deploy/mandate-keypair.json
solana-keygen new -o target/deploy/mandate_credit-keypair.json
solana address -k target/deploy/mandate-keypair.json
# paste that pubkey into programs/mandate/src/lib.rs declare_id! and Anchor.toml

anchor build
anchor deploy --provider.cluster devnet

# 2. Devnet inventory + config (live price check included)
node scripts/devnet-bootstrap.mjs
# writes config/devnet.json

# 3. Keeper (live prices)
cp config/devnet.json apps/keeper/
ONDO_API_KEY=... npm run keeper   # optional key; works without it via SPY cash market

# 4. UI
cp config/devnet.json apps/web/devnet.json
cd apps/web && npm i && npm run dev
```

Phantom: switch to **Devnet**. Airdrop SOL. Bootstrap mints faucet is the mint authority in `config/devnet.json`.

---

## User flows to test

1. Deposit stand-in USDY in commit window → see vote power.  
2. Cast weights on Mag7 labels; winning name is recorded.  
3. Open credit: deposit stand-in SPYon, borrow stand-in USDY at 60% LTV using **live SPY and USDY prices**.  
4. Drop the oracle SPY print far enough in a local sim and run liquidate (do not fabricate UI prices; change the keeper feed only in a fork).

---

## Legal / risk (short)

Pooling capital to buy securities is a fund in most jurisdictions. The executor is a legal wrapper, not a raw DAO. Non-US first, same perimeter as Ondo Global Markets. Not audited. Do not put mainnet size in until oracles, mint adapter, and counsel exist.

---

## Specs

- [docs/EPOCH_SPEC.md](docs/EPOCH_SPEC.md)
- [docs/IDLE_YIELD.md](docs/IDLE_YIELD.md)
- [docs/FEE_SPLIT.md](docs/FEE_SPLIT.md)
- [docs/CREDIT_SLEEVE.md](docs/CREDIT_SLEEVE.md)
- [docs/ONDO_REBATE.md](docs/ONDO_REBATE.md)
- [docs/ORACLE.md](docs/ORACLE.md)
