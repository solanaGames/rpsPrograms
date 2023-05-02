#![allow(clippy::result_large_err)]
use anchor_lang::prelude::*;
pub mod logic;

use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};
use logic::{process_action, Actions, GameConfig, GameState, Winner, RPS};
use program::RpsToken;
use serde::{Deserialize, Serialize};

declare_id!("rpsVN2ZC1K9hoGPs83xahjWo46cDNP49Tk7rQb56ipE");

pub mod game_cleaner {
    solana_program::declare_id!("rcxrvoLEpa65dqqRVEqAADicpNnhWBwtWtzyFHZMzKW");
}
pub mod local_bpf_loader {
    solana_program::declare_id!("BPFLoader2111111111111111111111111111111111");
}

static PLAYER_1_FEE_BPS: u64 = 350;

#[program]
pub mod rps_token {
    use super::*;
    use crate::logic::RPS;

    pub fn create_player_info(ctx: Context<CreatePlayerInfo>) -> Result<()> {
        ctx.accounts.player_info.owner = ctx.accounts.owner.key();
        ctx.accounts.player_info.mint = ctx.accounts.mint.key();

        Ok(())
    }

    pub fn create_game(
        ctx: Context<CreateGame>,
        game_seed: u64,
        commitment: [u8; 32],
        wager_amount: u64,
        entry_proof: Option<[u8; 32]>,
    ) -> Result<()> {
        ctx.accounts.game.state = GameState::Initialized;

        let action = Actions::CreateGame {
            player_1_pubkey: ctx.accounts.player.key(),
            commitment,
            config: GameConfig { entry_proof },
        };

        ctx.accounts.game.seed = game_seed;
        ctx.accounts.game.mint = ctx.accounts.mint.key();
        ctx.accounts.game.wager_amount = wager_amount;
        ctx.accounts.game.fee_amount = wager_amount
            .checked_mul(PLAYER_1_FEE_BPS)
            .ok_or(RpsError::BetTooLarge)?
            .checked_div(10000)
            .ok_or(RpsError::BetTooLarge)?;
        ctx.accounts.game.state = process_action(
            ctx.accounts.game.key(),
            ctx.accounts.game.state,
            action,
            Clock::get()?.slot,
        );

        match ctx.accounts.game.state {
            GameState::AcceptingChallenge { .. } => {
                anchor_spl::token::transfer(
                    CpiContext::new(
                        ctx.accounts.token_program.to_account_info(),
                        anchor_spl::token::Transfer {
                            from: ctx.accounts.player_token_account.to_account_info(),
                            to: ctx.accounts.escrow_token_account.to_account_info(),
                            authority: ctx.accounts.player.to_account_info(),
                        },
                    ),
                    wager_amount + ctx.accounts.game.fee_amount,
                )?;
            }
            _ => panic!("Invalid state"),
        };

        ctx.accounts.player_info.amount_in_games = ctx
            .accounts
            .player_info
            .amount_in_games
            .checked_add(ctx.accounts.game.wager_amount)
            .ok_or(RpsError::BetTooLarge)?;

        ctx.accounts.player_info.lifetime_wagering = ctx
            .accounts
            .player_info
            .lifetime_wagering
            .checked_add(ctx.accounts.game.wager_amount)
            .ok_or(RpsError::BetTooLarge)?;

        emit!(GameStartEvent {
            game_pubkey: ctx.accounts.game.key(),
            wager_amount,
            fee_amount: ctx.accounts.game.fee_amount,
            public: entry_proof.is_some(),
        });
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
            GameState::AcceptingReveal { .. } => {
                anchor_spl::token::transfer(
                    CpiContext::new(
                        ctx.accounts.token_program.to_account_info(),
                        anchor_spl::token::Transfer {
                            from: ctx.accounts.player_token_account.to_account_info(),
                            to: ctx.accounts.escrow_token_account.to_account_info(),
                            authority: ctx.accounts.player.to_account_info(),
                        },
                    ),
                    ctx.accounts.game.wager_amount,
                )?;
            }
            _ => panic!("Invalid state"),
        };

