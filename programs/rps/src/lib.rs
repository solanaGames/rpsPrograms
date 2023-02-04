use anchor_lang::prelude::*;
mod logic;

use logic::{process_action, Actions, GameConfig, GameState, Winner, RPS};
use program::Rps;
use serde::{Deserialize, Serialize};

declare_id!("rpsx2U29nY4LQmzw9kdvc7sgDBYK8N2UXpex3SJofuX");

pub mod game_cleaner {
    solana_program::declare_id!("rcxrvoLEpa65dqqRVEqAADicpNnhWBwtWtzyFHZMzKW");
}
pub mod local_bpf_loader {
    solana_program::declare_id!("BPFLoader2111111111111111111111111111111111");
}

#[program]
pub mod rps {
    use super::*;
    use crate::logic::RPS;

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
            config: GameConfig {
                entry_proof: entry_proof,
            },
        };

        ctx.accounts.game.seed = game_seed;
        ctx.accounts.game.wager_amount = wager_amount;
        ctx.accounts.game.state = process_action(
            ctx.accounts.game.key(),
            ctx.accounts.game.state,
            action,
            Clock::get()?.slot,
        );

        match ctx.accounts.game.state {
            GameState::AcceptingChallenge { .. } => {
                anchor_lang::system_program::transfer(
                    CpiContext::new(
                        ctx.accounts.system_program.to_account_info(),
                        anchor_lang::system_program::Transfer {
                            from: ctx.accounts.player.to_account_info(),
                            to: ctx.accounts.game_authority.to_account_info(),
                        },
                    ),
                    wager_amount,
                )?;
            }
            _ => panic!("Invalid state"),
        };

        emit!(GameStartEvent{
            game_pubkey: ctx.accounts.game.key(),
            wager_amount: wager_amount,
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
                anchor_lang::system_program::transfer(
                    CpiContext::new(
                        ctx.accounts.system_program.to_account_info(),
                        anchor_lang::system_program::Transfer {
                            from: ctx.accounts.player.to_account_info(),
                            to: ctx.accounts.game_authority.to_account_info(),
                        },
                    ),
                    ctx.accounts.game.wager_amount,
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
                config: _,
            } => match result {
                Winner::P1 => {
                    // if game expired they just get wager_amount back
                    let payout_amount = if player_1.pubkey() == player_2.pubkey() {
                        ctx.accounts.game.wager_amount
                    } else {
                        ctx.accounts.game.wager_amount * 2
                    };
                    anchor_lang::system_program::transfer(
                        CpiContext::new_with_signer(
                            ctx.accounts.system_program.to_account_info(),
                            anchor_lang::system_program::Transfer {
                                from: ctx.accounts.game_authority.to_account_info(),
                                to: ctx.accounts.player_1.to_account_info(),
                            },
                            &[&[
                                b"authority".as_ref(),
                                ctx.accounts.game.key().as_ref(),
                                &[*ctx.bumps.get("game_authority").unwrap()],
                            ]]
                        ),
                        payout_amount,
                    )?;
                }
                Winner::P2 => {
                    anchor_lang::system_program::transfer(
                        CpiContext::new_with_signer(
                            ctx.accounts.system_program.to_account_info(),
                            anchor_lang::system_program::Transfer {
                                from: ctx.accounts.game_authority.to_account_info(),
                                to: ctx.accounts.player_2.to_account_info(),
                            },
                            &[&[
                                b"authority".as_ref(),
                                ctx.accounts.game.key().as_ref(),
                                &[*ctx.bumps.get("game_authority").unwrap()],
                            ]]
                        ),
                        ctx.accounts.game.wager_amount * 2,
                    )?;
                }
                Winner::TIE => {
                    anchor_lang::system_program::transfer(
                        CpiContext::new_with_signer(
                            ctx.accounts.system_program.to_account_info(),
                            anchor_lang::system_program::Transfer {
                                from: ctx.accounts.game_authority.to_account_info(),
                                to: ctx.accounts.player_1.to_account_info(),
                            },
                            &[&[
                                b"authority".as_ref(),
                                ctx.accounts.game.key().as_ref(),
                                &[*ctx.bumps.get("game_authority").unwrap()],
                            ]]
                        ),
                        ctx.accounts.game.wager_amount,
                    )?;
                    anchor_lang::system_program::transfer(
                        CpiContext::new_with_signer(
                            ctx.accounts.system_program.to_account_info(),
                            anchor_lang::system_program::Transfer {
                                from: ctx.accounts.game_authority.to_account_info(),
                                to: ctx.accounts.player_2.to_account_info(),
                            },
                            &[&[
                                b"authority".as_ref(),
                                ctx.accounts.game.key().as_ref(),
                                &[*ctx.bumps.get("game_authority").unwrap()],
                            ]]
                        ),
                        ctx.accounts.game.wager_amount,
                    )?;
                }
            },
            _ => panic!("Invalid state"),
        };

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
                    result: result,
                    wager_amount: ctx.accounts.game.wager_amount,
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
    public: bool,
}

#[event]
pub struct GameStartEvent {
    game_pubkey: Pubkey,
    wager_amount: u64,
    public: bool,
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

    /// CHECK: this is a pda that manages the escrow account
    #[account(mut, seeds = [b"authority".as_ref(), game.key().as_ref()], bump)]
    pub game_authority: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct JoinGame<'info> {
    #[account(mut)]
    player: Signer<'info>,

    #[account(
        mut,
        seeds = [b"game".as_ref(), &game.seed.to_le_bytes()],
        bump,
    )]
    pub game: Account<'info, Game>,

    /// CHECK: this is a pda that escrows the sol
    #[account(mut, seeds = [b"authority".as_ref(), game.key().as_ref()], bump)]
    pub game_authority: AccountInfo<'info>,

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

    /// CHECK:
    #[account(mut, constraint = Some(player_2.key()) == game.player_2())]
    pub player_2: AccountInfo<'info>,

    /// CHECK: this is a pda that manages the escrow account
    #[account(mut, seeds = [b"authority".as_ref(), game.key().as_ref()], bump)]
    pub game_authority: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CleanGame<'info> {
    #[account(
        mut,
        close = cleaner,
        seeds = [b"game".as_ref(), &game.seed.to_le_bytes()],
        bump,
    )]
    pub game: Account<'info, Game>,

    /// CHECK: this is a pda that manages the escrow account
    #[account(seeds = [b"authority".as_ref(), game.key().as_ref()], bump)]
    pub game_authority: AccountInfo<'info>,

    #[account(mut, constraint = (&local_bpf_loader::id() == &rps_program.owner.key() || &game_cleaner::id() == &cleaner.key()))]
    pub cleaner: Signer<'info>,

    pub system_program: Program<'info, System>,
    
    pub rps_program: Program<'info, Rps>
}

#[account]
#[derive(Debug, PartialEq, Eq, Copy)]
pub struct Game {
    pub seed: u64,
    pub wager_amount: u64,
    pub state: GameState,
}

impl Game {
    pub fn space() -> usize {
        // idk lmao
        232
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
