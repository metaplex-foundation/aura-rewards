use crate::{
    asserts::assert_account_key,
    error::MplxRewardsError,
    state::{WrappedMining, WrappedRewardPool},
    utils::{AccountLoader, SafeArithmeticOperations},
};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey, system_program,
};

pub fn process_close_mining<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter().enumerate();

    let mining = AccountLoader::next_with_owner(account_info_iter, program_id)?;
    let mining_owner = AccountLoader::next_signer(account_info_iter)?;
    let target_account = AccountLoader::next_with_owner(account_info_iter, &system_program::id())?;
    let deposit_authority = AccountLoader::next_signer(account_info_iter)?;
    let reward_pool = AccountLoader::next_with_owner(account_info_iter, program_id)?;

    {
        let reward_pool_data = &mut reward_pool.data.borrow_mut();
        let wrapped_reward_pool = WrappedRewardPool::from_bytes_mut(reward_pool_data)?;
        assert_account_key(
            deposit_authority,
            &wrapped_reward_pool.pool.deposit_authority,
        )?;

        let mining_data = &mut (*mining.data).borrow_mut();
        let mut wrapped_mining = WrappedMining::from_bytes_mut(mining_data)?;
        assert_account_key(mining_owner, &wrapped_mining.mining.owner)?;
        let mining_pubkey = Pubkey::create_program_address(
            &[
                b"mining".as_ref(),
                wrapped_mining.mining.owner.as_ref(),
                reward_pool.key.as_ref(),
                &[wrapped_mining.mining.bump],
            ],
            program_id,
        )?;
        assert_account_key(mining, &mining_pubkey)?;

        wrapped_mining.refresh_rewards(wrapped_reward_pool.cumulative_index)?;

        if wrapped_mining.mining.stake_from_others > 0 {
            return Err(MplxRewardsError::StakeFromOthersMustBeZero.into());
        }
        if wrapped_mining.mining.unclaimed_rewards != 0 {
            return Err(MplxRewardsError::RewardsMustBeClaimed.into());
        }
    }

    // Snippet from
    // https://github.com/coral-xyz/anchor/blob/master/lang/src/common.rs#L6-L15
    let dest_starting_lamports = target_account.lamports();

    **target_account.lamports.borrow_mut() = dest_starting_lamports.safe_add(mining.lamports())?;
    **mining.lamports.borrow_mut() = 0;
    mining.assign(&system_program::ID);
    mining.realloc(0, false)?;

    Ok(())
}