        ctx.accounts.player_info.amount_in_games = ctx
            .accounts
            .player_info
            .amount_in_games
            .checked_add(ctx.accounts.game.wager_amount)
            .ok_or(RpsError::BetTooLarge)?;
        ctx.accounts.player_info.lifetime_wagering = ctx
            .accounts
            .player_info
            .lifetime_wagering
            .checked_add(ctx.accounts.game.wager_amount)
            .ok_or(RpsError::BetTooLarge)?;

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

        let is_p1_expired = ctx.accounts.player_1.key() == ctx.accounts.player_2.key();

        ctx.accounts.player_1_info.amount_in_games = ctx
            .accounts
            .player_1_info
            .amount_in_games
            .checked_sub(ctx.accounts.game.wager_amount)
            .ok_or(RpsError::MathOverflow)?;

        if !is_p1_expired {
            ctx.accounts.player_2_info.amount_in_games = ctx
                .accounts
                .player_2_info
                .amount_in_games
                .checked_sub(ctx.accounts.game.wager_amount)
                .ok_or(RpsError::MathOverflow)?;
        }

        match ctx.accounts.game.state {
            GameState::Settled {
                result,
                player_1,
                player_2,
                config: _,
            } => match result {
                Winner::P1 => {
                    // if game expired they just get wager_amount back
                    let payout_amount = if player_1.pubkey() == player_2.pubkey() {
                        ctx.accounts.game.wager_amount
                    } else {
                        ctx.accounts.game.wager_amount * 2
                    };

                    anchor_spl::token::transfer(
                        CpiContext::new_with_signer(
                            ctx.accounts.token_program.to_account_info(),
                            anchor_spl::token::Transfer {
                                from: ctx.accounts.escrow_token_account.to_account_info(),
                                to: ctx.accounts.player1_token_account.to_account_info(),
                                authority: ctx.accounts.game_authority.to_account_info(),
                            },
                            &[&[
                                b"authority".as_ref(),
                                ctx.accounts.game.key().as_ref(),
                                &[*ctx.bumps.get("game_authority").unwrap()],
                            ]],
                        ),
                        payout_amount,
                    )?;

                    if !is_p1_expired {
                        ctx.accounts.player_1_info.games_won = ctx
                            .accounts
                            .player_1_info
                            .games_won
                            .checked_add(1)
                            .ok_or(RpsError::MathOverflow)?;
                        ctx.accounts.player_1_info.lifetime_earnings = ctx
                            .accounts
                            .player_1_info
                            .lifetime_earnings
                            .checked_add(ctx.accounts.game.wager_amount.try_into().unwrap())
                            .ok_or(RpsError::MathOverflow)?;

                        ctx.accounts.player_2_info.games_lost = ctx
                            .accounts
                            .player_2_info
                            .games_lost
                            .checked_add(1)
                            .ok_or(RpsError::MathOverflow)?;
                        ctx.accounts.player_2_info.lifetime_earnings = ctx
                            .accounts
                            .player_2_info
                            .lifetime_earnings
                            .checked_sub(ctx.accounts.game.wager_amount.try_into().unwrap())
                            .ok_or(RpsError::MathOverflow)?;
                    }
                }
                Winner::P2 => {
                    anchor_spl::token::transfer(
                        CpiContext::new_with_signer(
                            ctx.accounts.token_program.to_account_info(),
                            anchor_spl::token::Transfer {
                                from: ctx.accounts.escrow_token_account.to_account_info(),
                                to: ctx.accounts.player2_token_account.to_account_info(),
                                authority: ctx.accounts.game_authority.to_account_info(),
                            },
                            &[&[
                                b"authority".as_ref(),
                                ctx.accounts.game.key().as_ref(),
                                &[*ctx.bumps.get("game_authority").unwrap()],
                            ]],
                        ),
                        ctx.accounts.game.wager_amount * 2,
                    )?;

                    ctx.accounts.player_1_info.games_lost = ctx
                        .accounts
                        .player_1_info
                        .games_lost
                        .checked_add(1)
                        .ok_or(RpsError::MathOverflow)?;
                    ctx.accounts.player_1_info.lifetime_earnings = ctx
                        .accounts
                        .player_1_info
                        .lifetime_earnings
                        .checked_sub(ctx.accounts.game.wager_amount.try_into().unwrap())
                        .ok_or(RpsError::MathOverflow)?;

                    ctx.accounts.player_2_info.games_won = ctx
                        .accounts
                        .player_2_info
                        .games_won
                        .checked_add(1)
                        .ok_or(RpsError::MathOverflow)?;
                    ctx.accounts.player_2_info.lifetime_earnings = ctx
                        .accounts
                        .player_2_info
                        .lifetime_earnings
                        .checked_add(ctx.accounts.game.wager_amount.try_into().unwrap())
                        .ok_or(RpsError::MathOverflow)?;
                }
                Winner::TIE => {
                    anchor_spl::token::transfer(
                        CpiContext::new_with_signer(
                            ctx.accounts.token_program.to_account_info(),
                            anchor_spl::token::Transfer {
                                from: ctx.accounts.escrow_token_account.to_account_info(),
                                to: ctx.accounts.player1_token_account.to_account_info(),
                                authority: ctx.accounts.game_authority.to_account_info(),
                            },
                            &[&[
                                b"authority".as_ref(),
                                ctx.accounts.game.key().as_ref(),
                                &[*ctx.bumps.get("game_authority").unwrap()],
                            ]],
                        ),
                        ctx.accounts.game.wager_amount,
                    )?;
                    anchor_spl::token::transfer(
                        CpiContext::new_with_signer(
                            ctx.accounts.token_program.to_account_info(),
                            anchor_spl::token::Transfer {
                                from: ctx.accounts.escrow_token_account.to_account_info(),
                                to: ctx.accounts.player2_token_account.to_account_info(),
                                authority: ctx.accounts.game_authority.to_account_info(),
                            },
                            &[&[
                                b"authority".as_ref(),
                                ctx.accounts.game.key().as_ref(),
                                &[*ctx.bumps.get("game_authority").unwrap()],
                            ]],
                        ),
                        ctx.accounts.game.wager_amount,
                    )?;
                    ctx.accounts.player_1_info.games_drawn = ctx
                        .accounts
                        .player_1_info
                        .games_drawn
                        .checked_add(1)
                        .ok_or(RpsError::MathOverflow)?;

                    ctx.accounts.player_2_info.games_drawn = ctx
                        .accounts
                        .player_2_info
                        .games_drawn
                        .checked_add(1)
                        .ok_or(RpsError::MathOverflow)?;
                }
            },
            _ => panic!("Invalid state"),
        };

