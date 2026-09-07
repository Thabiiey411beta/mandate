"use client";

import { useEffect, useMemo, useState } from "react";
import { useConnection, useWallet } from "@solana/wallet-adapter-react";
import { WalletMultiButton } from "@solana/wallet-adapter-react-ui";
import { PublicKey, Transaction, TransactionInstruction } from "@solana/web3.js";
import { anchorDisc, encodeU64Vec } from "../lib/anchor";

const DEMO_BALLOT = ["NVDAon", "TSLAon", "AAPLon", "MSFTon", "GOOGLon", "METAon", "AMZNon", "SPYon"];

export default function Page() {
  const { connection } = useConnection();
  const { publicKey, sendTransaction, connected } = useWallet();
  const [weights, setWeights] = useState(DEMO_BALLOT.map(() => 0));
  const [epochPk, setEpochPk] = useState("");
  const [status, setStatus] = useState("loading live prices…");
  const [prices, setPrices] = useState<{ spyUsd: number | null; usdyUsd: number | null; source: { spy: string } } | null>(null);
  const [cfg, setCfg] = useState<Record<string, unknown> | null>(null);

  useEffect(() => {
    fetch("/api/prices")
      .then((r) => r.json())
      .then(setPrices)
      .catch((e) => setStatus(String(e)));
    fetch("/api/config")
      .then((r) => r.json())
      .then(setCfg)
      .catch(() => null);
  }, []);

  const total = useMemo(() => weights.reduce((a, b) => a + b, 0), [weights]);
  const programIdStr = (cfg?.programMandate as string) || process.env.NEXT_PUBLIC_MANDATE_PROGRAM_ID;

  async function vote() {
    if (!publicKey || !programIdStr) {
      setStatus("Connect wallet and deploy mandate (set program id in devnet.json).");
      return;
    }
    if (!epochPk) {
      setStatus("Paste epoch PDA from CLI open-epoch.");
      return;
    }
    const vaultEnv = (cfg as { vault?: string })?.vault || process.env.NEXT_PUBLIC_VAULT;
    if (!vaultEnv) {
      setStatus("No vault in config. initialize_vault on devnet first.");
      return;
    }
    const PROGRAM_ID = new PublicKey(programIdStr);
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
          <p className="muted">Solana devnet. Prices below are live market data, not mocks.</p>
        </div>
        <WalletMultiButton />
      </div>

      <div className="card">
        <div className="row">
          <div>
            <div className="muted">SPY / SPYon USD</div>
            <strong className="ticker">{prices?.spyUsd ?? "—"}</strong>
            <div className="muted">{prices?.source?.spy}</div>
          </div>
          <div>
            <div className="muted">USDY USD</div>
            <strong className="ticker">{prices?.usdyUsd ?? "set ONDO_API_KEY"}</strong>
          </div>
        </div>
      </div>

      <div className="card">
        <div className="muted">Epoch PDA</div>
        <input
          style={{ width: "100%", marginTop: 8, padding: 10, borderRadius: 8, border: "1px solid #1d2430", background: "#0b0e14", color: "#e8edf5" }}
          placeholder="from: npx tsx cli/src/index.ts open-epoch"
          value={epochPk}
          onChange={(e) => setEpochPk(e.target.value)}
        />
      </div>

      <div className="card">
        {DEMO_BALLOT.map((t, i) => (
          <div key={t} style={{ marginBottom: 16 }}>
            <div className="row">
              <strong className="ticker">{t}</strong>
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
        <div className="muted">Slider sum {total}. On-chain weights scale to your USDY vote power.</div>
      </div>

      <div className="card">
        <div className="row">
          <button disabled={!connected} onClick={vote}>Cast vote</button>
          <span className="muted">{status}</span>
        </div>
      </div>
    </div>
  );
}
