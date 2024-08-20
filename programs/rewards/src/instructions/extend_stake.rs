use crate::{
    asserts::assert_and_get_pool_and_mining,
    error::MplxRewardsError,
    state::WrappedMining,
    utils::{get_delegate_mining, AccountLoader, LockupPeriod},
};
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

#[allow(clippy::too_many_arguments)]
pub fn process_extend_stake<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    old_lockup_period: LockupPeriod,
    new_lockup_period: LockupPeriod,
    deposit_start_ts: u64,
    base_amount: u64,
    additional_amount: u64,
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
        if *delegate_mining.key
            != Pubkey::create_program_address(
                &[
                    "mining".as_bytes(),
                    &delegate_wallet_addr.to_bytes(),
                    &reward_pool.key.to_bytes(),
                    &[
                        WrappedMining::from_bytes_mut(&mut delegate_mining.data.borrow_mut())?
                            .mining
                            .bump,
                    ],
                ],
                program_id,
            )
            .map_err(|_| MplxRewardsError::DerivationError)?
        {
            return Err(MplxRewardsError::InvalidMining.into());
        };
    }

    wrapped_reward_pool.extend(
        &mut wrapped_mining,
        old_lockup_period,
        new_lockup_period,
        deposit_start_ts,
        base_amount,
        additional_amount,
        delegate_mining,
    )?;

    Ok(())
}
