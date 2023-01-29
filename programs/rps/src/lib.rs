use anchor_lang::prelude::*;
mod logic;
use anchor_spl::{
    associated_token::{get_associated_token_address, AssociatedToken},
    token::{Mint, Token, TokenAccount},
};
use spl_associated_token_account::instruction::create_associated_token_account;

use logic::{process_action, Actions, GameConfig, GameState};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod rps {
    use super::*;

    pub fn create_game(
        ctx: Context<CreateGame>,
        commitment: [u8; 32],
        wager_amount: u64,
    ) -> Result<()> {
        ctx.accounts.game.state = GameState::Initialized;

        let action = Actions::CreateGame {
            player_1_pubkey: ctx.accounts.player.key(),
            commitment,
            config: GameConfig {
                mint: ctx.accounts.mint.key(),
                wager_amount,
                entry_proof: None,
            },
        };

        ctx.accounts.game.state = process_action(
            ctx.accounts.game.key(),
            ctx.accounts.game.state,
            action,
            Clock::get()?.slot,
        );

        match ctx.accounts.game.state {
            GameState::AcceptingChallenge { .. } => {
                solana_program::program::invoke(
                    &create_associated_token_account(
                        &ctx.accounts.player.key(),
                        &ctx.accounts.game_authority.key(),
                        &ctx.accounts.mint.key(),
                        &ctx.accounts.token_program.key(),
                    ),
                    &[
                        ctx.accounts.player.to_account_info(),
                        ctx.accounts.escrow_token_account.to_account_info(),
                        ctx.accounts.game_authority.to_account_info(),
                        ctx.accounts.mint.to_account_info(),
                        ctx.accounts.system_program.to_account_info(),
                        ctx.accounts.token_program.to_account_info(),
                    ],
                )?;

                solana_program::program::invoke(
                    &spl_token::instruction::transfer(
                        &ctx.accounts.token_program.key(),
                        &ctx.accounts.player_token_account.key(),
                        &ctx.accounts.escrow_token_account.key(),
                        &ctx.accounts.player.key(),
                        &[],
                        wager_amount,
                    )?,
                    &[
                        ctx.accounts.token_program.to_account_info(),
                        ctx.accounts.player_token_account.to_account_info(),
                        ctx.accounts.escrow_token_account.to_account_info(),
                        ctx.accounts.player.to_account_info(),
                    ],
                )?;
            }
            _ => panic!("Invalid state"),
        };

        Ok(())
    }

    pub fn make_action(ctx: Context<MakeAction>, action: Actions) -> Result<()> {
        let pubkey = ctx.accounts.game.key();
        let clock = &Clock::get()?;
        ctx.accounts.game.state =
            process_action(pubkey, ctx.accounts.game.state, action, clock.slot);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateGame<'info> {
    #[account(init, payer = player, space = 10000)]
    pub game: Account<'info, Game>,

    #[account(mut)]
    pub player: Signer<'info>,

    pub mint: Account<'info, Mint>,

    #[account(mut)]
    pub player_token_account: Account<'info, TokenAccount>,

    /// CHECK: this is a pda that manages the escrow account
    #[account(mut, seeds = [game.key().as_ref()], bump)]
    pub game_authority: AccountInfo<'info>,

    /// CHECK: this is being create in this call
    #[account(mut, address = get_associated_token_address(&game_authority.key(), &mint.key()))]
    pub escrow_token_account: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct MakeAction<'info> {
    #[account(mut)]
    pub game: Account<'info, Game>,
}

#[account]
pub struct Game {
    pub state: GameState,
}
