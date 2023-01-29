use ring::digest::Context;
use ring::digest::SHA256;
use ring::digest::SHA256_OUTPUT_LEN;
use solana_sdk::pubkey::Pubkey;
use anchor_lang::prelude::*;

pub fn verify_commitment(
    pubkey: Pubkey,
    commitment: [u8; SHA256_OUTPUT_LEN],
    salt: u64,
    choice: RPS,
) -> bool {
    let mut context = Context::new(&SHA256);
    context.update(pubkey.as_ref());
    context.update(&salt.to_le_bytes());
    context.update([choice.into()].as_ref());

    let hash = context.finish();
    hash.as_ref() == commitment
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, AnchorSerialize, AnchorDeserialize)]
pub enum RPS {
    Rock,
    Paper,
    Scissors,
}

impl From<RPS> for u8 {
    fn from(rps: RPS) -> Self {
        match rps {
            RPS::Rock => 0,
            RPS::Paper => 1,
            RPS::Scissors => 2,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, AnchorSerialize, AnchorDeserialize)]
pub enum PlayerState {
    Empty,
    Waiting(Pubkey),
    Committed {
        pubkey: Pubkey,
        commitment: [u8; SHA256_OUTPUT_LEN],
    },
    Revealed(Pubkey, RPS),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, AnchorSerialize, AnchorDeserialize)]
pub struct GameConfig {
    wager_amount: u64,
    mint: Pubkey,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, AnchorSerialize, AnchorDeserialize)]