        // transfer out the fee
        anchor_spl::token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::Transfer {
                    from: ctx.accounts.escrow_token_account.to_account_info(),
                    to: ctx.accounts.player2_token_account.to_account_info(),
                    authority: ctx.accounts.game_authority.to_account_info(),
                },
                &[&[
                    b"authority".as_ref(),
                    ctx.accounts.game.key().as_ref(),
                    &[*ctx.bumps.get("game_authority").unwrap()],
                ]],
            ),
            ctx.accounts.game.fee_amount,
        )?;

        Ok(())
    }

    pub fn clean_game(ctx: Context<CleanGame>) -> Result<()> {
        match ctx.accounts.game.state {
            GameState::Settled {
                result,
                player_1,
                player_2,
                config: GameConfig { entry_proof },
            } => {
                let gr = ReadableGameEvent {
                    event_name: "game_result".to_string(),
                    event_version: 1,
                    player_1: player_1.pubkey().to_string(),
                    choice_1: player_1.choice_or_unrevealed(),
                    player_2: player_2.pubkey().to_string(),
                    choice_2: player_2.choice_or_unrevealed(),
                    result,
                    wager_amount: ctx.accounts.game.wager_amount,
                    fee_amount: ctx.accounts.game.fee_amount,
                    public: entry_proof.is_none(),
                };
                msg!("{}", serde_json::to_string(&gr).unwrap());
            }
            _ => {
                panic!("game not settled can't clean")
            }
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReadableGameEvent {
    event_name: String,
    event_version: u64,
    player_1: String,
    choice_1: Option<RPS>,
    player_2: String,
    choice_2: Option<RPS>,
    result: Winner,
    wager_amount: u64,
    fee_amount: u64,
    public: bool,
}

#[event]
pub struct GameStartEvent {
    game_pubkey: Pubkey,
    wager_amount: u64,
    fee_amount: u64,
    public: bool,
}

#[derive(Accounts)]
pub struct CreatePlayerInfo<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub mint: Account<'info, Mint>,

    #[account(
        init,
        seeds = [b"player_info".as_ref(), owner.key().as_ref(), mint.key().as_ref()],
        bump,
        payer = owner,
        space = PlayerInfo::space()
    )]
    pub player_info: Account<'info, PlayerInfo>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(game_seed: u64)]
