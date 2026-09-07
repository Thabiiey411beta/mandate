import { PublicKey } from "@solana/web3.js";

export const CREDIT_PROGRAM = new PublicKey(
  process.env.MANDATE_CREDIT_PROGRAM_ID ?? "Cred1T111111111111111111111111111111111111"
);

export const MINTS = {
  SPYon: "k18WJUULWheRkSpSquYGdNNmtuE2Vbw1hpuUi92ondo",
  USDY: "A1KLoBrKBde8Ty9qtNQUtq3C2ortoC3u7twggz7sEto6",
};

export function marketPda(collateral: PublicKey, debt: PublicKey, programId = CREDIT_PROGRAM) {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("market"), collateral.toBuffer(), debt.toBuffer()],
    programId
  );
}

export function oraclePda(market: PublicKey, programId = CREDIT_PROGRAM) {
  return PublicKey.findProgramAddressSync([Buffer.from("ondo_oracle"), market.toBuffer()], programId);
}

export async function fetchOndoPrimaryE6(symbol: string): Promise<{ e6: number; ts: number }> {
  const url =
    process.env.ONDO_PRICES_URL ?? "https://api.gm.ondo.finance/v1/assets/all/prices/latest";
  const headers: Record<string, string> = {};
  if (process.env.ONDO_API_KEY) headers["x-api-key"] = process.env.ONDO_API_KEY;
  const res = await fetch(url, { headers });
  if (!res.ok) throw new Error(`Ondo GM prices HTTP ${res.status}`);
  const rows = (await res.json()) as {
    primaryMarket?: { symbol: string; price: string };
    timestamp?: number;
  }[];
  const hit = rows.find((r) => r.primaryMarket?.symbol === symbol);
  if (!hit?.primaryMarket) throw new Error(`no Ondo primaryMarket price for ${symbol}`);
  const e6 = Math.round(parseFloat(hit.primaryMarket.price) * 1_000_000);
  return { e6, ts: hit.timestamp ?? Date.now() };
}