pub enum GameState {
    Initialized,
    Pending {
        player: Pubkey,
        config: GameConfig,
    },
    AcceptingCommitments {
        player_1: PlayerState,
        player_2: PlayerState,
        config: GameConfig,
    },
    AcceptingReveals {
        player_1: PlayerState,
        player_2: PlayerState,
        config: GameConfig,
    },
    Done {
        winner: Option<Pubkey>,
        player_1: PlayerState,
        player_2: PlayerState,
        config: GameConfig,
    },
    Settled {
        winner: Option<Pubkey>,
        player_1: PlayerState,
        player_2: PlayerState,
        config: GameConfig,
    },
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Actions {
    CreateGame {
        player: Pubkey,
        config: GameConfig,
    },
    JoinGame {
        player: Pubkey,
    },
    Commit {
        player: Pubkey,
        commitment: [u8; SHA256_OUTPUT_LEN],
    },
    Reveal {
        player: Pubkey,
        salt: u64,
        choice: RPS,
    },
    Settle,
}

pub fn process_action(state: GameState, action: Actions) -> GameState {
    match (state, action) {
        (GameState::Initialized, Actions::CreateGame { player, config }) => {
            GameState::Pending { player, config }
        }

        (
            GameState::Pending {
                player: player_1,
                config,
            },
            Actions::JoinGame { player: player_2 },
        ) => GameState::AcceptingCommitments {
            player_1: PlayerState::Waiting(player_1),
            player_2: PlayerState::Waiting(player_2),
            config,
        },

        (
            GameState::AcceptingCommitments {
                player_1,
                player_2,
                config,
            },
            Actions::Commit { player, commitment },
        ) => {
            let new_state = match (player_1, player_2) {
                (PlayerState::Waiting(player_1), player_2) if player == player_1 => {
                    GameState::AcceptingCommitments {
                        player_1: PlayerState::Committed {
                            pubkey: player_1,
                            commitment,
                        },
                        player_2,
                        config,
                    }
                }
                (player_1, PlayerState::Waiting(player_2)) if player == player_2 => {
                    GameState::AcceptingCommitments {
                        player_1,
                        player_2: PlayerState::Committed {
                            pubkey: player_2,
                            commitment,
                        },
                        config,
                    }
                }
                _ => panic!("Unreachable state"),
            };

            match new_state {
                GameState::AcceptingCommitments {
                    player_1: p1 @ PlayerState::Committed { .. },
                    player_2: p2 @ PlayerState::Committed { .. },
                    config,
                } => GameState::AcceptingReveals {
                    player_1: p1,
                    player_2: p2,
                    config,
                },
                _ => new_state,
            }
        }

        (
            GameState::AcceptingReveals {
                player_1,
                player_2,
                config,
            },
            Actions::Reveal {
                player,
                salt,
                choice,
            },
        ) => {
            let new_state = match (player_1, player_2) {
                (
                    PlayerState::Committed {
                        pubkey: player_1,
                        commitment,
                    },
                    player_2,
                ) if player == player_1 => {
                    if !verify_commitment(player_1, commitment, salt, choice) {
                        panic!("Invalid commitment");
                    }

                    GameState::AcceptingReveals {
                        player_1: PlayerState::Revealed(player_1, choice),
                        player_2,
                        config,
                    }
                }
                (
                    player_1,
                    PlayerState::Committed {
                        pubkey: player_2,
                        commitment,
                    },
                ) if player == player_2 => {
                    if !verify_commitment(player_2, commitment, salt, choice) {
                        panic!("Invalid commitment");
                    }

                    GameState::AcceptingReveals {
                        player_1,
                        player_2: PlayerState::Revealed(player_2, choice),
                        config,
                    }
                }
                _ => panic!("Unreachable state"),
            };

            match new_state {
                GameState::AcceptingReveals {
                    player_1: PlayerState::Revealed(p1, c1),
                    player_2: PlayerState::Revealed(p2, c2),
                    config,
                } => {
                    let winner = match (c1, c2) {
                        (RPS::Rock, RPS::Scissors) => Some(p1),
                        (RPS::Paper, RPS::Rock) => Some(p1),
                        (RPS::Scissors, RPS::Paper) => Some(p1),
                        (RPS::Rock, RPS::Paper) => Some(p2),
                        (RPS::Paper, RPS::Scissors) => Some(p2),
                        (RPS::Scissors, RPS::Rock) => Some(p2),
                        _ => None,
                    };

                    GameState::Done {
                        winner,
                        player_1: PlayerState::Revealed(p1, c1),
                        player_2: PlayerState::Revealed(p2, c2),
                        config,
                    }
                }
                _ => new_state,
            }
        }

        (
            GameState::Done {
                winner,
                player_1,
                player_2,
                config,
            },
            Actions::Settle,
        ) => GameState::Settled {
            winner,
            player_1,
            player_2,
            config,
        },

        _ => panic!("Invalid (state, action) pair: {:#?} {:#?}", state, action),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_process_action() {
        let state = GameState::Initialized;

        let player_1 = Pubkey::new_unique();
        let player_2 = Pubkey::new_unique();
        let usdc_mint = Pubkey::new_unique();

        let state = {
            let action = Actions::CreateGame {
                player: player_1,
                config: GameConfig {
                    wager_amount: 10,
                    mint: usdc_mint,
                },
            };
            let expected = GameState::Pending {
                player: player_1,
                config: GameConfig {
                    wager_amount: 10,
                    mint: usdc_mint,
                },
            };

            assert_eq!(process_action(state, action), expected);
            expected
        };

        let state = {
            let action = Actions::JoinGame { player: player_2 };
            let expected = GameState::AcceptingCommitments {
                player_1: PlayerState::Waiting(player_1),
                player_2: PlayerState::Waiting(player_2),
                config: GameConfig {
                    wager_amount: 10,
                    mint: usdc_mint,
                },
            };

            assert_eq!(process_action(state, action), expected);
            expected
        };

        let commitment_1 = create_commitment(player_1, 36, RPS::Rock);
        let state = {
            let action = Actions::Commit {
                player: player_1,
                commitment: commitment_1,
            };
            let expected = GameState::AcceptingCommitments {
                player_1: PlayerState::Committed {
                    pubkey: player_1,
                    commitment: commitment_1,
                },
                player_2: PlayerState::Waiting(player_2),
                config: GameConfig {
                    wager_amount: 10,
                    mint: usdc_mint,
                },
            };

            assert_eq!(process_action(state, action), expected);
            expected
        };

        let commitment_2 = create_commitment(player_2, 48, RPS::Paper);
        let state = {
            let action = Actions::Commit {
                player: player_2,
                commitment: commitment_2,
            };
            let expected = GameState::AcceptingReveals {
                player_1: PlayerState::Committed {
                    pubkey: player_1,
                    commitment: commitment_1,
                },
                player_2: PlayerState::Committed {
                    pubkey: player_2,
                    commitment: commitment_2,
                },
                config: GameConfig {
                    wager_amount: 10,
                    mint: usdc_mint,
                },
            };

            assert_eq!(process_action(state, action), expected);
            expected
        };

        let state = {
            let action = Actions::Reveal {
                player: player_1,
                salt: 36,
                choice: RPS::Rock,
            };
            let expected = GameState::AcceptingReveals {
                player_1: PlayerState::Revealed(player_1, RPS::Rock),
                player_2: PlayerState::Committed {
                    pubkey: player_2,
                    commitment: commitment_2,
                },
                config: GameConfig {
                    wager_amount: 10,
                    mint: usdc_mint,
                },
            };

            assert_eq!(process_action(state, action), expected);
            expected
        };

        let state = {
            let action = Actions::Reveal {
                player: player_2,
                salt: 48,
                choice: RPS::Paper,
            };
            let expected = GameState::Done {
                winner: Some(player_2),
                player_1: PlayerState::Revealed(player_1, RPS::Rock),
                player_2: PlayerState::Revealed(player_2, RPS::Paper),
                config: GameConfig {
                    wager_amount: 10,
                    mint: usdc_mint,
                },
            };

            assert_eq!(process_action(state, action), expected);
            expected
        };

        {
            let action = Actions::Settle;
            let expected = GameState::Settled {
                winner: Some(player_2),
                player_1: PlayerState::Revealed(player_1, RPS::Rock),
                player_2: PlayerState::Revealed(player_2, RPS::Paper),
                config: GameConfig {
                    wager_amount: 10,
                    mint: usdc_mint,
                },
            };

            assert_eq!(process_action(state, action), expected);
            expected
        };
    }

    pub fn create_commitment(pubkey: Pubkey, salt: u64, choice: RPS) -> [u8; SHA256_OUTPUT_LEN] {
        let mut context = Context::new(&SHA256);
        context.update(pubkey.as_ref());
        context.update(&salt.to_le_bytes());
        context.update([choice.into()].as_ref());

        let hash = context.finish();
        let mut buf = [0; SHA256_OUTPUT_LEN];
        buf.copy_from_slice(hash.as_ref());
        buf
    }
}
