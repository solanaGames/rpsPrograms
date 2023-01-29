use anchor_lang::prelude::*;
use rockpaperscissors::GameState;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod rps {
    use super::*;

    pub fn initialize(ctx: Context<CreateGame>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateGame<'info> {
    #[account(init)]
    pub game: Account<'info, Game>,
}

#[account]
pub struct Game {
    pub state: GameState
}
