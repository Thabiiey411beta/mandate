# Mandate

Solana-first collective allocation protocol on Ondo tokenized stocks.

Repo: https://github.com/Thabiiey411beta/mandate

## Layout

```
programs/mandate   Anchor program (vault, epoch, rebate)
cli                thin CLI — open-epoch, pdas
apps/web           Next.js devnet vote UI
docs               epoch / idle USDY / ONDO rebate
```

## ONDO rebate

See [docs/ONDO_REBATE.md](docs/ONDO_REBATE.md). Stake ONDO on a per-user `RebateAccount`. Fees rebate up to 100%. Unstake cooldown is 7 days.

## CLI

```bash
npm install
export SOLANA_RPC=https://api.devnet.solana.com
npx tsx cli/src/index.ts pdas --authority <YOUR_PUBKEY>
NEXT_EPOCH=1 npx tsx cli/src/index.ts open-epoch --tickers NVDAon,TSLAon,AAPLon,MSFTon,GOOGLon,METAon,AMZNon
```

`open-epoch` needs a deployed program + `initialize_vault`. Until then it prints the missing vault PDA.

## Vote UI

```bash
cd apps/web
cp .env.example .env.local   # set NEXT_PUBLIC_VAULT
npm install
npm run dev
```

Connect Phantom on **devnet**, paste the epoch PDA from the CLI, allocate sliders, cast vote.

## Status

Scaffold. Placeholder program id. Not audited. Do not deposit mainnet funds.
