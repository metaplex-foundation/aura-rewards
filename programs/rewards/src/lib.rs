#![deny(missing_docs)]

//! Rewards contract

pub mod entrypoint;
pub mod error;
pub mod instruction;
pub mod instructions;
pub mod processor;
pub mod state;
pub mod utils;

pub use solana_program;

solana_program::declare_id!("J8oa8UUJBydrTKtCdkvwmQQ27ZFDq54zAxWJY5Ey72Ji");
