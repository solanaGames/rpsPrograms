use ring::digest::Context;
use ring::digest::SHA256;
use ring::digest::SHA256_OUTPUT_LEN;
use solana_sdk::pubkey::Pubkey;
use anchor_lang::prelude::*;
use solana_sdk::slot_history::Slot;

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
pub enum RESULT {
    P1,
    P2,
    TIE,
}
impl From<RESULT> for u8 {
    fn from(result: RESULT) -> Self {
        match result {
            RESULT::P1 => 0,
            RESULT::P2 => 1,
            RESULT::TIE => 2,
        }
    }
}


#[derive(Debug, PartialEq, Eq, Clone, Copy, AnchorSerialize, AnchorDeserialize)]
pub enum PlayerState {
    Empty,
    Waiting(Pubkey),
    Committed(Pubkey, [u8; SHA256_OUTPUT_LEN]),
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
    AcceptingChallenge {
        config: GameConfig,
        player_1: PlayerState,
        expiry_slot: Slot,
    },
    AcceptingReveal {
        player_1: PlayerState,
        player_2: PlayerState,
        config: GameConfig,
        expiry_slot: Slot,
    },
    AcceptingSettle {
        result: RESULT,
        player_1: PlayerState,
        player_2: PlayerState,
        config: GameConfig,
    },
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Actions {
    CreateGame {
        player_1_pubkey: Pubkey,
        commitment: [u8; SHA256_OUTPUT_LEN],
        config: GameConfig,
    },
    JoinGame {
        player_2_pubkey: Pubkey,
        choice: RPS,
    },
    Reveal {
        player_1_pubkey: Pubkey,
        salt: u64,
        choice: RPS,
    },
    ExpireGame {
        player: Pubkey,
    },
    Settle,
}

pub fn process_action(state: GameState, action: Actions, slot: Slot) -> GameState {
    match (state, action) {
        (GameState::Initialized, Actions::CreateGame { player_1_pubkey, commitment, config }) => {
            GameState::AcceptingChallenge { 
                config, 
                player_1: PlayerState::Committed(player_1_pubkey, commitment),
                expiry_slot: slot + 2 * 60 * 5
            }
        }

        (
            GameState::AcceptingChallenge {
                player_1,
                config,
                expiry_slot
            },
            Actions::JoinGame { player_2_pubkey, choice },
        ) => {
            if slot > expiry_slot {
                panic!("challenge expired");
            }
            GameState::AcceptingReveal {
                player_1,
                player_2: PlayerState::Revealed(player_2_pubkey, choice),
                config,
                expiry_slot: slot + 2 * 60 * 5
            }
        },
        (
            GameState::AcceptingChallenge {
                player_1: PlayerState::Committed(p1, player_1_commitment),
                config,
                expiry_slot
            },
            Actions::ExpireGame { player},
        ) => {
            if slot < expiry_slot {
                panic!("challenge not expired yet");
            }
            if player != p1 {
                panic!("only player 1 can expire unmatched games");
            }
            GameState::AcceptingSettle {
                result: RESULT::P1,
                player_1: PlayerState::Committed(p1, player_1_commitment),
                player_2: PlayerState::Committed(p1, player_1_commitment),
                config,
            }
        },

        (
            GameState::AcceptingReveal {
                player_1: PlayerState::Committed(p1, player_1_commitment),
                player_2: PlayerState::Revealed(p2, player_2_choice),
                config,
                expiry_slot,
            },
            Actions::Reveal { player_1_pubkey, salt, choice }
        ) => {
            if slot > expiry_slot {
                panic!("challenge expired");
            }
            if p1 != player_1_pubkey {
                panic!("player1 must reveal");
            }
            if !verify_commitment(player_1_pubkey, player_1_commitment, salt, choice){
                panic!("Invalid commitment");
            }
            let result = match (choice, player_2_choice) {
                (RPS::Rock, RPS::Scissors) => RESULT::P1,
                (RPS::Paper, RPS::Rock) => RESULT::P1,
                (RPS::Scissors, RPS::Paper) => RESULT::P1,
                (RPS::Rock, RPS::Paper) => RESULT::P2,
                (RPS::Paper, RPS::Scissors) => RESULT::P2,
                (RPS::Scissors, RPS::Rock) => RESULT::P2,
                _ => RESULT::TIE,
            };
            GameState::AcceptingSettle {
                result,
                player_1: PlayerState::Revealed(p1, choice),
                player_2: PlayerState::Revealed(p2, player_2_choice),
                config,
            }
        }
        (
            GameState::AcceptingReveal {
                player_1,
                player_2: PlayerState::Revealed(p2, player_2_choice),
                config,
                expiry_slot,
            },
            Actions::ExpireGame { player},
        ) => {
            if slot < expiry_slot {
                panic!("challenge not expired yet");
            }
            if player != p2 {
                panic!("only player 2 can expire unrevealed games");
            }
            GameState::AcceptingSettle {
                result: RESULT::P2,
                player_1,
                player_2: PlayerState::Revealed(p2, player_2_choice),
                config,
            }
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

        let player_1_pubkey = Pubkey::new_unique();
        let salt = 36;
        let commitment = create_commitment(player_1_pubkey, salt, RPS::Rock);
        let player_2_pubkey = Pubkey::new_unique();
        let usdc_mint = Pubkey::new_unique();
        let slot: Slot = 0;

        let state = {
            let action = Actions::CreateGame {
                player_1_pubkey,
                commitment,
                config: GameConfig {
                    wager_amount: 10,
                    mint: usdc_mint,
                },
            };
            let expected = GameState::AcceptingChallenge { 
                config: GameConfig {
                    wager_amount: 10,
                    mint: usdc_mint,
                }, 
                player_1: PlayerState::Committed(player_1_pubkey, commitment),
                expiry_slot: 600,
            };

            assert_eq!(process_action(state, action, slot), expected);
            expected
        };

        let state = {
            let action = Actions::JoinGame{
                player_2_pubkey,
                choice: RPS::Paper,
            };
            let expected = GameState::AcceptingReveal { 
                player_1: PlayerState::Committed(player_1_pubkey, commitment), 
                player_2: PlayerState::Revealed(player_2_pubkey, RPS::Paper), 
                config: GameConfig {
                    wager_amount: 10,
                    mint: usdc_mint,
                },
                expiry_slot: 600,
            };
            assert_eq!(process_action(state, action, slot), expected);
            expected
        };

        let _state = {
            let action = Actions::Reveal { 
                player_1_pubkey, 
                salt, 
                choice: RPS::Rock
            };
            let expected = GameState::AcceptingSettle { 
                result: RESULT::P2,
                player_1: PlayerState::Revealed(player_1_pubkey, RPS::Rock), 
                player_2: PlayerState::Revealed(player_2_pubkey, RPS::Paper), 
                config: GameConfig {
                    wager_amount: 10,
                    mint: usdc_mint,
                }, 
            };
            assert_eq!(process_action(state, action, slot), expected);
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
