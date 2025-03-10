// #![deny(missing_docs)]

//! Rewards contract

pub mod asserts;
#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;
pub mod error;
pub mod instruction;
pub mod instructions;
pub mod state;
pub mod utils;

pub use solana_program::{self, pubkey::Pubkey};

solana_program::declare_id!("5R1hXrm6aPgrjsDip4sJ9cByPtkzNXCBZut11bRauY36");

/// Address: BFzPVi4JULrec2xGWcUqtfNudZxUQQa2Qk2osn4uXB8P
pub const STAKING_PROGRAM_ID: Pubkey = Pubkey::new_from_array([
    152, 107, 227, 164, 196, 8, 225, 8, 221, 164, 56, 134, 190, 72, 90, 215, 218, 224, 215, 159,
    170, 41, 68, 57, 34, 129, 191, 156, 249, 82, 32, 36,
]);
/// HFWawhL4qv1a6MCMj7PnPPCZE7w5DW3r47pwZgRsiayC
/// Derived from PDA(realm.key(), "registrar", governing_token_mint),
/// Realm is derived from PDA("dao_name", governance_program_id)
pub const STAKING_PROGRAM_REGISTRAR: Pubkey = Pubkey::new_from_array([
    241, 114, 147, 3, 70, 61, 229, 235, 254, 172, 188, 149, 93, 10, 179, 67, 204, 89, 191, 235,
    123, 112, 229, 54, 132, 68, 132, 190, 218, 224, 2, 167,
]);
