/// This is an Solana program using the Anchor libarary.
use anchor_lang::prelude::*;
use num_derive::*;

declare_id!("H5dHHXFBR4VpzR5YREHHX4cf4LAPfiK1xWZKsrghs7gx");
// declare_id!("EJnMK54zAn9W3dFXj2B9n7PQXNHVqNDK8QPBADuoQ5cv");

#[program]
pub mod tic_tac_toe {
    use super::*;

    pub fn setup_game(ctx: Context<SetupGame>, player_two: Pubkey) -> ProgramResult {
        let game = &mut ctx.accounts.game;
        game.players = [ctx.accounts.player_one.key(), player_two];
        game.turn = 0;
        Ok(())
    }

    pub fn play(ctx: Context<Play>, tile: Tile) -> ProgramResult {
        let game = &mut ctx.accounts.game;
        game.play(ctx.accounts.player.key(), &tile)
    }
}

#[account]
#[derive(Default)]
pub struct Game {
    players: [Pubkey; 2],          // 64
    turn: u8,                      // 1
    board: [[Option<Sign>; 3]; 3], // 9 * (1 + 1) = 18
    state: GameState,              // 32 + 1
}

impl Game {
    const MAXIMUM_SIZE: usize = 116;

    pub fn current_player(&self) -> Pubkey {
        self.players[self.turn as usize]
    }

    pub fn play(&mut self, player: Pubkey, tile: &Tile) -> ProgramResult {
        let row = tile.row as usize;
        let col = tile.col as usize;

        require!(
            self.current_player() == player,
            TicTactoeError::NotPlayersTurn
        );

        require!(
            self.state == GameState::Active,
            TicTactoeError::GameAlreadyOver
        );

        require!(
            self.board[row][col].is_none(),
            TicTactoeError::TileAlreadySet
        );

        require!(
            row < 3 && col < 3,
            TicTactoeError::TileOutOfBounds
        );


        let sign = self.players_sign();
        self.board[row][col] = Some(sign);

        // check for win or tie
        if self.is_win_condition(sign) {
            self.state = GameState::Won {
                winner: self.current_player(),
            };
            return Ok(());
        }

        // if we're here it means it's not a win condition
        // but if the board is full, it's a tie
        if self.is_tie_condition() {
            self.state = GameState::Tie;
            return Ok(());
        }

        self.switch_turn();
        Ok(())
    }

    pub fn switch_turn(&mut self) {
        self.turn = (self.turn + 1) % 2;
    }

    pub fn players_sign(&self) -> Sign {
        if self.turn == 0 {
            Sign::X
        } else {
            Sign::O
        }
    }

    pub fn is_win_condition(&self, sign: Sign) -> bool {
        // row victory
        if self.board.iter().any(|row| row.iter().all(|tile| tile.is_some() && tile.unwrap() == sign)) {
            return true;
        }

        // column victory
        for c in 0..3 {
            if self.board[0][c].is_some() && self.board[0][c].unwrap() == sign &&
                self.board[1][c].is_some() && self.board[1][c].unwrap() == sign &&
                self.board[2][c].is_some() && self.board[2][c].unwrap() == sign {
                return true;
            }
        }

        // diagonal victory 1
        if self.board[0][0].is_some() && self.board[0][0].unwrap() == sign &&
            self.board[1][1].is_some() && self.board[1][1].unwrap() == sign &&
            self.board[2][2].is_some() && self.board[2][2].unwrap() == sign {
            return true;
        }

        // diagonal victory 1
        if self.board[0][2].is_some() && self.board[0][2].unwrap() == sign &&
            self.board[1][1].is_some() && self.board[1][1].unwrap() == sign &&
            self.board[2][0].is_some() && self.board[2][0].unwrap() == sign {
            return true;
        }

        return false;
    }

