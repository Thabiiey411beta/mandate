"use client";

import { useMemo, useState } from "react";
import { useConnection, useWallet } from "@solana/wallet-adapter-react";
import { WalletMultiButton } from "@solana/wallet-adapter-react-ui";
import { PublicKey, Transaction, TransactionInstruction } from "@solana/web3.js";
import { createHash } from "crypto";

const PROGRAM_ID = new PublicKey(
  process.env.NEXT_PUBLIC_MANDATE_PROGRAM_ID ?? "MandatE11111111111111111111111111111111111"
);

const DEMO_BALLOT = [
  { label: "NVDAon", mint: "11111111111111111111111111111112" },
  { label: "TSLAon", mint: "11111111111111111111111111111113" },
  { label: "AAPLon", mint: "11111111111111111111111111111114" },
  { label: "MSFTon", mint: "11111111111111111111111111111115" },
  { label: "GOOGLon", mint: "11111111111111111111111111111116" },
  { label: "METAon", mint: "11111111111111111111111111111117" },
  { label: "AMZNon", mint: "11111111111111111111111111111118" },
];

function disc(name: string) {
  return createHash("sha256").update(`global:${name}`).digest().subarray(0, 8);
}

function encodeU64Vec(xs: number[]) {
  const parts: Buffer[] = [Buffer.alloc(4)];
  parts[0].writeUInt32LE(xs.length);
  for (const x of xs) {
    const b = Buffer.alloc(8);
    b.writeBigUInt64LE(BigInt(x));
    parts.push(b);
  }
  return Buffer.concat(parts);
}

export default function Page() {
  const { connection } = useConnection();
  const { publicKey, sendTransaction, connected } = useWallet();
  const [weights, setWeights] = useState(DEMO_BALLOT.map(() => 0));
  const [epochPk, setEpochPk] = useState("");
  const [status, setStatus] = useState("devnet UI — set epoch PDA after `mandate open-epoch`");

  const total = useMemo(() => weights.reduce((a, b) => a + b, 0), [weights]);

  async function vote() {
    if (!publicKey) return;
    if (!epochPk) {
      setStatus("Paste the epoch PDA from the CLI output.");
      return;
    }
    if (total <= 0) {
      setStatus("Put weight on at least one name.");
      return;
    }
    const epoch = new PublicKey(epochPk);
    const idx = BigInt(process.env.NEXT_PUBLIC_EPOCH_INDEX ?? "1");
    const vaultEnv = process.env.NEXT_PUBLIC_VAULT;
    if (!vaultEnv) {
      setStatus("Set NEXT_PUBLIC_VAULT to the vault PDA.");
      return;
    }
    const vault = new PublicKey(vaultEnv);
    const buf = Buffer.alloc(8);
    buf.writeBigUInt64LE(idx);
    const [position] = PublicKey.findProgramAddressSync(
      [Buffer.from("pos"), vault.toBuffer(), buf, publicKey.toBuffer()],
      PROGRAM_ID
    );

    const data = Buffer.concat([disc("cast_vote"), encodeU64Vec(weights.map((w) => Math.max(0, Math.floor(w))))]);
    const ix = new TransactionInstruction({
      programId: PROGRAM_ID,
      keys: [
        { pubkey: publicKey, isSigner: true, isWritable: false },
        { pubkey: epoch, isSigner: false, isWritable: true },
        { pubkey: position, isSigner: false, isWritable: true },
      ],
      data,
    });
    try {
      const sig = await sendTransaction(new Transaction().add(ix), connection);
      setStatus(`sent ${sig}`);
    } catch (e: unknown) {
      setStatus(e instanceof Error ? e.message : String(e));
    }
  }

  return (
    <div className="wrap">
      <div className="row">
        <div>
          <h1>Mandate</h1>
          <p className="muted">Devnet vote. Idle cash stays in USDY. ONDO stake rebates fees, it does not pick the stock.</p>
        </div>
        <WalletMultiButton />
      </div>

      <div className="card">
        <div className="muted">Epoch PDA</div>
        <input
          style={{ width: "100%", marginTop: 8, padding: 10, borderRadius: 8, border: "1px solid #1d2430", background: "#0b0e14", color: "#e8edf5" }}
          placeholder="Epoch public key from CLI"
          value={epochPk}
          onChange={(e) => setEpochPk(e.target.value)}
        />
      </div>

      <div className="card">
        {DEMO_BALLOT.map((t, i) => (
          <div key={t.label} style={{ marginBottom: 16 }}>
            <div className="row">
              <strong className="ticker">{t.label}</strong>
              <span className="muted">{weights[i]}%</span>
            </div>
            <input
              type="range"
              min={0}
              max={100}
              value={weights[i]}
              onChange={(e) => {
                const next = [...weights];
                next[i] = Number(e.target.value);
                setWeights(next);
              }}
            />
          </div>
        ))}
        <div className="muted">Weights are normalized on-chain to your vote power. Total slider {total}.</div>
      </div>

      <div className="card">
        <div className="row">
          <button disabled={!connected} onClick={vote}>
            Cast vote
          </button>
          <span className="muted">{status}</span>
        </div>
      </div>
    </div>
  );
}
