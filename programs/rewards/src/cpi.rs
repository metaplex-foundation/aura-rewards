//! CPI

use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::program::{invoke, invoke_signed};
use solana_program::pubkey::Pubkey;

use crate::utils::LockupPeriod;

/// Rewards initialize mining
#[allow(clippy::too_many_arguments)]
pub fn initialize_mining<'a>(
    program_id: &Pubkey,
    reward_pool: AccountInfo<'a>,
    mining: AccountInfo<'a>,
    user: AccountInfo<'a>,
    payer: AccountInfo<'a>,
    system_program: AccountInfo<'a>,
    rent: AccountInfo<'a>,
) -> ProgramResult {
    let ix = crate::instruction::initialize_mining(
        program_id,
        reward_pool.key,
        mining.key,
        user.key,
        payer.key,
    );

    invoke(
        &ix,
        &[reward_pool, mining, user, payer, system_program, rent],
    )
}

/// Rewards deposit mining
#[allow(clippy::too_many_arguments)]
pub fn deposit_mining<'a>(
    program_id: &Pubkey,
    reward_pool: AccountInfo<'a>,
    mining: AccountInfo<'a>,
    user: AccountInfo<'a>,
    deposit_authority: AccountInfo<'a>,
    amount: u64,
    lockup_period: LockupPeriod,
    signers_seeds: &[&[&[u8]]],
) -> ProgramResult {
    let ix = crate::instruction::deposit_mining(
        program_id,
        reward_pool.key,
        mining.key,
        user.key,
        deposit_authority.key,
        amount,
        lockup_period,
    );

    invoke_signed(
        &ix,
        &[reward_pool, mining, user, deposit_authority],
        signers_seeds,
    )
}

/// Rewards withdraw mining
#[allow(clippy::too_many_arguments)]
pub fn withdraw_mining<'a>(
    program_id: &Pubkey,
    reward_pool: AccountInfo<'a>,
    mining: AccountInfo<'a>,
    user: AccountInfo<'a>,
    deposit_authority: AccountInfo<'a>,
    amount: u64,
    signers_seeds: &[&[&[u8]]],
) -> ProgramResult {
    let ix = crate::instruction::withdraw_mining(
        program_id,
        reward_pool.key,
        mining.key,
        user.key,
        deposit_authority.key,
        amount,
    );

    invoke_signed(
        &ix,
        &[reward_pool, mining, user, deposit_authority],
        signers_seeds,
    )
}

/// Rewards fill vault
#[allow(clippy::too_many_arguments)]
pub fn fill_vault<'a>(
    program_id: &Pubkey,
    reward_pool: AccountInfo<'a>,
    reward_mint: AccountInfo<'a>,
    fee_account: AccountInfo<'a>,
    vault: AccountInfo<'a>,
    from: AccountInfo<'a>,
    authority: AccountInfo<'a>,
    amount: u64,
    signers_seeds: &[&[&[u8]]],
) -> ProgramResult {
    let ix = crate::instruction::fill_vault(
        program_id,
        reward_pool.key,
        reward_mint.key,
        vault.key,
        fee_account.key,
        authority.key,
        from.key,
        amount,
    );

    invoke_signed(
        &ix,
        &[
            reward_pool,
            reward_mint,
            vault,
            fee_account,
            from,
            authority,
        ],
        signers_seeds,
    )
}
