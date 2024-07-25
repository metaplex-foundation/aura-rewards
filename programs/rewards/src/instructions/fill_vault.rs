use crate::{
    asserts::assert_account_key,
    error::MplxRewardsError,
    state::WrappedRewardPool,
    utils::{get_curr_unix_ts, spl_transfer, AccountLoader, SafeArithmeticOperations},
};
use solana_program::{
    account_info::AccountInfo, clock::SECONDS_PER_DAY, entrypoint::ProgramResult, pubkey::Pubkey,
};

pub fn process_fill_vault<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    rewards: u64,
    distribution_ends_at: u64,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter().enumerate();

    let reward_pool = AccountLoader::next_with_owner(account_info_iter, program_id)?;
    let reward_mint = AccountLoader::next_with_owner(account_info_iter, &spl_token::id())?;
    let vault = AccountLoader::next_with_owner(account_info_iter, &spl_token::id())?;
    let fill_authority = AccountLoader::next_signer(account_info_iter)?;
    let source_token_account = AccountLoader::next_with_owner(account_info_iter, &spl_token::id())?;
    let _token_program = AccountLoader::next_with_key(account_info_iter, &spl_token::id())?;

    if rewards == 0 {
        return Err(MplxRewardsError::RewardsMustBeGreaterThanZero.into());
    }

    let reward_pool_data = &mut reward_pool.data.borrow_mut();
    let wrapped_reward_pool = WrappedRewardPool::from_bytes_mut(reward_pool_data)?;

    assert_account_key(fill_authority, &wrapped_reward_pool.pool.fill_authority)?;

    {
        let vault_seeds = &[
            b"vault".as_ref(),
            reward_pool.key.as_ref(),
            reward_mint.key.as_ref(),
            &[wrapped_reward_pool.pool.token_account_bump],
        ];
        assert_account_key(
            vault,
            &Pubkey::create_program_address(vault_seeds, program_id)?,
        )?;
    }

    {
        // beginning of the day where distribution_ends_at
        let distribution_ends_at_day_start =
            distribution_ends_at - (distribution_ends_at % SECONDS_PER_DAY);
        let curr_ts = get_curr_unix_ts();
        let beginning_of_the_curr_day = curr_ts - (curr_ts % SECONDS_PER_DAY);
        if distribution_ends_at_day_start < beginning_of_the_curr_day {
            return Err(MplxRewardsError::DistributionInThePast.into());
        }

        let days_diff = distribution_ends_at_day_start
            .safe_sub(wrapped_reward_pool.pool.distribution_ends_at)?;

        wrapped_reward_pool.pool.distribution_ends_at = wrapped_reward_pool
            .pool
            .distribution_ends_at
            .safe_add(days_diff)?;

        wrapped_reward_pool.pool.tokens_available_for_distribution = wrapped_reward_pool
            .pool
            .tokens_available_for_distribution
            .safe_add(rewards)?;
    }

    spl_transfer(
        source_token_account.clone(),
        vault.clone(),
        fill_authority.clone(),
        rewards,
        &[],
    )?;

    Ok(())
}
