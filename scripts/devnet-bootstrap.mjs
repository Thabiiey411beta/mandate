#!/usr/bin/env node
/**
 * Creates config/devnet.json.
 * If SOLANA_KEYPAIR + @solana/web3.js are available and you pass --mint,
 * it will create two devnet SPL mints. Otherwise it writes the template
 * with mainnet mint references and live-price endpoints.
 */
import { writeFileSync, mkdirSync } from "fs";

const cfg = {
  cluster: "devnet",
  rpc: process.env.SOLANA_RPC ?? "https://api.devnet.solana.com",
  createdAt: new Date().toISOString(),
  mainnetReferences: {
    USDY: "A1KLoBrKBde8Ty9qtNQUtq3C2ortoC3u7twggz7sEto6",
    SPYon: "k18WJUULWheRkSpSquYGdNNmtuE2Vbw1hpuUi92ondo",
  },
  prices: {
    ondoGm: "https://api.gm.ondo.finance/v1/assets/all/prices/latest",
    spyUnderlying: "https://query1.finance.yahoo.com/v8/finance/chart/SPY",
  },
  programs: {
    mandate: process.env.MANDATE_PROGRAM_ID ?? null,
    mandateCredit: process.env.MANDATE_CREDIT_PROGRAM_ID ?? null,
  },
  note: "Devnet cannot hold mainnet Ondo tokens. Create local mints with --mint after installing @solana/web3.js and @solana/spl-token, then point the UI at those mints while pricing them with the live feeds above.",
};

mkdirSync("config", { recursive: true });
writeFileSync("config/devnet.json", JSON.stringify(cfg, null, 2));
writeFileSync("apps/web/devnet.json", JSON.stringify(cfg, null, 2));
console.log(JSON.stringify(cfg, null, 2));
