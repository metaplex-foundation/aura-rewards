use crate::{
    asserts::assert_account_key,
    state::{WrappedMining, WrappedRewardPool},
    utils::{spl_transfer, AccountLoader},
};
use borsh::BorshSerialize;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program::set_return_data,
    program_error::ProgramError, program_pack::Pack, pubkey::Pubkey,
};
use spl_token::state::Account;

pub fn process_claim<'a>(program_id: &Pubkey, accounts: &'a [AccountInfo<'a>]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter().enumerate();

    let reward_pool = AccountLoader::next_with_owner(account_info_iter, program_id)?;
    let reward_mint = AccountLoader::next_with_owner(account_info_iter, &spl_token::id())?;
    let vault = AccountLoader::next_with_owner(account_info_iter, &spl_token::id())?;
    let mining = AccountLoader::next_with_owner(account_info_iter, program_id)?;
    let mining_owner = AccountLoader::next_signer(account_info_iter)?;
    let deposit_authority = AccountLoader::next_signer(account_info_iter)?;
    let mining_owner_reward_token_account =
        AccountLoader::next_with_owner(account_info_iter, &spl_token::id())?;
    let _token_program = AccountLoader::next_with_key(account_info_iter, &spl_token::id())?;

    let reward_pool_data = &mut reward_pool.data.borrow_mut();
    let wrapped_reward_pool = WrappedRewardPool::from_bytes_mut(reward_pool_data)?;

    let mining_data = &mut mining.data.borrow_mut();
    let mut wrapped_mining = WrappedMining::from_bytes_mut(mining_data)?;

    assert_account_key(
        deposit_authority,
        &wrapped_reward_pool.pool.deposit_authority,
    )?;

    {
        let mining_user_rewards =
            Account::unpack(&mining_owner_reward_token_account.data.borrow())?;
        if mining_user_rewards.owner != *mining_owner.key {
            msg!(
                "Rewards account is not owned by mining owner. Got {} Expected {}",
                mining_user_rewards.owner,
                mining_owner.key
            );
            return Err(ProgramError::InvalidArgument);
        }
    }

    let reward_pool_seeds = &[
        b"reward_pool".as_ref(),
        &wrapped_reward_pool.pool.deposit_authority.to_bytes(),
        &[wrapped_reward_pool.pool.bump],
    ];

    {
        assert_account_key(mining_owner, &wrapped_mining.mining.owner)?;
        assert_account_key(reward_pool, &wrapped_mining.mining.reward_pool)?;
        assert_account_key(
            reward_pool,
            &Pubkey::create_program_address(reward_pool_seeds, program_id)?,
        )?;

        let vault_seeds = &[
            b"vault".as_ref(),
            &reward_pool.key.to_bytes(),
            &reward_mint.key.to_bytes(),
            &[wrapped_reward_pool.pool.token_account_bump],
        ];
        assert_account_key(
            vault,
            &Pubkey::create_program_address(vault_seeds, program_id)?,
        )?;
    }
    wrapped_mining.refresh_rewards(&wrapped_reward_pool.cumulative_index)?;
    let amount = wrapped_mining.mining.unclaimed_rewards;
    wrapped_mining.mining.claim();

    if amount > 0 {
        spl_transfer(
            vault.clone(),
            mining_owner_reward_token_account.clone(),
            reward_pool.clone(),
            amount,
            &[reward_pool_seeds],
        )?;
    }

    let mut amount_writer = vec![];
    amount.serialize(&mut amount_writer)?;
    set_return_data(&amount_writer);

    Ok(())
}
