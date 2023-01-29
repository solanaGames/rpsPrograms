import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Rps } from "../target/types/rps";
import { BN } from "bn.js";

describe("rps", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Rps as Program<Rps>;

  it("Is initialized!", async () => {
    const player = anchor.web3.Keypair.generate();
    const game = anchor.web3.Keypair.generate();
    const provider = anchor.getProvider();

    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(player.publicKey, 1000000000)
    );

    const tx = await program.methods
      .initialize()
      .accounts({
        player: player.publicKey,
        game: game.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([game, player])
      .rpc();

    console.log("Your transaction signature", tx);

    const gameAccount = await program.account.game.fetch(game.publicKey);
    console.log("Your game account", gameAccount);

    let commitment = [];
    for (let i = 0; i < 32; i++) {
      commitment.push(0);
    }

    const mint = anchor.web3.Keypair.generate();

    const createGameTx = await program.methods
      .makeAction({ 
        createGame: { 
          player_1_pubkey: player.publicKey, 
          commitment: commitment, 
          config: { 
            wagerAmount: new BN(3), 
            mint: mint.publicKey,
            entryProof: null
          } 
      }})
      .accounts({
        game: game.publicKey,
      })
      .rpc();

    const gameAccount2 = await program.account.game.fetch(game.publicKey);
    console.log("Your game account", JSON.stringify(gameAccount2));
  });
});
