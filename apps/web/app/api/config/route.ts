import { NextResponse } from "next/server";
import { readFileSync, existsSync } from "fs";
import { join } from "path";

export const dynamic = "force-dynamic";

export async function GET() {
  const p = join(process.cwd(), "devnet.json");
  if (!existsSync(p)) {
    return NextResponse.json({
      cluster: "devnet",
      rpc: process.env.NEXT_PUBLIC_RPC ?? "https://api.devnet.solana.com",
      vault: process.env.NEXT_PUBLIC_VAULT ?? null,
      programMandate: process.env.NEXT_PUBLIC_MANDATE_PROGRAM_ID ?? null,
      programCredit: process.env.NEXT_PUBLIC_CREDIT_PROGRAM_ID ?? null,
      usdyMint: process.env.NEXT_PUBLIC_USDY_MINT ?? "A1KLoBrKBde8Ty9qtNQUtq3C2ortoC3u7twggz7sEto6",
      spyOnMint: process.env.NEXT_PUBLIC_SPYON_MINT ?? "k18WJUULWheRkSpSquYGdNNmtuE2Vbw1hpuUi92ondo",
      ready: false,
      hint: "Run node scripts/devnet-bootstrap.mjs and copy config/devnet.json to apps/web/devnet.json",
    });
  }
  const cfg = JSON.parse(readFileSync(p, "utf8"));
  return NextResponse.json({ ...cfg, ready: true });
}
