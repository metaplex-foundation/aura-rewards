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

solana_program::declare_id!("DdAfv8RS2BS41FRjDX5nLXSmQWrPsdC17sbgD66oKcU8");

/// Address: DdAfv8RS2BS41FRjDX5nLXSmQWrPsdC17sbgD66oKcU8
pub const STAKING_PROGRAM_ID: Pubkey = Pubkey::new_from_array([
    164, 185, 242, 100, 167, 7, 245, 144, 85, 224, 196, 151, 192, 113, 216, 27, 46, 144, 114, 107,
    253, 180, 250, 83, 45, 239, 58, 211, 199, 87, 250, 145,
]);

/// 88qpWZJxdKku9vsg7otRCUbk7rAjkQmRxB9QxDbRwg9B
/// Derived from PDA(realm.key(), "registrar", governing_token_mint),
/// Realm is derived from PDA("dao_name", governance_program_id)
pub const STAKING_PROGRAM_REGISTRAR: Pubkey = Pubkey::new_from_array([
    106, 4, 18, 249, 60, 28, 25, 114, 22, 73, 216, 106, 24, 44, 254, 61, 181, 207, 37, 223, 224,
    123, 86, 118, 101, 68, 246, 132, 166, 69, 19, 6,
]);
