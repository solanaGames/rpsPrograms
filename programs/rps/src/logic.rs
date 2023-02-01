use anchor_lang::prelude::*;
use solana_program::keccak::hashv;

pub fn verify_commitment(pubkey: Pubkey, commitment: [u8; 32], salt: u64, choice: RPS) -> bool {
    let choice8: u8 = choice.into();
    let buffer = &[pubkey.as_ref(), &salt.to_le_bytes(), &choice8.to_le_bytes()];
    let hash = hashv(buffer);
    hash.0 == commitment
}

pub fn verify_entry(pubkey: Pubkey, entry_proof: [u8; 32], secret: u64) -> bool {
    let hash = hashv(&[pubkey.as_ref(), &secret.to_le_bytes()]);
    hash.0 == entry_proof
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
pub enum Winner {
    P1,
    P2,
    TIE,
}
impl From<Winner> for u8 {
    fn from(result: Winner) -> Self {
        match result {
            Winner::P1 => 0,
            Winner::P2 => 1,
            Winner::TIE => 2,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, AnchorSerialize, AnchorDeserialize)]
pub enum PlayerState {
    Committed {
        pubkey: Pubkey,
        commitment: [u8; 32],
    },
    Revealed {
        pubkey: Pubkey,
        choice: RPS,
    },
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, AnchorSerialize, AnchorDeserialize)]
pub struct GameConfig {
    pub wager_amount: u64,
    pub mint: Pubkey,
    pub entry_proof: Option<[u8; 32]>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, AnchorSerialize, AnchorDeserialize)]
pub enum GameState {
    Initialized,
    AcceptingChallenge {
        config: GameConfig,
        player_1: PlayerState,
        expiry_slot: u64,
    },
    AcceptingReveal {
        player_1: PlayerState,
        player_2: PlayerState,
        config: GameConfig,
        expiry_slot: u64,
    },
    AcceptingSettle {
        result: Winner,
        player_1: PlayerState,
        player_2: PlayerState,
        config: GameConfig,
    },
    Settled {
        result: Winner,
        player_1: PlayerState,
        player_2: PlayerState,
        config: GameConfig,
    },
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, AnchorSerialize, AnchorDeserialize)]
pub enum Actions {
    CreateGame {
        player_1_pubkey: Pubkey,
        commitment: [u8; 32],
        config: GameConfig,
    },
    JoinGame {
        player_2_pubkey: Pubkey,
        choice: RPS,
        secret: Option<u64>,
    },
    Reveal {
        player_1_pubkey: Pubkey,
        salt: u64,
        choice: RPS,
    },
    ExpireGame {
        player_pubkey: Pubkey,
    },
    Settle,
}

