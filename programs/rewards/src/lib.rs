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
use solana_program::pubkey::Pubkey;

solana_program::declare_id!("J8oa8UUJBydrTKtCdkvwmQQ27ZFDq54zAxWJY5Ey72Ji");

/// Generates mining address
pub fn find_mining_program_address(
    program_id: &Pubkey,
    user: &Pubkey,
    reward_pool: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            "mining".as_bytes(),
            &user.to_bytes(),
            &reward_pool.to_bytes(),
        ],
        program_id,
    )
}

/// Generates vault address
pub fn find_vault_program_address(
    program_id: &Pubkey,
    reward_pool: &Pubkey,
    reward_mint: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            "vault".as_bytes(),
            &reward_pool.to_bytes(),
            &reward_mint.to_bytes(),
        ],
        program_id,
    )
}
