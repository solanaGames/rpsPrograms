import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Rps } from "../target/types/rps";

describe("rps", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Rps as Program<Rps>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