    pub fn is_tie_condition(&self) -> bool {
        self.board.iter().all(|row| row.iter().all(|tile| tile.is_some()))
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum GameState {
    Active,
    Tie,
    Won { winner: Pubkey },
}

impl Default for GameState {
    fn default() -> Self {
        GameState::Active
    }
}


#[derive(AnchorSerialize, AnchorDeserialize, FromPrimitive, ToPrimitive, Copy, Clone, PartialEq, Eq, Debug)]
pub enum Sign {
    X,
    O,
}

#[derive(Accounts)]
pub struct SetupGame<'info> {
    // +8 is for Anchor's discriminator
    #[account(init, payer = player_one, space = Game::MAXIMUM_SIZE + 8)]
    pub game: Account<'info, Game>,
    #[account(mut)]
    pub player_one: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Play<'info> {
    #[account(mut)]
    pub game: Account<'info, Game>,
    pub player: Signer<'info>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct Tile {
    row: u8,
    col: u8,
}

#[error]
pub enum TicTactoeError {
    TileOutOfBounds,
    TileAlreadySet,
    GameAlreadyOver,
    NotPlayersTurn,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn play_a_game() {
        let pk1 = Pubkey::new_unique();
        let pk2 = Pubkey::new_unique();
        let mut g = Game {
            players: [pk1, pk2],
            turn: 0,
            board: [[None; 3]; 3],
            state: GameState::Active,
        };

        assert_eq!(g.current_player(), pk1);
        assert_eq!(g.players_sign(), Sign::X);

        g.play(pk1, &Tile { row: 0, col: 0 }).unwrap();
        let board = [[Some(Sign::X), None, None],
                    [None, None, None],
                    [None, None, None]];
        assert_eq!(g.board, board);

        // wrong player's turn
        let res = g.play(pk1, &Tile { row: 1, col: 1 });
        assert!(res.is_err());

        g.play(pk2, &Tile { row: 1, col: 1 }).unwrap();
        let board = [[Some(Sign::X), None, None],
                    [None, Some(Sign::O), None],
                    [None, None, None]];
        assert_eq!(g.board, board);

        g.play(pk1, &Tile { row: 0, col: 1 }).unwrap();
        let board = [[Some(Sign::X), Some(Sign::X), None],
                    [None, Some(Sign::O), None],
                    [None, None, None]];
        assert_eq!(g.board, board);

        g.play(pk2, &Tile { row: 2, col: 2 }).unwrap();
        let board = [[Some(Sign::X), Some(Sign::X), None],
                    [None, Some(Sign::O), None],
                    [None, None, Some(Sign::O)]];
        assert_eq!(g.board, board);

        g.play(pk1, &Tile { row: 0, col: 2 }).unwrap();
        let board = [[Some(Sign::X), Some(Sign::X), Some(Sign::X)],
                    [None, Some(Sign::O), None],
                    [None, None, Some(Sign::O)]];
        assert_eq!(g.board, board);

        // game is over, player 1 won
        let res = g.play(pk2, &Tile { row: 1, col: 2 });
        assert!(res.is_err());

        assert_eq!(g.state, GameState::Won { winner: pk1 });
    }

    #[test]
    fn win_condition_rows() {
        let pk1 = Pubkey::new_unique();
        let pk2 = Pubkey::new_unique();
        let mut g = Game {
            players: [pk1, pk2],
            turn: 0,
            board: [[None; 3]; 3],
            state: GameState::Active,
        };
        g.board = [
            [Some(Sign::X), Some(Sign::X), Some(Sign::X)],
            [Some(Sign::O), Some(Sign::O), None],
            [None, None, None],
        ];
        assert!(g.is_win_condition(Sign::X));

        g.board = [
            [Some(Sign::O), Some(Sign::O), None],
            [Some(Sign::X), Some(Sign::X), Some(Sign::X)],
            [None, None, None],
        ];
        assert!(g.is_win_condition(Sign::X));

        g.board = [
            [Some(Sign::O), Some(Sign::O), None],
            [None, None, None],
            [Some(Sign::X), Some(Sign::X), Some(Sign::X)],
        ];
        assert!(g.is_win_condition(Sign::X));
    }

    #[test]
    fn win_condition_columns() {
        let pk1 = Pubkey::new_unique();
        let pk2 = Pubkey::new_unique();
        let mut g = Game {
            players: [pk1, pk2],
            turn: 0,
            board: [[None; 3]; 3],
            state: GameState::Active,
        };
        g.board = [
            [Some(Sign::X), Some(Sign::O), Some(Sign::O)],
            [Some(Sign::X), Some(Sign::O), None],
            [Some(Sign::X), None, None],
        ];
        assert!(g.is_win_condition(Sign::X));

        g.board = [
            [Some(Sign::X), Some(Sign::O), Some(Sign::O)],
            [Some(Sign::X), Some(Sign::O), None],
            [None, Some(Sign::O), None],
        ];
        assert!(g.is_win_condition(Sign::O));

        g.board = [
            [Some(Sign::X), Some(Sign::O), Some(Sign::O)],
            [Some(Sign::X), None,          Some(Sign::O)],
            [None,          None,          Some(Sign::O)],
        ];
        assert!(g.is_win_condition(Sign::O));
    }

    #[test]
    fn win_condition_diagonals() {
        let pk1 = Pubkey::new_unique();
        let pk2 = Pubkey::new_unique();
        let mut g = Game {
            players: [pk1, pk2],
            turn: 0,
            board: [[None; 3]; 3],
            state: GameState::Active,
        };
        g.board = [
            [Some(Sign::X), Some(Sign::X), Some(Sign::O)],
            [Some(Sign::O), Some(Sign::X), None],
            [None, None, Some(Sign::X)],
        ];
        assert!(g.is_win_condition(Sign::X));

        g.board = [
            [None, Some(Sign::X), Some(Sign::O)],
            [None, Some(Sign::O), None],
            [Some(Sign::O), None, Some(Sign::X)],
        ];
        assert!(g.is_win_condition(Sign::O));
    }

    #[test]
    fn tie_condition() {
        let pk1 = Pubkey::new_unique();
        let pk2 = Pubkey::new_unique();
        let mut g = Game {
            players: [pk1, pk2],
            turn: 0,
            board: [[None; 3]; 3],
            state: GameState::Active,
        };
        g.board = [
            [Some(Sign::X), Some(Sign::X), Some(Sign::O)],
            [Some(Sign::O), Some(Sign::X), Some(Sign::X)],
            [Some(Sign::X), Some(Sign::O), Some(Sign::O)],
        ];
        assert!(g.is_tie_condition());
    }
}
