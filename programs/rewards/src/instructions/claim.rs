use crate::{
    asserts::{assert_account_key, assert_account_owner},
    error::MplxRewardsError,
    state::{WrappedMining, WrappedRewardPool},
    utils::{spl_transfer, AccountLoader},
};
use borsh::BorshSerialize;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::set_return_data,
    program_pack::Pack, pubkey::Pubkey,
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

    {
        let mining_user_rewards =
            Account::unpack(&mining_owner_reward_token_account.data.borrow())?;
        assert_account_key(mining_owner, &mining_user_rewards.owner)?;
    }

    let amount = {
        let reward_pool_data = &mut reward_pool.data.borrow_mut();
        let wrapped_reward_pool = WrappedRewardPool::from_bytes_mut(reward_pool_data)?;

        assert_account_key(
            deposit_authority,
            &wrapped_reward_pool.pool.deposit_authority,
        )?;

        let amount = {
            let mining_data = &mut mining.data.borrow_mut();
            let mut wrapped_mining = WrappedMining::from_bytes_mut(mining_data)?;

            if wrapped_mining.mining.is_tokenflow_restricted() {
                return Err(MplxRewardsError::ClaimingRestricted.into());
            }

            assert_account_owner(reward_pool, program_id)?;
            assert_account_key(mining_owner, &wrapped_mining.mining.owner)?;
            assert_account_key(reward_pool, &wrapped_mining.mining.reward_pool)?;

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

            wrapped_mining.refresh_rewards(&*wrapped_reward_pool.cumulative_index)?;
            let amount = wrapped_mining.mining.unclaimed_rewards;
            wrapped_mining.mining.claim();
            amount
        };

        amount
    };

    if amount > 0 {
        spl_transfer(
            vault.to_owned(),
            mining_owner_reward_token_account.to_owned(),
            deposit_authority.to_owned(),
            amount,
            &[],
        )?;
    }

    let mut amount_writer = vec![];
    amount.serialize(&mut amount_writer)?;
    set_return_data(&amount_writer);

    Ok(())
}