pub struct CreateGame<'info> {
    #[account(
        init,
        seeds = [b"game".as_ref(), &game_seed.to_le_bytes()],
        bump,
        payer = player,
        space = Game::space()
    )]
    pub game: Account<'info, Game>,

    #[account(mut)]
    pub player: Signer<'info>,

    pub mint: Account<'info, Mint>,

    #[account(mut, constraint = player_token_account.owner == player.key())]
    pub player_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"player_info".as_ref(), player.key().as_ref(), mint.key().as_ref()],
        bump,
        constraint = player_info.owner == player.key()
    )]
    pub player_info: Account<'info, PlayerInfo>,

    /// CHECK: this is a pda that manages the escrow account
    #[account(mut, seeds = [b"authority".as_ref(), game.key().as_ref()], bump)]
    pub game_authority: AccountInfo<'info>,

    #[account(init,
        payer = player,
        seeds = [b"escrow".as_ref(), game.key().as_ref()],
        bump,
        token::mint = mint,
        token::authority = game_authority,
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct JoinGame<'info> {
    #[account(mut)]
    player: Signer<'info>,

    #[account(mut, constraint = player_token_account.owner == player.key())]
    pub player_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"player_info".as_ref(), player.key().as_ref()],
        bump,
        constraint = player_info.owner == player.key()
    )]
    pub player_info: Account<'info, PlayerInfo>,

    #[account(
        mut,
        seeds = [b"game".as_ref(), &game.seed.to_le_bytes()],
        bump,
    )]
    pub game: Account<'info, Game>,

    /// CHECK: this is a pda that manages the escrow account
    #[account(mut, seeds = [b"authority".as_ref(), game.key().as_ref()], bump)]
    pub game_authority: AccountInfo<'info>,

    #[account(mut,
        seeds = [b"escrow".as_ref(), game.key().as_ref()],
        bump,
        token::mint = game.mint,
        token::authority = game_authority,
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RevealGame<'info> {
    #[account(
        mut,
        seeds = [b"game".as_ref(), &game.seed.to_le_bytes()],
        bump,
    )]
    pub game: Account<'info, Game>,

    #[account(constraint = Some(player.key()) == game.player_1() || Some(player.key()) == game.player_2())]
    pub player: Signer<'info>,

    #[account(
        mut,
        seeds = [b"player_info".as_ref(), player.key().as_ref()],
        bump,
        constraint = player_info.owner == player.key()
    )]
    pub player_info: Account<'info, PlayerInfo>,
}

#[derive(Accounts)]
pub struct ExpireGame<'info> {
    #[account(
        mut,
        seeds = [b"game".as_ref(), &game.seed.to_le_bytes()],
        bump,
    )]
    pub game: Account<'info, Game>,

    /// CHECK: anyone can expire the game and the default winning player is
    /// checked in the game logic code so this doesn't need to be a signer
    #[account(constraint = Some(player.key()) == game.player_1() || Some(player.key()) == game.player_2())]
    pub player: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [b"player_info".as_ref(), player.key().as_ref()],
        bump,
        constraint = player_info.owner == player.key()
    )]
    pub player_info: Account<'info, PlayerInfo>,
}

