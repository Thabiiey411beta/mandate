/** Pull live prices. If ONDO_API_KEY is set, use official GM primaryMarket. */
const SYMBOL = process.env.SYMBOL ?? "SPYon";

async function ondo() {
  const key = process.env.ONDO_API_KEY;
  if (!key) return null;
  const res = await fetch("https://api.gm.ondo.finance/v1/assets/all/prices/latest", {
    headers: { "x-api-key": key },
  });
  if (!res.ok) throw new Error(`ondo ${res.status}`);
  const rows = await res.json();
  const hit = rows.find((r) => r.primaryMarket?.symbol === SYMBOL);
  if (!hit) return null;
  return {
    source: "ondo-gm",
    symbol: SYMBOL,
    usd: parseFloat(hit.primaryMarket.price),
    e6: Math.round(parseFloat(hit.primaryMarket.price) * 1e6),
    underlying: hit.underlyingMarket,
    ts: hit.timestamp,
  };
}

async function yahooSpy() {
  const res = await fetch("https://query1.finance.yahoo.com/v8/finance/chart/SPY?interval=1m&range=1d", {
    headers: { "User-Agent": "MandateKeeper/0.1" },
  });
  if (!res.ok) throw new Error(`yahoo ${res.status}`);
  const j = await res.json();
  const usd = j.chart.result[0].meta.regularMarketPrice;
  return { source: "yahoo-SPY", symbol: "SPY", usd, e6: Math.round(usd * 1e6), ts: Date.now() };
}

const out = (await ondo()) ?? (await yahooSpy());
console.log(JSON.stringify(out, null, 2));
console.log("Pass e6 into refresh_prices once mandate-credit is deployed.");
