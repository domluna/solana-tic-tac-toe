/// This is an Solana program using the Anchor libarary.
use anchor_lang::prelude::*;
use num_derive::*;
use num_traits::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");
// declare_id!("H5dHHXFBR4VpzR5YREHHX4cf4LAPfiK1xWZKsrghs7gx");

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

        require!(
            game.current_player() == ctx.accounts.player.key(),
            TicTactoeError::NotPlayersTurn
        );

        game.play(&tile)
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

    pub fn play(&mut self, tile: &Tile) -> ProgramResult {
        let row = tile.row as usize;
        let col = tile.col as usize;

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
        self.turn = self.turn + 1 % 2;
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
        for r in  0..2 {
            if self.board[r][0].is_some() && self.board[r][0].unwrap() == sign &&
                self.board[r][1].is_some() && self.board[r][1].unwrap() == sign &&
                self.board[r][2].is_some() && self.board[r][2].unwrap() == sign {
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

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
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


#[derive(AnchorSerialize, AnchorDeserialize, FromPrimitive, ToPrimitive, Copy, Clone, PartialEq, Eq)]
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
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
