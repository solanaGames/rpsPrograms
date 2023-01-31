export type Rps = {
  "version": "0.1.0",
  "name": "rps",
  "instructions": [
    {
      "name": "createGame",
      "accounts": [
        {
          "name": "game",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "player",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "mint",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "playerTokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "gameAuthority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "escrowTokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "associatedTokenProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "gameSeed",
          "type": "u64"
        },
        {
          "name": "commitment",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        },
        {
          "name": "wagerAmount",
          "type": "u64"
        }
      ]
    },
    {
      "name": "joinGame",
      "accounts": [
        {
          "name": "player",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "game",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "playerTokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "gameAuthority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "escrowTokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "choice",
          "type": {
            "defined": "RPS"
          }
        },
        {
          "name": "secret",
          "type": {
            "option": "u64"
          }
        }
      ]
    },
    {
      "name": "revealGame",
      "accounts": [
        {
          "name": "game",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "player",
          "isMut": true,
          "isSigner": true
        }
      ],
      "args": [
        {
          "name": "choice",
          "type": {
            "defined": "RPS"
          }
        },
        {
          "name": "salt",
          "type": "u64"
        }
      ]
    },
    {
      "name": "expireGame",
      "accounts": [
        {
          "name": "game",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "player",
          "isMut": true,
          "isSigner": true
        }
      ],
      "args": []
    },
    {
      "name": "settleGame",
      "accounts": [
        {
          "name": "game",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "player1TokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "player2TokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "gameAuthority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "escrowTokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": []
    },
    {
      "name": "cleanGame",
      "accounts": [
        {
          "name": "game",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "gameAuthority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "escrowTokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "cleaner",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": []
    }
  ],
  "accounts": [
    {
      "name": "game",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "seed",
            "type": "u64"
          },
          {
            "name": "state",
            "type": {
              "defined": "GameState"
            }
          }
        ]
      }
    }
  ],
  "types": [
    {
      "name": "GameConfig",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "wagerAmount",
            "type": "u64"
          },
          {
            "name": "mint",
            "type": "publicKey"
          },
          {
            "name": "entryProof",
            "type": {
              "option": {
                "array": [
                  "u8",
                  32
                ]
              }
            }
          }
        ]
      }
    },
    {
      "name": "RPS",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Rock"
          },
          {
            "name": "Paper"
          },
          {
            "name": "Scissors"
          }
        ]
      }
    },
    {
      "name": "Winner",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "P1"
          },
          {
            "name": "P2"
          },
          {
            "name": "TIE"
          }
        ]
      }
    },
    {
      "name": "PlayerState",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Committed",
            "fields": [
              {
                "name": "pubkey",
                "type": "publicKey"
              },
              {
                "name": "commitment",
                "type": {
                  "array": [
                    "u8",
                    32
                  ]
                }
              }
            ]
          },
          {
            "name": "Revealed",
            "fields": [
              {
                "name": "pubkey",
                "type": "publicKey"
              },
              {
                "name": "choice",
                "type": {
                  "defined": "RPS"
                }
              }
            ]
          }
        ]
      }
    },
    {
      "name": "GameState",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Initialized"
          },
          {
            "name": "AcceptingChallenge",
            "fields": [
              {
                "name": "config",
                "type": {
                  "defined": "GameConfig"
                }
              },
              {
                "name": "player_1",
                "type": {
                  "defined": "PlayerState"
                }
              },
              {
                "name": "expiry_slot",
                "type": "u64"
              }
            ]
          },
          {
            "name": "AcceptingReveal",
            "fields": [
              {
                "name": "player_1",
                "type": {
                  "defined": "PlayerState"
                }
              },
              {
                "name": "player_2",
                "type": {
                  "defined": "PlayerState"
                }
              },
              {
                "name": "config",
                "type": {
                  "defined": "GameConfig"
                }
              },
              {
                "name": "expiry_slot",
                "type": "u64"
              }
            ]
          },
          {
            "name": "AcceptingSettle",
            "fields": [
              {
                "name": "result",
                "type": {
                  "defined": "Winner"
                }
              },
              {
                "name": "player_1",
                "type": {
                  "defined": "PlayerState"
                }
              },
              {
                "name": "player_2",
                "type": {
                  "defined": "PlayerState"
                }
              },
              {
                "name": "config",
                "type": {
                  "defined": "GameConfig"
                }
              }
            ]
          },
          {
            "name": "Settled",
            "fields": [
              {
                "name": "result",
                "type": {
                  "defined": "Winner"
                }
              },
              {
                "name": "player_1",
                "type": {
                  "defined": "PlayerState"
                }
              },
              {
                "name": "player_2",
                "type": {
                  "defined": "PlayerState"
                }
              },
              {
                "name": "config",
                "type": {
                  "defined": "GameConfig"
                }
              }
            ]
          }
        ]
      }
    },
    {
      "name": "Actions",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "CreateGame",
            "fields": [
              {
                "name": "player_1_pubkey",
                "type": "publicKey"
              },
              {
                "name": "commitment",
                "type": {
                  "array": [
                    "u8",
                    32
                  ]
                }
              },
              {
                "name": "config",
                "type": {
                  "defined": "GameConfig"
                }
              }
            ]
          },
          {
            "name": "JoinGame",
            "fields": [
              {
                "name": "player_2_pubkey",
                "type": "publicKey"
              },
              {
                "name": "choice",
                "type": {
                  "defined": "RPS"
                }
              },
              {
                "name": "secret",
                "type": {
                  "option": "u64"
                }
              }
            ]
          },
          {
            "name": "Reveal",
            "fields": [
              {
                "name": "player_1_pubkey",
                "type": "publicKey"
              },
              {
                "name": "salt",
                "type": "u64"
              },
              {
                "name": "choice",
                "type": {
                  "defined": "RPS"
                }
              }
            ]
          },
          {
            "name": "ExpireGame",
            "fields": [
              {
                "name": "player_pubkey",
                "type": "publicKey"
              }
            ]
          },
          {
            "name": "Settle"
          }
        ]
      }
    }
  ]
};

