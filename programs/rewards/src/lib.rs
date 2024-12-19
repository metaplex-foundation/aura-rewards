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

solana_program::declare_id!("CnAF8KyrSH3mmzdyeRR7wh69ZxBrxf3diue7VbRH8Nzh");

/// Address: FH6rnJ4qiVmzUCpnyRaQwCYcZhByY92NUQ6Eormni8J
pub const STAKING_PROGRAM_ID: Pubkey = Pubkey::new_from_array([
    3, 168, 86, 55, 247, 116, 32, 234, 116, 99, 208, 79, 42, 141, 17, 21, 75, 212, 118, 215, 162,
    134, 10, 98, 59, 43, 62, 230, 96, 243, 30, 211,
]);

// DmjYbU42Ycfcqj352QKeK4zquhg3ua24qQ2PaouxaN5m
pub const STAKING_PROGRAM_REGISTRAR: Pubkey = Pubkey::new_from_array([
    189, 193, 231, 121, 54, 169, 180, 47, 178, 231, 180, 250, 227, 113, 240, 243, 203, 229, 214,
    110, 128, 153, 62, 196, 154, 51, 38, 27, 4, 92, 221, 32,
]);
