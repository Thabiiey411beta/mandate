# Mandate

Solana-first collective allocation protocol on [Ondo](https://ondo.finance) tokenized stocks.

Users commit capital, vote which `XXXon` stock the vault should buy, and receive a pro-rata vault share. Unallocated cash sits in **USDY** and accrues Treasury yield until execution. `$ONDO` is used for proposer bonds and fee rebates — not for stock-pick weight.

**Repo:** https://github.com/Thabiiey411beta/mandate

## Why Solana first

- Epoch voting is many small txs (commit, vote, tally). Fees must stay negligible.
- Ondo Stocks + USDY already live on Solana.
- Kamino is the deepest venue if we ever graduate idle cash *beyond* native USDY accrual — v0 does **not** do that.

## Product loop

```
USDC / USDon  →  vault (idle USDY)  →  capital-weighted vote  →  Ondo mint XXXon  →  oFUND shares
```

See [docs/EPOCH_SPEC.md](docs/EPOCH_SPEC.md) and [docs/IDLE_YIELD.md](docs/IDLE_YIELD.md).

## Safety stance (idle cash)

| Tier | Strategy | v0 |
|---|---|---|
| 0 | Hold USDY, earn embedded T-bill NAV (~3.5% APY) | **default, shipped** |
| 1 | USDY as isolated collateral on a blue-chip Solana lender, borrow none | gated, opt-in later |
| 2 | LP USDY/USDC | **out of scope** |
| 3 | Looping, farms, points, Pendle PTs | **forbidden in core vault** |

Native USDY already *is* the sound-money sleeve. Adding DeFi yield on top of T-bills is optional alpha, not the product.

## Programs

| Program | Role |
|---|---|
| `mandate` | Vault, epochs, votes, shares |
| (adapter) | Ondo mint/redeem — executor-gated in v0 |

## Quick start

```bash
# requires Solana CLI + Anchor 0.31+
anchor build
anchor test
```

Devnet mints in `Anchor.toml` are placeholders. USDY mainnet mint:

`A1KLoBrKBde8Ty9qtNQUtq3C2ortoC3u7twggz7sEto6`

## Status

Scaffold. Not audited. Not a registered fund. Non-US perimeter. Do not deposit mainnet funds.
