use anchor_lang::prelude::*;
mod logic;
use anchor_spl::{
    associated_token::{get_associated_token_address, AssociatedToken},
    token::{Mint, Token, TokenAccount},
};
use spl_associated_token_account::instruction::create_associated_token_account;

use logic::{process_action, Actions, GameConfig, GameState, Winner, RPS};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod rps {
    use super::*;
    use crate::logic::RPS;

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

    pub fn join_game(ctx: Context<JoinGame>, choice: RPS, secret: Option<u64>) -> Result<()> {
        let action = Actions::JoinGame {
            player_2_pubkey: ctx.accounts.player.key(),
            choice,
            secret,
        };

        ctx.accounts.game.state = process_action(
            ctx.accounts.game.key(),
            ctx.accounts.game.state,
            action,
            Clock::get()?.slot,
        );

        match ctx.accounts.game.state {
            GameState::AcceptingReveal {
                player_1,
                player_2,
                config,
                expiry_slot,
            } => {
                // transfer tokens from player_token_account to escrow_token_account
                solana_program::program::invoke(
                    &spl_token::instruction::transfer(
                        &ctx.accounts.token_program.key(),
                        &ctx.accounts.player_token_account.key(),
                        &ctx.accounts.escrow_token_account.key(),
                        &ctx.accounts.player.key(),
                        &[],
                        config.wager_amount,
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

    pub fn reveal_game(ctx: Context<RevealGame>, choice: RPS, salt: u64) -> Result<()> {
        let action = Actions::Reveal {
            player_1_pubkey: ctx.accounts.player.key(),
            choice,
            salt,
        };

        ctx.accounts.game.state = process_action(
            ctx.accounts.game.key(),
            ctx.accounts.game.state,
            action,
            Clock::get()?.slot,
        );

        Ok(())
    }

    pub fn expire_game(ctx: Context<ExpireGame>) -> Result<()> {
        let action = Actions::ExpireGame {
            player_pubkey: ctx.accounts.player.key(),
        };
        ctx.accounts.game.state = process_action(
            ctx.accounts.game.key(),
            ctx.accounts.game.state,
            action,
            Clock::get()?.slot,
        );
        Ok(())
    }

    pub fn settle_game(ctx: Context<SettleGame>) -> Result<()> {
        let action = Actions::Settle;
        ctx.accounts.game.state = process_action(
            ctx.accounts.game.key(),
            ctx.accounts.game.state,
            action,
            Clock::get()?.slot,
        );

        match ctx.accounts.game.state {
            GameState::Settled {
                result,
                player_1,
                player_2,
                config,
            } => match result {
                Winner::P1 => {
                    solana_program::program::invoke_signed(
                        &spl_token::instruction::transfer(
                            &ctx.accounts.token_program.key(),
                            &ctx.accounts.escrow_token_account.key(),
                            &ctx.accounts.player1_token_account.key(),
                            &ctx.accounts.game_authority.key(),
                            &[],
                            config.wager_amount * 2,
                        )?,
                        &[
                            ctx.accounts.token_program.to_account_info(),
                            ctx.accounts.escrow_token_account.to_account_info(),
                            ctx.accounts.player1_token_account.to_account_info(),
                            ctx.accounts.game_authority.to_account_info(),
                        ],
                        &[&[
                            ctx.accounts.game.key().as_ref(),
                            &[*ctx.bumps.get("game_authority").unwrap()],
                        ]],
                    )?;
                }
                Winner::P2 => {
                    solana_program::program::invoke_signed(
                        &spl_token::instruction::transfer(
                            &ctx.accounts.token_program.key(),
                            &ctx.accounts.escrow_token_account.key(),
                            &ctx.accounts.player2_token_account.key(),
                            &ctx.accounts.game_authority.key(),
                            &[],
                            config.wager_amount * 2,
                        )?,
                        &[
                            ctx.accounts.token_program.to_account_info(),
                            ctx.accounts.escrow_token_account.to_account_info(),
                            ctx.accounts.player2_token_account.to_account_info(),
                            ctx.accounts.game_authority.to_account_info(),
                        ],
                        &[&[
                            ctx.accounts.game.key().as_ref(),
                            &[*ctx.bumps.get("game_authority").unwrap()],
                        ]],
                    )?;
                }
                Winner::TIE => {
                    solana_program::program::invoke_signed(
                        &spl_token::instruction::transfer(
                            &ctx.accounts.token_program.key(),
                            &ctx.accounts.escrow_token_account.key(),
                            &ctx.accounts.player1_token_account.key(),
                            &ctx.accounts.game_authority.key(),
                            &[],
                            config.wager_amount,
                        )?,
                        &[
                            ctx.accounts.token_program.to_account_info(),
                            ctx.accounts.escrow_token_account.to_account_info(),
                            ctx.accounts.player1_token_account.to_account_info(),
                            ctx.accounts.game_authority.to_account_info(),
                        ],
                        &[&[
                            ctx.accounts.game.key().as_ref(),
                            &[*ctx.bumps.get("game_authority").unwrap()],
                        ]],
                    )?;
                    solana_program::program::invoke_signed(
                        &spl_token::instruction::transfer(
                            &ctx.accounts.token_program.key(),
                            &ctx.accounts.escrow_token_account.key(),
                            &ctx.accounts.player2_token_account.key(),
                            &ctx.accounts.game_authority.key(),
                            &[],
                            config.wager_amount,
                        )?,
                        &[
                            ctx.accounts.token_program.to_account_info(),
                            ctx.accounts.escrow_token_account.to_account_info(),
                            ctx.accounts.player2_token_account.to_account_info(),
                            ctx.accounts.game_authority.to_account_info(),
                        ],
                        &[&[
                            ctx.accounts.game.key().as_ref(),
                            &[*ctx.bumps.get("game_authority").unwrap()],
                        ]],
                    )?;
                }
            },
            _ => panic!("Invalid state"),
        };

        Ok(())
    }

    pub fn clean_game(ctx: Context<CleanGame>) -> Result<()> {
        // check only designated closer can close
        match ctx.accounts.game.state {
            GameState::Settled { result: _, player_1: _, player_2: _, config: _} => {
                solana_program::program::invoke_signed(
                    &spl_token::instruction::close_account(
                        &ctx.accounts.token_program.key(), 
                        &ctx.accounts.escrow_token_account.key(),
                        &ctx.accounts.cleaner.key(),
                        &ctx.accounts.game_authority.key(),
                        &[]
                    )?,
                    &[
                        ctx.accounts.token_program.to_account_info(), 
                        ctx.accounts.escrow_token_account.to_account_info(),
                        ctx.accounts.cleaner.to_account_info(),
                        ctx.accounts.game_authority.to_account_info(),
                    ],
                    &[&[
                        ctx.accounts.game.key().as_ref(),
                        &[*ctx.bumps.get("game_authority").unwrap()],
                    ]],
                )?;
            }
            _ => {panic!("game not settled can't clean")}
        }
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
pub struct JoinGame<'info> {
    #[account(mut)]
    player: Signer<'info>,

    #[account(mut)]
    pub game: Account<'info, Game>,

    pub mint: Account<'info, Mint>,

    #[account(mut)]
    pub player_token_account: Account<'info, TokenAccount>,

    /// CHECK: this is a pda that manages the escrow account
    #[account(mut, seeds = [game.key().as_ref()], bump)]
    pub game_authority: AccountInfo<'info>,

    #[account(mut, address = get_associated_token_address(&game_authority.key(), &mint.key()))]
    pub escrow_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct RevealGame<'info> {
    #[account(mut)]
    pub game: Account<'info, Game>,

    #[account(mut)]
    pub player: Signer<'info>,
}

#[derive(Accounts)]
pub struct ExpireGame<'info> {
    #[account(mut)]
    pub game: Account<'info, Game>,

    #[account(mut)]
    pub player: Signer<'info>,
}

#[derive(Accounts)]
pub struct SettleGame<'info> {
    #[account(mut)]
    pub game: Account<'info, Game>,

    // i dont think you actually need this you can just do game.state.mint in the ata check below in fact that is prob better
    pub mint: Account<'info, Mint>,

    #[account(mut)]
    pub player1_token_account: Account<'info, TokenAccount>,
    pub player2_token_account: Account<'info, TokenAccount>,

    /// CHECK: this is a pda that manages the escrow account
    #[account(mut, seeds = [game.key().as_ref()], bump)]
    pub game_authority: AccountInfo<'info>,

    /// CHECK: this is being create in this call
    #[account(mut, address = get_associated_token_address(&game_authority.key(), &mint.key()))]
    pub escrow_token_account: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct CleanGame<'info> {
    #[account(mut, close = cleaner)]
    pub game: Account<'info, Game>,

    // i dont think you actually need this you can just do game.state.mint in the ata check below in fact that is prob better
    pub mint: Account<'info, Mint>,

    /// CHECK: this is a pda that manages the escrow account
    #[account(seeds = [game.key().as_ref()], bump)]
    pub game_authority: AccountInfo<'info>,

    #[account(mut, address = get_associated_token_address(&game_authority.key(), &mint.key()), close = cleaner)]
    pub escrow_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub cleaner: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

#[account]
pub struct Game {
    pub state: GameState,
}
