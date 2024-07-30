use crate::{
    asserts::assert_account_key,
    state::{Mining, WrappedMining},
    utils::AccountLoader,
};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey, system_program,
};

pub fn process_initialize_mining<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter().enumerate();

    let reward_pool = AccountLoader::next_with_owner(account_info_iter, program_id)?;
    let mining = AccountLoader::next_uninitialized(account_info_iter)?;
    let mining_owner = AccountLoader::next_signer(account_info_iter)?;
    let _system_program = AccountLoader::next_with_key(account_info_iter, &system_program::id())?;

    let mining_pubkey = Pubkey::create_with_seed(&mining_owner.key, "mining", program_id)?;
    assert_account_key(mining, &mining_pubkey)?;

    let mining_data = &mut mining.data.borrow_mut();
    let wrapped_mining = WrappedMining::from_bytes_mut(mining_data)?;
    let mining = Mining::initialize(*reward_pool.key, *mining_owner.key);
    *wrapped_mining.mining = mining;
    wrapped_mining.weighted_stake_diffs.initialize();

    Ok(())
}
