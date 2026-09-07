/** Anchor ix discriminator = sha256("global:<name>")[0..8] */
export async function anchorDisc(name: string): Promise<Uint8Array> {
  const enc = new TextEncoder().encode(`global:${name}`);
  const hash = new Uint8Array(await crypto.subtle.digest("SHA-256", enc));
  return hash.slice(0, 8);
}

export function encodeU64Vec(xs: number[]): Uint8Array {
  const out = new Uint8Array(4 + xs.length * 8);
  new DataView(out.buffer).setUint32(0, xs.length, true);
  xs.forEach((x, i) => {
    const v = BigInt(Math.max(0, Math.floor(x)));
    const view = new DataView(out.buffer, 4 + i * 8, 8);
    view.setUint32(0, Number(v & 0xffffffffn), true);
    view.setUint32(4, Number((v >> 32n) & 0xffffffffn), true);
  });
  return out;
}
