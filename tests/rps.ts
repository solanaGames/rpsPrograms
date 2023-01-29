import * as anchor from "@project-serum/anchor";
import { PublicKey, SystemProgram, Transaction, Connection, Commitment } from "@solana/web3.js";
import { Program } from "@project-serum/anchor";
import { Rps } from "../target/types/rps";
import { BN } from "bn.js";
import {
  TOKEN_PROGRAM_ID,
  createMint,
  createAccount,
  mintTo,
  getAccount,
  getAssociatedTokenAddress,
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@solana/spl-token";

describe("rps", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Rps as Program<Rps>;

  const LAMPORTS_PER_SOL = new BN(1000000000);
  const WRAPPED_SOL_MINT = new anchor.web3.PublicKey(
    "So11111111111111111111111111111111111111112"
  );

  it("Is initialized!", async () => {
    const player = anchor.web3.Keypair.generate();
    const game = anchor.web3.Keypair.generate();
    const provider = anchor.getProvider();

    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(
        player.publicKey,
        1000000000 * 10
      )
    );

    const mintAuthority = anchor.web3.Keypair.generate();
    const mint = await createMint(
      provider.connection,
      player,
      mintAuthority.publicKey,
      null,
      0
    );
    const tokenAccount = await createAccount(
      provider.connection,
      player,
      mint,
      player.publicKey
    );
    await mintTo(
      provider.connection,
      player,
      mint,
      tokenAccount,
      mintAuthority,
      10
    );

    // const acc = await getAccount(provider.connection, tokenAccount);
    // console.log("Token account ", acc);


    const [gameAuthority, _] = await PublicKey.findProgramAddress(
      [game.publicKey.toBuffer()],
      program.programId
    );
    const escrowTokenAccount = await getAssociatedTokenAddress(mint, gameAuthority, true);

    let commitment = [];
    for (let i = 0; i < 32; i++) {
      commitment.push(0);
    }

    const tx = await program.methods
      .createGame(commitment, new BN(10))
      .accounts({
        game: game.publicKey,
        player: player.publicKey,
        mint: mint,
        playerTokenAccount: tokenAccount,
        gameAuthority: gameAuthority,
        escrowTokenAccount: escrowTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .signers([game, player])
      .rpc();

    console.log("Your transaction signature", tx);

    const gameAccount = await program.account.game.fetch(game.publicKey);
    console.log("Your game account", gameAccount);
  });
});
