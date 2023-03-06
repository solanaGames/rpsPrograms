import * as anchor from "@coral-xyz/anchor";
import {
  PublicKey,
  SystemProgram,
  Transaction,
  Connection,
  Commitment,
} from "@solana/web3.js";
import { Rps, IDL } from "../target/types/rps";
import { BN } from "bn.js";
import { keccak_256 } from "js-sha3";
import { expect, should } from "chai";

describe("rps", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.getProvider();
  // const program = anchor.workspace.Rps as anchor.Program<Rps>;
  const program = new anchor.Program(
    IDL,
    new PublicKey("rpsVN2ZC1K9hoGPs83xahjWo46cDNP49Tk7rQb56ipE"),
    provider
  );

  const LAMPORTS_PER_SOL = new BN(1000000000);
  const WRAPPED_SOL_MINT = new anchor.web3.PublicKey(
    "So11111111111111111111111111111111111111112"
  );

  it("Is initialized!", async () => {
    const player = anchor.web3.Keypair.generate();
    const gameSeed = new BN(10);
    const wagerAmount = new BN(1000000);
    const [game, _gameBump] = PublicKey.findProgramAddressSync(
      [
        Buffer.from(anchor.utils.bytes.utf8.encode("game")),
        gameSeed.toArrayLike(Buffer, "le", 8),
      ],
      program.programId
    );
    const salt = Math.floor(Math.random() * 10000000);
    const player1Choice = 1;

    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(
        player.publicKey,
        1000000000 * 10
      )
    );

    const [playerInfo, _playerInfoBump] = PublicKey.findProgramAddressSync(
      [
        Buffer.from(anchor.utils.bytes.utf8.encode("player_info")),
        player.publicKey.toBuffer(),
      ],
      program.programId
    );
    const tx0 = await program.methods
      .createPlayerInfo()
      .accounts({
        playerInfo,
        systemProgram: anchor.web3.SystemProgram.programId,
        owner: player.publicKey,
      })
      .signers([player])
      .rpc({ skipPreflight: true });
    console.log("Your transaction signature 0", tx0);

    const [gameAuthority, _gameAuthorityBump] =
      PublicKey.findProgramAddressSync(
        [
          Buffer.from(anchor.utils.bytes.utf8.encode("authority")),
          game.toBuffer(),
        ],
        program.programId
      );

    const buf = Buffer.concat([
      player.publicKey.toBuffer(),
      new anchor.BN(salt).toArrayLike(Buffer, "le", 8),
      new anchor.BN(player1Choice).toArrayLike(Buffer, "le", 1),
    ]);
    let commitment = Buffer.from(keccak_256(buf), "hex");

    const tx = await program.methods
      .createGame(gameSeed, commitment.toJSON().data, wagerAmount, null)
      .accounts({
        game: game,
        player: player.publicKey,
        playerInfo: playerInfo,
        gameAuthority: gameAuthority,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([player])
      .rpc({ skipPreflight: true });

    console.log("Your transaction signature", tx);
    const player1InfoAccountGameCreated =
      await program.account.playerInfo.fetch(playerInfo);
    expect(player1InfoAccountGameCreated.amountInGames.toString()).eq(
      "1000000"
    );

    // const gameAccount = await program.account.game.fetch(game);
    // console.log("Your game account", gameAccount);

    const player2 = anchor.web3.Keypair.generate();
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(
        player2.publicKey,
        1000000000 * 10
      )
    );
    const [player2Info, _playerInfo2Bump] = PublicKey.findProgramAddressSync(
      [
        Buffer.from(anchor.utils.bytes.utf8.encode("player_info")),
        player2.publicKey.toBuffer(),
      ],
      program.programId
    );
    const txCP2 = await program.methods
      .createPlayerInfo()
      .accounts({
        playerInfo: player2Info,
        systemProgram: anchor.web3.SystemProgram.programId,
        owner: player2.publicKey,
      })
      .signers([player2])
      .rpc({ skipPreflight: true });
    console.log("Your transaction signature cp2", txCP2);

    const tx2 = await program.methods
      .joinGame({ rock: {} }, null)
      .accounts({
        player: player2.publicKey,
        playerInfo: player2Info,
        game: game,
        gameAuthority: gameAuthority,
      })
      .signers([player2])
      .rpc({ skipPreflight: true });
    console.log("Your transaction signature", tx);

    // const gameAccount2 = await program.account.game.fetch(game);
    // console.log("Your game account", gameAccount2);

    const tx3 = await program.methods
      .revealGame({ paper: {} }, new BN(salt))
      .accounts({
        player: player.publicKey,
        playerInfo,
        game: game,
      })
      .signers([player])
      .rpc({ skipPreflight: true });

    const gameAccount3 = await program.account.game.fetch(game);
    console.log("Your game account", gameAccount3);

    const tx4 = await program.methods
      .settleGame()
      .accounts({
        game: game,
        player1: player.publicKey,
        player2: player2.publicKey,
        player1Info: playerInfo,
        player2Info: player2Info,
        gameAuthority: gameAuthority,
      })
      .rpc({ skipPreflight: true });

    const gameAccount4 = await program.account.game.fetch(game);
    console.log("Your game account", JSON.stringify(gameAccount4));

    const tx5 = await program.methods
      .cleanGame()
      .accounts({
        game: game,
        gameAuthority: gameAuthority,
        systemProgram: anchor.web3.SystemProgram.programId,
        player1: player.publicKey,
        rpsProgram: program.programId,
      })
      .rpc({ skipPreflight: true });
  });
});
