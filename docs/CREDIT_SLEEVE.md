# Credit sleeve — collateralize XXXon, borrow USDY

Version: 0.1  
Status: design spec, not implemented  
Relation to vault: **separate program / separate share class**. Do not mix borrow risk into `oFUND` NAV.

## What users asked for

Holders of Ondo tokenized stocks should be able to post them as collateral, draw **USDY**, and keep the stock’s economic upside (price + Ondo total-return / reinvested dividends). Idle equity stops being dead wallet inventory. Drawn USDY can sit in Tier-0 (earn T-bill NAV) or enter a Mandate epoch as *new* committed capital.

That is a **credit loop**, not a second yield farm on the same dollars.

```
XXXon  →  isolated market  →  USDY out
                ↑                    ↓
         liquidations         hold USDY (Tier 0)
                              or deposit to Mandate vault
```

## Why this can be “sound”

- Collateral is exchange-backed tokenized equity (Ondo), not an unbacked perp.
- Debt is USDY, the same cash asset the vault already treats as idle sleeve.
- Stock “earns” because `XXXon` is a total-return token, not because we invent a farm.
- Protocol earns **interest**, which is real if utilization is real.

## Why this can also kill the product

- NVDA −30% in a week + thin on-chain liquidity → bad debt.
- Users borrow USDY, dump it, and the book is left holding the bag.
- Users loop: borrow USDY → buy more `XXXon` → borrow again. That is leverage, not “passive income.”
- Mixing this risk into `oFUND` makes epoch voters subsidize borrowers.

Closed loop is allowed. **Infinite loop is not.**

## Market design (v1)

One isolated pool per allowed ticker. No cross-margin. No e-mode basket until we have liquidations that actually work on Solana hours.

### Allowlist (start tiny)

Only deep names: `SPYon`, `QQQon`, `NVDAon`, `AAPLon`, `MSFTon`.  
Add `TSLAon` later; it is a liquidation factory.

### Risk params (genesis)

| Collateral | Max LTV | Liq threshold | Liq penalty | Oracle |
|---|---|---|---|---|
| `SPYon` / `QQQon` | 60% | 70% | 4% | Ondo NAV + max(DEX TWAP, 30s) |
| Mega-cap single name | 45% | 55% | 5% | same |
| Everything else | off | — | — | — |

Debt asset: **USDY only**. No USDC debt in v1 (keeps the closed loop inside Ondo cash).

Supply of USDY into the credit program comes from:

1. Insurance + OpCo idle (small), and/or  
2. A dedicated `mUSDY` lender vault — users who *opt in* to earn borrow interest on top of USDY NAV.  
   This is **not** the epoch vault.

### Interest

```
borrow_apr = usdY_nav_apy + 0.75% + kink_curve(utilization)
kink = 50% utilization → +0 bps extra
      80%             → +300 bps extra
      100%            → +1200 bps extra
```

Floor at `USDY NAV APY + 75 bps` so nobody borrows USDY to “earn the same T-bill twice.”

Lender APY ≈ borrow_apr × utilization × (1 − protocol_take).  
Protocol take = fee waterfall in FEE_SPLIT.md (interest is `net_fee` before 50/25/25).

## Closed-loop flows that are allowed

1. **Hold and spend**  
   Post `NVDAon`, borrow 40% LTV in USDY, spend or hold USDY. Stock stays in the credit PDA; total return still accrues on the token.

2. **Hold and mandate**  
   Same borrow, deposit USDY into the *epoch vault* as a separate position. Vote with that cash. If NVDA dumps, credit sleeve liquidates independently of the vault.

3. **Hold and sit**  
   Borrow 0%. Collateral just sits; no fee. This is worse than leaving the token in-wallet; don’t incentivize empty deposits.

## Flows that are forbidden in core

- Recycle borrowed USDY to buy *the same* `XXXon` inside the credit program (self-loop). Hard cap: borrowed USDY cannot mint/buy the collateral ticker in the same tx tree.
- Using `oFUND` shares as collateral in v1.
- Cross-collateral “Mag7 basket LTV 70%.”
- Borrowing to lever the idle-yield story (“4% on USDY plus 4% on stocks plus 4% on the borrow”). That pitch is a lie; publish the net.

Net for a 45% LTV NVDA borrower who holds USDY:

```
stock total return
+ USDY NAV APY on the borrowed dollars
− borrow APR
− liquidation option (left-tail)
```

If stock return is 10% and USDY is 3.5% and borrow is 5%, net on full equity is about:

`10% + 0.45*(3.5%-5%) = 10% - 0.7% = 9.3%` on equity, with crash risk.  
The “extra money” is small. Sell it as **liquidity against appreciated stock**, not as free yield.

## Liquidations

- Keepers repay USDY, receive `XXXon` at threshold × (1 + penalty).
- Bonus must clear Solana priority fees + slippage on the name.
- If Ondo mint/redeem is halted, pause borrows and raise LTV freeze; do not pretend a DEX pool of $3m USDY is the backstop for a $20m book.
- Daily collateral cap per ticker (start $2m).
- Protocol insolvency → insurance PDA first, then freeze, then socialize *only the credit sleeve*. Epoch vault stays untouched.

## Accounting wall

| Product | Token | Risk |
|---|---|---|
| Epoch allocator | `oFUND` | market + mandate |
| Credit sleeve | `cNVDAon` claim / borrow USDY | credit + liquidation |
| Idle cash | raw USDY | Ondo / T-bill |

One dashboard. Three balances. Never one NAV.

## Implementation slice (after vault v0)

1. Isolated market program, USDY debt, one ticker (`SPYon`).  
2. Oracle: Ondo official price; refuse updates older than 120s.  
3. No loop instruction.  
4. Cap + pause.  
5. Only then Mag7 names.

## Copy (honest)

> You can borrow USDY against selected Ondo stocks without selling them. The stock still tracks total return. USDY still earns Treasury NAV if you hold it. You pay interest. If the stock falls far enough, you get liquidated. This is credit, not a second printing press.
