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

solana_program::declare_id!("BF5PatmRTQDgEKoXR7iHRbkibEEi83nVM38cUKWzQcTR");
pub const STAKING_PROGRAM_ID: Pubkey = Pubkey::new_from_array([
    164, 185, 242, 100, 167, 7, 245, 144, 85, 224, 196, 151, 192, 113, 216, 27, 46, 144, 114, 107,
    253, 180, 250, 83, 45, 239, 58, 211, 199, 87, 250, 145,
]);
