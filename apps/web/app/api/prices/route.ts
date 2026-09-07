import { NextResponse } from "next/server";

export const dynamic = "force-dynamic";

async function ondoPrimary(symbol: string): Promise<number | null> {
  const key = process.env.ONDO_API_KEY;
  if (!key) return null;
  const res = await fetch("https://api.gm.ondo.finance/v1/assets/all/prices/latest", {
    headers: { "x-api-key": key },
    cache: "no-store",
  });
  if (!res.ok) return null;
  const rows = (await res.json()) as { primaryMarket?: { symbol: string; price: string } }[];
  const hit = rows.find((r) => r.primaryMarket?.symbol === symbol);
  return hit?.primaryMarket ? parseFloat(hit.primaryMarket.price) : null;
}

async function yahoo(symbol: string): Promise<number | null> {
  const url = `https://query1.finance.yahoo.com/v8/finance/chart/${encodeURIComponent(symbol)}?interval=1m&range=1d`;
  const res = await fetch(url, {
    headers: { "User-Agent": "MandateProtocol/0.1" },
    cache: "no-store",
  });
  if (!res.ok) return null;
  const j = await res.json();
  const px = j?.chart?.result?.[0]?.meta?.regularMarketPrice;
  return typeof px === "number" ? px : null;
}

async function usdyNav(): Promise<number | null> {
  // USDY is a rising NAV token. Prefer Ondo app-style NAV if key present; else ~live proxy via Yahoo-less static band is forbidden.
  const ondo = await ondoPrimary("USDY");
  if (ondo) return ondo;
  const res = await fetch("https://api.coinbase.com/v2/prices/USDT-USD/spot", { cache: "no-store" });
  // last-resort: do not invent 1.14. Return null and let client show stale.
  if (!res.ok) return null;
  return null;
}

export async function GET() {
  const spyOndo = await ondoPrimary("SPYon");
  const spy = spyOndo ?? (await yahoo("SPY"));
  const usdy = (await ondoPrimary("USDY")) ?? (await usdyNav());
  return NextResponse.json({
    source: {
      spy: spyOndo ? "ondo-gm" : "yahoo-SPY",
      usdy: process.env.ONDO_API_KEY ? "ondo-gm-or-null" : "needs ONDO_API_KEY or pyth",
    },
    spyUsd: spy,
    usdyUsd: usdy,
    ts: Date.now(),
    note: "Devnet mints are priced with these live USD figures. Mainnet uses the same sources plus on-chain refresh_prices.",
  });
}
