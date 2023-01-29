use anchor_lang::prelude::*;
use rockpaperscissors::{
    GameState,
    process_action,
    Actions,
};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod rps {
    use rockpaperscissors::Actions;

    use super::*;

    pub fn initialize(ctx: Context<InitializeGame>) -> Result<()> {
        ctx.accounts.game.state = GameState::Initialized;
        Ok(())
    }

    pub fn make_action(ctx: Context<MakeAction>, action: Actions) -> Result<()> {
        let pubkey = ctx.accounts.game.key();
        let clock = &Clock::get()?;
        ctx.accounts.game.state = process_action(pubkey, ctx.accounts.game.state, action, clock.slot);
        Ok(())
    }

}


#[derive(Accounts)]
pub struct InitializeGame<'info> {
    #[account(init, payer = player, space = 10000)]
    pub game: Account<'info, Game>,
    #[account(mut)]
    pub player: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MakeAction<'info> {
    #[account(mut)]
    pub game: Account<'info, Game>,
}

#[account]
pub struct Game {
    pub state: GameState
}
