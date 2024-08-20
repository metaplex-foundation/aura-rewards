use crate::{
    asserts::assert_and_get_pool_and_mining,
    utils::{get_delegate_mining, AccountLoader, LockupPeriod},
};
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};
use crate::error::MplxRewardsError;
use crate::state::WrappedMining;
use crate::utils::verify_mining_address;

pub fn process_deposit_mining<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    amount: u64,
    lockup_period: LockupPeriod,
    mining_owner: &Pubkey,
    delegate_wallet_addr: &Pubkey,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter().enumerate();

    let reward_pool = AccountLoader::next_with_owner(account_info_iter, program_id)?;
    let mining = AccountLoader::next_with_owner(account_info_iter, program_id)?;
    let deposit_authority = AccountLoader::next_signer(account_info_iter)?;
    let delegate_mining = AccountLoader::next_with_owner(account_info_iter, program_id)?;

    let mining_data = &mut mining.data.borrow_mut();
    let reward_pool_data = &mut reward_pool.data.borrow_mut();

    let (mut wrapped_reward_pool, mut wrapped_mining) = assert_and_get_pool_and_mining(
        program_id,
        mining_owner,
        mining,
        reward_pool,
        deposit_authority,
        reward_pool_data,
        mining_data,
    )?;

    let delegate_mining = get_delegate_mining(delegate_mining, mining)?;


    if let Some(delegate_mining) = delegate_mining {
        if *delegate_mining.key != verify_mining_address(
            program_id,
            delegate_wallet_addr,
            reward_pool.key,
            WrappedMining::from_bytes_mut(&mut delegate_mining.data.borrow_mut())?.mining.bump
        ).map_err(|_| MplxRewardsError::DerivationError)? {
            return Err(MplxRewardsError::InvalidMining.into())
        };
    }

    wrapped_reward_pool.deposit(&mut wrapped_mining, amount, lockup_period, delegate_mining)?;

    Ok(())
}
