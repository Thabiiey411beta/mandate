import * as anchor from "@coral-xyz/anchor";

describe("mandate", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  it("workspace loads", async () => {
    const program = anchor.workspace.Mandate as anchor.Program;
    console.log("program id", program.programId.toBase58());
  });
});
