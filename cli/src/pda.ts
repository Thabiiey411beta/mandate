import { PublicKey } from "@solana/web3.js";

export function vaultPda(programId: PublicKey, authority: PublicKey) {
  return PublicKey.findProgramAddressSync([Buffer.from("vault"), authority.toBuffer()], programId);
}

export function epochPda(programId: PublicKey, vault: PublicKey, index: bigint) {
  const buf = Buffer.alloc(8);
  buf.writeBigUInt64LE(index);
  return PublicKey.findProgramAddressSync(
    [Buffer.from("epoch"), vault.toBuffer(), buf],
    programId
  );
}

export function rebatePda(programId: PublicKey, vault: PublicKey, user: PublicKey) {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("rebate"), vault.toBuffer(), user.toBuffer()],
    programId
  );
}

export function positionPda(
  programId: PublicKey,
  vault: PublicKey,
  index: bigint,
  user: PublicKey
) {
  const buf = Buffer.alloc(8);
  buf.writeBigUInt64LE(index);
  return PublicKey.findProgramAddressSync(
    [Buffer.from("pos"), vault.toBuffer(), buf, user.toBuffer()],
    programId
  );
}
