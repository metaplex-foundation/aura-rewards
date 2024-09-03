use crate::{asserts::assert_and_get_pool_and_mining, utils::AccountLoader};
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

pub fn process_restrict_batch_minting<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    restrict_batch_minting_until_ts: u64,
    mining_owner: &Pubkey,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter().enumerate();

    let deposit_authority = AccountLoader::next_signer(account_info_iter)?;
    let reward_pool = AccountLoader::next_with_owner(account_info_iter, program_id)?;
    let mining = AccountLoader::next_with_owner(account_info_iter, program_id)?;

    let reward_pool_data = &mut reward_pool.data.borrow_mut();
    let mining_data = &mut mining.data.borrow_mut();

    let (_, wrapped_mining) = assert_and_get_pool_and_mining(
        program_id,
        mining_owner,
        mining,
        reward_pool,
        deposit_authority,
        reward_pool_data,
        mining_data,
    )?;

    wrapped_mining.mining.batch_minting_restricted_until = restrict_batch_minting_until_ts;

    Ok(())
}
