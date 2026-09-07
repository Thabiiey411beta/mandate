"use client";

import { useMemo, useState } from "react";
import { useConnection, useWallet } from "@solana/wallet-adapter-react";
import { WalletMultiButton } from "@solana/wallet-adapter-react-ui";
import { PublicKey, Transaction, TransactionInstruction } from "@solana/web3.js";
import { anchorDisc, encodeU64Vec } from "../lib/anchor";

const PROGRAM_ID = new PublicKey(
  process.env.NEXT_PUBLIC_MANDATE_PROGRAM_ID ?? "MandatE11111111111111111111111111111111111"
);

const DEMO_BALLOT = [
  { label: "NVDAon" },
  { label: "TSLAon" },
  { label: "AAPLon" },
  { label: "MSFTon" },
  { label: "GOOGLon" },
  { label: "METAon" },
  { label: "AMZNon" },
];

export default function Page() {
  const { connection } = useConnection();
  const { publicKey, sendTransaction, connected } = useWallet();
  const [weights, setWeights] = useState(DEMO_BALLOT.map(() => 0));
  const [epochPk, setEpochPk] = useState("");
  const [status, setStatus] = useState("devnet UI — paste epoch PDA from `mandate open-epoch`");

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
    const vaultEnv = process.env.NEXT_PUBLIC_VAULT;
    if (!vaultEnv) {
      setStatus("Set NEXT_PUBLIC_VAULT in apps/web/.env.local");
      return;
    }
    const vault = new PublicKey(vaultEnv);
    const epoch = new PublicKey(epochPk);
    const idx = BigInt(process.env.NEXT_PUBLIC_EPOCH_INDEX ?? "1");
    const buf = Buffer.alloc(8);
    buf.writeBigUInt64LE(idx);
    const [position] = PublicKey.findProgramAddressSync(
      [Buffer.from("pos"), vault.toBuffer(), buf, publicKey.toBuffer()],
      PROGRAM_ID
    );

    const disc = await anchorDisc("cast_vote");
    const weightsEnc = encodeU64Vec(weights);
    const data = new Uint8Array(disc.length + weightsEnc.length);
    data.set(disc, 0);
    data.set(weightsEnc, disc.length);

    const ix = new TransactionInstruction({
      programId: PROGRAM_ID,
      keys: [
        { pubkey: publicKey, isSigner: true, isWritable: false },
        { pubkey: epoch, isSigner: false, isWritable: true },
        { pubkey: position, isSigner: false, isWritable: true },
      ],
      data: Buffer.from(data),
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
          <p className="muted">
            Devnet vote. Idle cash stays in USDY. ONDO stake rebates fees — it does not pick the stock.
          </p>
        </div>
        <WalletMultiButton />
      </div>

      <div className="card">
        <div className="muted">Epoch PDA</div>
        <input
          style={{
            width: "100%",
            marginTop: 8,
            padding: 10,
            borderRadius: 8,
            border: "1px solid #1d2430",
            background: "#0b0e14",
            color: "#e8edf5",
          }}
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
              <span className="muted">{weights[i]}</span>
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
        <div className="muted">On-chain weights normalize to your vote power. Slider sum {total}.</div>
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