#[derive(Accounts)]
pub struct SettleGame<'info> {
    #[account(
        mut,
        seeds = [b"game".as_ref(), &game.seed.to_le_bytes()],
        bump,
    )]
    pub game: Account<'info, Game>,

    /// CHECK: how do i make this check that it's the one in the enum lmao?
    #[account(mut, constraint = Some(player_1.key()) == game.player_1())]
    pub player_1: AccountInfo<'info>,
    #[account(mut, constraint = player1_token_account.owner == player_1.key())]
    pub player1_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [b"player_info".as_ref(), player_1.key().as_ref()],
        bump,
        constraint = player_1_info.owner == player_1.key()
    )]
    pub player_1_info: Account<'info, PlayerInfo>,

    /// CHECK:
    #[account(mut, constraint = Some(player_2.key()) == game.player_2())]
    pub player_2: AccountInfo<'info>,
    #[account(mut, constraint = player2_token_account.owner == player_2.key())]
    pub player2_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [b"player_info".as_ref(), player_2.key().as_ref()],
        bump,
        constraint = player_2_info.owner == player_2.key()
    )]
    pub player_2_info: Account<'info, PlayerInfo>,

    /// CHECK: this is a pda that manages the escrow account
    #[account(mut, seeds = [b"authority".as_ref(), game.key().as_ref()], bump)]
    pub game_authority: AccountInfo<'info>,

    #[account(mut,
        seeds = [b"escrow".as_ref(), game.key().as_ref()],
        bump,
        token::mint = game.mint,
        token::authority = game_authority,
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CleanGame<'info> {
    #[account(
        mut,
        close = player_1,
        seeds = [b"game".as_ref(), &game.seed.to_le_bytes()],
        bump,
    )]
    pub game: Account<'info, Game>,
    

    /// CHECK: this is a pda that manages the escrow account
    #[account(seeds = [b"authority".as_ref(), game.key().as_ref()], bump)]
    pub game_authority: AccountInfo<'info>,

    #[account(mut,
        close = player_1,
        seeds = [b"escrow".as_ref(), game.key().as_ref()],
        bump,
        token::mint = game.mint,
        token::authority = game_authority,
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,

    // #[account(mut, constraint = (&local_bpf_loader::id() == &rps_program.owner.key() || &game_cleaner::id() == &cleaner.key()))]
    // pub cleaner: Signer<'info>,
    // #[account(mut)]
    // pub cleaner: Signer<'info>,
    /// CHECK: this is just the play1 account we check in the constraint i matches the one on the game
    #[account(mut, constraint = Some(player_1.key()) == game.player_1())]
    pub player_1: AccountInfo<'info>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,

    pub rps_program: Program<'info, RpsToken>,
}

// #[account(zero_copy(unsafe))]
#[account]
#[derive(Debug, PartialEq, Eq, Copy)]
pub struct Game {
    pub seed: u64,
    pub mint: Pubkey,
    pub wager_amount: u64,
    pub fee_amount: u64,
    pub state: GameState,
}

impl Game {
    pub fn space() -> usize {
        // idk lmao
        192
    }
    pub fn player_1(self) -> Option<Pubkey> {
        match self.state {
            GameState::AcceptingChallenge { player_1, .. } => Some(player_1.pubkey()),
            GameState::AcceptingReveal { player_1, .. } => Some(player_1.pubkey()),
            GameState::AcceptingSettle { player_1, .. } => Some(player_1.pubkey()),
            GameState::Settled { player_1, .. } => Some(player_1.pubkey()),
            _ => None,
        }
    }
    pub fn player_2(self) -> Option<Pubkey> {
        match self.state {
            GameState::AcceptingReveal { player_2, .. } => Some(player_2.pubkey()),
            GameState::AcceptingSettle { player_2, .. } => Some(player_2.pubkey()),
            GameState::Settled { player_2, .. } => Some(player_2.pubkey()),
            _ => None,
        }
    }
}

#[account]
pub struct PlayerInfo {
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub games_won: u64,
    pub games_drawn: u64,
    pub games_lost: u64,

    pub lifetime_wagering: u64,
    pub lifetime_earnings: i64,

    pub amount_in_games: u64,
}

impl PlayerInfo {
    pub fn space() -> usize {
        // idk lmao leaving some space for expansion
        420
    }
}

#[error_code]
pub enum RpsError {
    #[msg("Bet too large")]
    BetTooLarge,
    #[msg("Math Overflow")]
    MathOverflow,
}