export const IDL: Rps = {
  "version": "0.1.0",
  "name": "rps",
  "instructions": [
    {
      "name": "createGame",
      "accounts": [
        {
          "name": "game",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "player",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "mint",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "playerTokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "gameAuthority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "escrowTokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "associatedTokenProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "gameSeed",
          "type": "u64"
        },
        {
          "name": "commitment",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        },
        {
          "name": "wagerAmount",
          "type": "u64"
        }
      ]
    },
    {
      "name": "joinGame",
      "accounts": [
        {
          "name": "player",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "game",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "playerTokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "gameAuthority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "escrowTokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "choice",
          "type": {
            "defined": "RPS"
          }
        },
        {
          "name": "secret",
          "type": {
            "option": "u64"
          }
        }
      ]
    },
    {
      "name": "revealGame",
      "accounts": [
        {
          "name": "game",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "player",
          "isMut": true,
          "isSigner": true
        }
      ],
      "args": [
        {
          "name": "choice",
          "type": {
            "defined": "RPS"
          }
        },
        {
          "name": "salt",
          "type": "u64"
        }
      ]
    },
    {
      "name": "expireGame",
      "accounts": [
        {
          "name": "game",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "player",
          "isMut": true,
          "isSigner": true
        }
      ],
      "args": []
    },
    {
      "name": "settleGame",
      "accounts": [
        {
          "name": "game",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "player1TokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "player2TokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "gameAuthority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "escrowTokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": []
    },
    {
      "name": "cleanGame",
      "accounts": [
        {
          "name": "game",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "gameAuthority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "escrowTokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "cleaner",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": []
    }
  ],
  "accounts": [
    {
      "name": "game",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "seed",
            "type": "u64"
          },
          {
            "name": "state",
            "type": {
              "defined": "GameState"
            }
          }
        ]
      }
    }
  ],
  "types": [
    {
      "name": "GameConfig",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "wagerAmount",
            "type": "u64"
          },
          {
            "name": "mint",
            "type": "publicKey"
          },
          {
            "name": "entryProof",
            "type": {
              "option": {
                "array": [
                  "u8",
                  32
                ]
              }
            }
          }
        ]
      }
    },
    {
      "name": "RPS",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Rock"
          },
          {
            "name": "Paper"
          },
          {
            "name": "Scissors"
          }
        ]
      }
    },
    {
      "name": "Winner",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "P1"
          },
          {
            "name": "P2"
          },
          {
            "name": "TIE"
          }
        ]
      }
    },
    {
      "name": "PlayerState",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Committed",
            "fields": [
              {
                "name": "pubkey",
                "type": "publicKey"
              },
              {
                "name": "commitment",
                "type": {
                  "array": [
                    "u8",
                    32
                  ]
                }
              }
            ]
          },
          {
            "name": "Revealed",
            "fields": [
              {
                "name": "pubkey",
                "type": "publicKey"
              },
              {
                "name": "choice",
                "type": {
                  "defined": "RPS"
                }
              }
            ]
          }
        ]
      }
    },
    {
      "name": "GameState",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Initialized"
          },
          {
            "name": "AcceptingChallenge",
            "fields": [
              {
                "name": "config",
                "type": {
                  "defined": "GameConfig"
                }
              },
              {
                "name": "player_1",
                "type": {
                  "defined": "PlayerState"
                }
              },
              {
                "name": "expiry_slot",
                "type": "u64"
              }
            ]
          },
          {
            "name": "AcceptingReveal",
            "fields": [
              {
                "name": "player_1",
                "type": {
                  "defined": "PlayerState"
                }
              },
              {
                "name": "player_2",
                "type": {
                  "defined": "PlayerState"
                }
              },
              {
                "name": "config",
                "type": {
                  "defined": "GameConfig"
                }
              },
              {
                "name": "expiry_slot",
                "type": "u64"
              }
            ]
          },
          {
            "name": "AcceptingSettle",
            "fields": [
              {
                "name": "result",
                "type": {
                  "defined": "Winner"
                }
              },
              {
                "name": "player_1",
                "type": {
                  "defined": "PlayerState"
                }
              },
              {
                "name": "player_2",
                "type": {
                  "defined": "PlayerState"
                }
              },
              {
                "name": "config",
                "type": {
                  "defined": "GameConfig"
                }
              }
            ]
          },
          {
            "name": "Settled",
            "fields": [
              {
                "name": "result",
                "type": {
                  "defined": "Winner"
                }
              },
              {
                "name": "player_1",
                "type": {
                  "defined": "PlayerState"
                }
              },
              {
                "name": "player_2",
                "type": {
                  "defined": "PlayerState"
                }
              },
              {
                "name": "config",
                "type": {
                  "defined": "GameConfig"
                }
              }
            ]
          }
        ]
      }
    },
    {
      "name": "Actions",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "CreateGame",
            "fields": [
              {
                "name": "player_1_pubkey",
                "type": "publicKey"
              },
              {
                "name": "commitment",
                "type": {
                  "array": [
                    "u8",
                    32
                  ]
                }
              },
              {
                "name": "config",
                "type": {
                  "defined": "GameConfig"
                }
              }
            ]
          },
          {
            "name": "JoinGame",
            "fields": [
              {
                "name": "player_2_pubkey",
                "type": "publicKey"
              },
              {
                "name": "choice",
                "type": {
                  "defined": "RPS"
                }
              },
              {
                "name": "secret",
                "type": {
                  "option": "u64"
                }
              }
            ]
          },
          {
            "name": "Reveal",
            "fields": [
              {
                "name": "player_1_pubkey",
                "type": "publicKey"
              },
              {
                "name": "salt",
                "type": "u64"
              },
              {
                "name": "choice",
                "type": {
                  "defined": "RPS"
                }
              }
            ]
          },
          {
            "name": "ExpireGame",
            "fields": [
              {
                "name": "player_pubkey",
                "type": "publicKey"
              }
            ]
          },
          {
            "name": "Settle"
          }
        ]
      }
    }
  ]
};
