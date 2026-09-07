#!/usr/bin/env npx tsx
import { Command } from "commander";
import {
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
  Transaction,
  TransactionInstruction,
  sendAndConfirmTransaction,
} from "@solana/web3.js";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { epochPda, vaultPda } from "./pda.ts";
import { PROGRAM_ID_DEVNET } from "./idl.ts";
import { CREDIT_PROGRAM, MINTS, fetchOndoPrimaryE6, marketPda, oraclePda } from "./credit.ts";

const DEFAULT_RPC = process.env.SOLANA_RPC ?? "https://api.devnet.solana.com";
const PROGRAM_ID = new PublicKey(process.env.MANDATE_PROGRAM_ID ?? PROGRAM_ID_DEVNET);

function loadKeypair(p?: string) {
  const file = p ?? process.env.SOLANA_KEYPAIR ?? path.join(os.homedir(), ".config/solana/id.json");
  const raw = JSON.parse(fs.readFileSync(file, "utf8")) as number[];
  return Keypair.fromSecretKey(Uint8Array.from(raw));
}

function disc(name: string) {
  const { createHash } = require("node:crypto") as typeof import("node:crypto");
  return createHash("sha256").update(`global:${name}`).digest().subarray(0, 8);
}

function encodeStringVec(labels: string[]) {
  const parts = [u32(labels.length)];
  for (const s of labels) {
    const b = Buffer.from(s);
    parts.push(u32(b.length), b);
  }
  return Buffer.concat(parts);
}

function encodePubkeyVec(keys: PublicKey[]) {
  const parts = [u32(keys.length)];
  for (const k of keys) parts.push(k.toBuffer());
  return Buffer.concat(parts);
}

function u32(n: number) {
  const b = Buffer.alloc(4);
  b.writeUInt32LE(n);
  return b;
}

const program = new Command("mandate");

program
  .command("pdas")
  .requiredOption("--authority <pubkey>")
  .option("--epoch <n>", "epoch index", "1")
  .action((opts) => {
    const authority = new PublicKey(opts.authority);
    const [vault] = vaultPda(PROGRAM_ID, authority);
    const [epoch] = epochPda(PROGRAM_ID, vault, BigInt(opts.epoch));
    console.log(JSON.stringify({ programId: PROGRAM_ID.toBase58(), vault: vault.toBase58(), epoch: epoch.toBase58() }, null, 2));
  });

program
  .command("open-epoch")
  .description("Open the next epoch with a Mag7-style ballot")
  .option("--rpc <url>", "rpc", DEFAULT_RPC)
  .option("--keypair <path>")
  .option("--tickers <list>", "comma labels", "NVDAon,TSLAon,AAPLon,MSFTon,GOOGLon,METAon,AMZNon")
  .option("--mints <list>", "comma mint pubkeys")
  .action(async (opts) => {
    const kp = loadKeypair(opts.keypair);
    const connection = new Connection(opts.rpc, "confirmed");
    const labels: string[] = opts.tickers.split(",").map((s: string) => s.trim());
    const mints = opts.mints
      ? (opts.mints as string).split(",").map((s: string) => new PublicKey(s.trim()))
      : labels.map((_, i) => {
          const seed = Buffer.alloc(32, i + 1);
          return Keypair.fromSeed(seed).publicKey;
        });
    if (mints.length !== labels.length) throw new Error("mints/tickers length mismatch");
    const [vault] = vaultPda(PROGRAM_ID, kp.publicKey);
    const vaultInfo = await connection.getAccountInfo(vault);
    if (!vaultInfo) {
      console.error("Vault PDA not initialized:", vault.toBase58());
      process.exit(1);
    }
    const nextIndex = BigInt(process.env.NEXT_EPOCH ?? "1");
    const [epoch] = epochPda(PROGRAM_ID, vault, nextIndex);
    const data = Buffer.concat([disc("open_epoch"), encodePubkeyVec(mints), encodeStringVec(labels)]);
    const ix = new TransactionInstruction({
      programId: PROGRAM_ID,
      keys: [
        { pubkey: kp.publicKey, isSigner: true, isWritable: true },
        { pubkey: vault, isSigner: false, isWritable: true },
        { pubkey: epoch, isSigner: false, isWritable: true },
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      ],
      data,
    });
    const sig = await sendAndConfirmTransaction(connection, new Transaction().add(ix), [kp]);
    console.log(JSON.stringify({ signature: sig, vault: vault.toBase58(), epoch: epoch.toBase58(), index: nextIndex.toString(), labels }, null, 2));
  });

program
  .command("init-market")
  .description("Print PDAs for a SPYon/USDY isolated market")
  .option("--ticker <t>", "ticker", "SPYon")
  .option("--collateral <mint>")
  .option("--debt <mint>")
  .action((opts) => {
    const ticker = opts.ticker as string;
    const coll = new PublicKey(opts.collateral ?? (MINTS as Record<string, string>)[ticker] ?? MINTS.SPYon);
    const debt = new PublicKey(opts.debt ?? MINTS.USDY);
    const [market] = marketPda(coll, debt);
    const [oracle] = oraclePda(market);
    console.log(JSON.stringify({
      creditProgram: CREDIT_PROGRAM.toBase58(),
      ticker,
      collateral: coll.toBase58(),
      debt: debt.toBase58(),
      market: market.toBase58(),
      oracle: oracle.toBase58(),
      ltvBps: 6000,
      liqBps: 7000,
      penaltyBps: 400,
    }, null, 2));
  });

program
  .command("oracle-sync")
  .description("Fetch Ondo GM primaryMarket price (official total-return quote)")
  .requiredOption("--symbol <s>", "e.g. SPYon")
  .action(async (opts) => {
    const px = await fetchOndoPrimaryE6(opts.symbol);
    console.log(JSON.stringify({ symbol: opts.symbol, ...px, usd: px.e6 / 1_000_000 }, null, 2));
  });

program.parseAsync(process.argv);
