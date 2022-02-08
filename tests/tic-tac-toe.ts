import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { TicTacToe } from '../target/types/tic_tac_toe';
import { expect } from 'chai';


async function play(program, game, player,
    tile, expectedTurn, expectedGameState, expectedBoard) {
  await program.rpc.play(tile, {
    accounts: {
      player: player.publicKey,
      game
    },
    signers: player instanceof (anchor.Wallet as any) ? [] : [player]
  });


  const gameState = await program.account.game.fetch(game);
  expect(gameState.turn).to.equal(expectedTurn);
  expect(gameState.state).to.eql(expectedGameState);
  expect(gameState.board)
    .to
    .eql(expectedBoard);
}

describe('tic-tac-toe', () => {

  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  const program = anchor.workspace.TicTacToe as Program<TicTacToe>;

  it('setup game!', async () => {
    const gameKeypair = anchor.web3.Keypair.generate();
    const playerOne = program.provider.wallet;
    const playerTwo = anchor.web3.Keypair.generate();

    await program.rpc.setupGame(playerTwo.publicKey, {
      accounts: {
        game: gameKeypair.publicKey,
        playerOne: playerOne.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId
      },
      signers: [gameKeypair]
    });

    let gameState = await program.account.game.fetch(gameKeypair.publicKey);

    expect(gameState.turn).to.eql(0);
    expect(gameState.players)
      .to
      .eql([playerOne.publicKey, playerTwo.publicKey]);
    expect(gameState.state).to.eql({ active: {} });
    expect(gameState.board).to.eql([
      [null, null, null],
      [null, null, null],
      [null, null, null],
    ]);
  });

  it('player one wins', async() => {
      const gameKeypair = anchor.web3.Keypair.generate();
      const playerOne = program.provider.wallet;
      const playerTwo = anchor.web3.Keypair.generate();
      await program.rpc.setupGame(playerTwo.publicKey, {
        accounts: {
          game: gameKeypair.publicKey,
          playerOne: playerOne.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId
        },
        signers: [gameKeypair]
      });

      let gameState = await program.account.game.fetch(gameKeypair.publicKey);
      expect(gameState.turn).to.equal(0);
      expect(gameState.players)
        .to
        .eql([playerOne.publicKey, playerTwo.publicKey]);
      expect(gameState.state).to.eql({ active: {} });
      expect(gameState.board)
        .to
        .eql([[null,null,null],[null,null,null],[null,null,null]]);

      await play(
        program,
        gameKeypair.publicKey,
        playerOne,
        {row: 0, col: 0},
        1,
        { active: {}, },
        [
          [{x:{}},null,null],
          [null,null,null],
          [null,null,null]
        ]
      );

      await play(
        program,
        gameKeypair.publicKey,
        playerTwo,
        {row: 1, col: 1},
        0,
        { active: {}, },
        [
          [{x:{}},null,null],
          [null,{o:{}},null],
          [null,null,null]
        ]
      );

      await play(
        program,
        gameKeypair.publicKey,
        playerOne,
        {row: 0, col: 2},
        1,
        { active: {}, },
        [
          [{x:{}},null,{x:{}}],
          [null,{o:{}},null],
          [null,null,null]
        ]
      );

      await play(
        program,
        gameKeypair.publicKey,
        playerTwo,
        {row: 1, col: 2},
        0,
        { active: {}, },
        [
          [{x:{}},null,{x:{}}],
          [null,{o:{}},{o:{}}],
          [null,null,null]
        ]
      );

      await play(
        program,
        gameKeypair.publicKey,
        playerOne,
        {row: 0, col: 1},
        0, // win
        { won: { winner: { playerOne } }},
        [
          [{x:{}},{x:{}},{x:{}}],
          [null,{o:{}},{o:{}}],
          [null,null,null]
        ]
      );
  });
});
