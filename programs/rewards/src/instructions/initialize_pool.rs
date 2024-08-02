use crate::{
    asserts::{assert_account_key, assert_account_len, assert_account_owner},
    state::{RewardPool, WrappedRewardPool},
    utils::{create_account, find_vault_program_address, initialize_account, AccountLoader},
};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey, rent::Rent,
    system_program, sysvar::SysvarId,
};
use spl_token::state::Account as SplTokenAccount;

pub fn process_initialize_pool<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    fill_authority: Pubkey,
    distribute_authority: Pubkey,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter().enumerate();

    let reward_pool = AccountLoader::next_uninitialized(account_info_iter)?;
    let reward_mint = AccountLoader::next_with_owner(account_info_iter, &spl_token::id())?;
    let reward_vault = AccountLoader::next_uninitialized(account_info_iter)?;
    let payer = AccountLoader::next_signer(account_info_iter)?;
    let deposit_authority = AccountLoader::next_signer(account_info_iter)?;
    let rent = AccountLoader::next_with_key(account_info_iter, &Rent::id())?;
    let _token_program = AccountLoader::next_with_key(account_info_iter, &spl_token::id())?;
    let _system_program = AccountLoader::next_with_key(account_info_iter, &system_program::id())?;

    assert_account_owner(reward_pool, program_id)?;
    assert_account_len(reward_pool, WrappedRewardPool::LEN)?;

    let (vault_pubkey, token_account_bump) =
        find_vault_program_address(program_id, reward_pool.key, reward_mint.key);
    assert_account_key(reward_vault, &vault_pubkey)?;
    let vault_seeds = &[
        b"vault".as_ref(),
        reward_pool.key.as_ref(),
        reward_mint.key.as_ref(),
        &[token_account_bump],
    ];

    create_account::<SplTokenAccount>(
        &spl_token::id(),
        payer.clone(),
        reward_vault.clone(),
        &[vault_seeds],
    )?;
    initialize_account(
        reward_vault.clone(),
        reward_mint.clone(),
        deposit_authority.clone(),
        rent.clone(),
    )?;

    let pool = RewardPool::initialize(
        token_account_bump,
        *deposit_authority.key,
        distribute_authority,
        fill_authority,
        *reward_mint.key,
    );

    let reward_pool_data = &mut reward_pool.data.borrow_mut();
    let wrapped_reward_pool = WrappedRewardPool::from_bytes_mut(reward_pool_data)?;

    *wrapped_reward_pool.pool = pool;
    wrapped_reward_pool.weighted_stake_diffs.initialize();
    wrapped_reward_pool.cumulative_index.initialize();

    Ok(())
}