pub fn process_action(
    state_pubkey: Pubkey,
    state: GameState,
    action: Actions,
    slot: u64,
) -> GameState {
    match (state, action) {
        (
            GameState::Initialized,
            Actions::CreateGame {
                player_1_pubkey: pubkey,
                commitment,
                config,
            },
        ) => {
            msg!("{:?}", commitment);
            GameState::AcceptingChallenge {
                config,
                player_1: PlayerState::Committed { pubkey, commitment },
                expiry_slot: slot + 2 * 60 * 5,
            }
        }

        (
            GameState::AcceptingChallenge {
                player_1,
                config:
                    GameConfig {
                        wager_amount,
                        mint,
                        entry_proof,
                    },
                expiry_slot,
            },
            Actions::JoinGame {
                player_2_pubkey,
                choice,
                secret,
            },
        ) => {
            if slot > expiry_slot {
                panic!("challenge expired");
            }
            if let Some(entry_proof) = entry_proof {
                if let Some(secret) = secret {
                    if !verify_entry(state_pubkey, entry_proof, secret) {
                        panic!("invalid entry secret")
                    }
                } else {
                    panic!("invalid entry secret")
                }
            }
            GameState::AcceptingReveal {
                player_1,
                player_2: PlayerState::Revealed {
                    pubkey: player_2_pubkey,
                    choice,
                },
                config: GameConfig {
                    wager_amount,
                    mint,
                    entry_proof,
                },
                expiry_slot: slot + 2 * 60 * 5,
            }
        }
        (
            GameState::AcceptingChallenge {
                player_1:
                    PlayerState::Committed {
                        pubkey: p1,
                        commitment: player_1_commitment,
                    },
                config,
                expiry_slot,
            },
            Actions::ExpireGame { player_pubkey },
        ) => {
            if slot < expiry_slot {
                panic!("challenge not expired yet");
            }
            if player_pubkey != p1 {
                panic!("only player 1 can expire unmatched games");
            }
            GameState::AcceptingSettle {
                result: Winner::P1,
                player_1: PlayerState::Committed {
                    pubkey: p1,
                    commitment: player_1_commitment,
                },
                player_2: PlayerState::Committed {
                    pubkey: p1,
                    commitment: player_1_commitment,
                },
                config,
            }
        }

        (
            GameState::AcceptingReveal {
                player_1:
                    PlayerState::Committed {
                        pubkey: p1,
                        commitment: player_1_commitment,
                    },
                player_2:
                    PlayerState::Revealed {
                        pubkey: p2,
                        choice: player_2_choice,
                    },
                config,
                expiry_slot,
            },
            Actions::Reveal {
                player_1_pubkey,
                salt,
                choice,
            },
        ) => {
            if slot > expiry_slot {
                panic!("challenge expired");
            }
            if p1 != player_1_pubkey {
                panic!("player1 must reveal");
            }
            if !verify_commitment(player_1_pubkey, player_1_commitment, salt, choice) {
                panic!("Invalid commitment");
            }
            let result = match (choice, player_2_choice) {
                (RPS::Rock, RPS::Scissors) => Winner::P1,
                (RPS::Paper, RPS::Rock) => Winner::P1,
                (RPS::Scissors, RPS::Paper) => Winner::P1,
                (RPS::Rock, RPS::Paper) => Winner::P2,
                (RPS::Paper, RPS::Scissors) => Winner::P2,
                (RPS::Scissors, RPS::Rock) => Winner::P2,
                _ => Winner::TIE,
            };
            GameState::AcceptingSettle {
                result,
                player_1: PlayerState::Revealed { pubkey: p1, choice },
                player_2: PlayerState::Revealed {
                    pubkey: p2,
                    choice: player_2_choice,
                },
                config,
            }
        }
        (
            GameState::AcceptingReveal {
                player_1,
                player_2:
                    PlayerState::Revealed {
                        pubkey: p2,
                        choice: player_2_choice,
                    },
                config,
                expiry_slot,
            },
            Actions::ExpireGame { player_pubkey },
        ) => {
            if slot < expiry_slot {
                panic!("challenge not expired yet");
            }
            if player_pubkey != p2 {
                panic!("only player 2 can expire unrevealed games");
            }
            GameState::AcceptingSettle {
                result: Winner::P2,
                player_1,
                player_2: PlayerState::Revealed {
                    pubkey: p2,
                    choice: player_2_choice,
                },
                config,
            }
        }

        (
            GameState::AcceptingSettle {
                result,
                player_1: p1,
                player_2: p2,
                config,
            },
            Actions::Settle,
        ) => GameState::Settled {
            result,
            player_1: p1,
            player_2: p2,
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

        let state_pubkey = Pubkey::new_unique();
        let player_1_pubkey = Pubkey::new_unique();
        let salt = 36;
        let commitment = create_commitment(player_1_pubkey, salt, RPS::Rock);
        let player_2_pubkey = Pubkey::new_unique();
        let usdc_mint = Pubkey::new_unique();
        let slot: u64 = 0;

        let state = {
            let action = Actions::CreateGame {
                player_1_pubkey,
                commitment,
                config: GameConfig {
                    wager_amount: 10,
                    mint: usdc_mint,
                    entry_proof: None,
                },
            };
            let expected = GameState::AcceptingChallenge {
                config: GameConfig {
                    wager_amount: 10,
                    mint: usdc_mint,
                    entry_proof: None,
                },
                player_1: PlayerState::Committed {
                    pubkey: player_1_pubkey,
                    commitment,
                },
                expiry_slot: 600,
            };

            assert_eq!(process_action(state_pubkey, state, action, slot), expected);
            expected
        };

        let state = {
            let action = Actions::JoinGame {
                player_2_pubkey,
                choice: RPS::Paper,
                secret: None,
            };
            let expected = GameState::AcceptingReveal {
                player_1: PlayerState::Committed {
                    pubkey: player_1_pubkey,
                    commitment,
                },
                player_2: PlayerState::Revealed {
                    pubkey: player_2_pubkey,
                    choice: RPS::Paper,
                },
                config: GameConfig {
                    wager_amount: 10,
                    mint: usdc_mint,
                    entry_proof: None,
                },
                expiry_slot: 600,
            };
            assert_eq!(process_action(state_pubkey, state, action, slot), expected);
            expected
        };

        let state = {
            let action = Actions::Reveal {
                player_1_pubkey,
                salt,
                choice: RPS::Rock,
            };
            let expected = GameState::AcceptingSettle {
                result: Winner::P2,
                player_1: PlayerState::Revealed {
                    pubkey: player_1_pubkey,
                    choice: RPS::Rock,
                },
                player_2: PlayerState::Revealed {
                    pubkey: player_2_pubkey,
                    choice: RPS::Paper,
                },
                config: GameConfig {
                    wager_amount: 10,
                    mint: usdc_mint,
                    entry_proof: None,
                },
            };
            assert_eq!(process_action(state_pubkey, state, action, slot), expected);
            expected
        };

        let _state = {
            let action = Actions::Settle;
            let expected = GameState::Settled {
                result: Winner::P2,
                player_1: PlayerState::Revealed {
                    pubkey: player_1_pubkey,
                    choice: RPS::Rock,
                },
                player_2: PlayerState::Revealed {
                    pubkey: player_2_pubkey,
                    choice: RPS::Paper,
                },
                config: GameConfig {
                    wager_amount: 10,
                    mint: usdc_mint,
                    entry_proof: None,
                },
            };
            assert_eq!(process_action(state_pubkey, state, action, slot), expected);
            expected
        };
    }

    #[test]
    fn test_process_action_with_secret() {
        let state = GameState::Initialized;

        let state_pubkey = Pubkey::new_unique();
        let player_1_pubkey = Pubkey::new_unique();
        let salt = 36;
        let commitment = create_commitment(player_1_pubkey, salt, RPS::Rock);
        let player_2_pubkey = Pubkey::new_unique();
        let usdc_mint = Pubkey::new_unique();
        let slot: u64 = 0;
        let secret = Some(8238538u64);
        let entry_proof = Some(create_entry_proof(state_pubkey, 8238538u64));

        let state = {
            let action = Actions::CreateGame {
                player_1_pubkey,
                commitment,
                config: GameConfig {
                    wager_amount: 10,
                    mint: usdc_mint,
                    entry_proof,
                },
            };
            let expected = GameState::AcceptingChallenge {
                config: GameConfig {
                    wager_amount: 10,
                    mint: usdc_mint,
                    entry_proof,
                },
                player_1: PlayerState::Committed {
                    pubkey: player_1_pubkey,
                    commitment,
                },
                expiry_slot: 600,
            };

            assert_eq!(process_action(state_pubkey, state, action, slot), expected);
            expected
        };

        let state = {
            let action = Actions::JoinGame {
                player_2_pubkey,
                choice: RPS::Paper,
                secret,
            };
            let expected = GameState::AcceptingReveal {
                player_1: PlayerState::Committed {
                    pubkey: player_1_pubkey,
                    commitment,
                },
                player_2: PlayerState::Revealed {
                    pubkey: player_2_pubkey,
                    choice: RPS::Paper,
                },
                config: GameConfig {
                    wager_amount: 10,
                    mint: usdc_mint,
                    entry_proof,
                },
                expiry_slot: 600,
            };
            assert_eq!(process_action(state_pubkey, state, action, slot), expected);
            expected
        };

        let _state = {
            let action = Actions::Reveal {
                player_1_pubkey,
                salt,
                choice: RPS::Rock,
            };
            let expected = GameState::AcceptingSettle {
                result: Winner::P2,
                player_1: PlayerState::Revealed {
                    pubkey: player_1_pubkey,
                    choice: RPS::Rock,
                },
                player_2: PlayerState::Revealed {
                    pubkey: player_2_pubkey,
                    choice: RPS::Paper,
                },
                config: GameConfig {
                    wager_amount: 10,
                    mint: usdc_mint,
                    entry_proof,
                },
            };
            assert_eq!(process_action(state_pubkey, state, action, slot), expected);
            expected
        };
    }

    pub fn create_commitment(pubkey: Pubkey, salt: u64, choice: RPS) -> [u8; 32] {
        let hash = hashv(&[
            pubkey.as_ref(),
            &salt.to_le_bytes(),
            [choice.into()].as_ref(),
        ]);
        // let mut buf = [0; 32];
        // buf.copy_from_slice(hash.as_ref());
        // buf

        // idk why but this works
        hash.0
    }

    pub fn create_entry_proof(pubkey: Pubkey, secret: u64) -> [u8; 32] {
        let hash = hashv(&[pubkey.as_ref(), &secret.to_le_bytes()]);
        // let mut buf = [0; 32];
        // buf.copy_from_slice(hash.as_ref());
        // buf

        // idk why but this works
        hash.0
    }
}
