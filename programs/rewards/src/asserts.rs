//! Asserts for account verifications
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey, rent::Rent, sysvar::Sysvar,
};

use crate::{
    error::MplxRewardsError,
    state::{WrappedMining, WrappedRewardPool},
};

/// Assert signer.
pub fn assert_signer(account: &AccountInfo) -> ProgramResult {
    if account.is_signer {
        return Ok(());
    }

    Err(ProgramError::MissingRequiredSignature)
}

/// Assert owned by
pub fn assert_owned_by(account: &AccountInfo, owner: &Pubkey) -> ProgramResult {
    if account.owner == owner {
        Ok(())
    } else {
        msg!(
            "Assert {} owner error. Got {} Expected {}",
            *account.key,
            *account.owner,
            *owner
        );
        Err(MplxRewardsError::InvalidAccountOwner.into())
    }
}

/// Assert account key
pub fn assert_account_key(account_info: &AccountInfo, key: &Pubkey) -> ProgramResult {
    if *account_info.key == *key {
        Ok(())
    } else {
        msg!(
            "Assert account error. Got {} Expected {}",
            *account_info.key,
            *key
        );
        Err(ProgramError::InvalidArgument)
    }
}

/// Assert rent exempt
pub fn assert_rent_exempt(account_info: &AccountInfo) -> ProgramResult {
    let rent = Rent::get()?;

    if rent.is_exempt(account_info.lamports(), account_info.data_len()) {
        Ok(())
    } else {
        msg!(&rent.minimum_balance(account_info.data_len()).to_string());
        Err(ProgramError::AccountNotRentExempt)
    }
}

pub fn assert_pubkey_eq(given: &Pubkey, expected: &Pubkey) -> ProgramResult {
    if given == expected {
        Ok(())
    } else {
        msg!(
            "Assert account error. Got {} Expected {}",
            *given,
            *expected
        );
        Err(ProgramError::InvalidArgument)
    }
}

pub fn assert_account_len(account: &AccountInfo, len: usize) -> ProgramResult {
    if account.data_len() == len {
        Ok(())
    } else {
        msg!(
            "Assert account len error. Got {} Expected {}",
            account.data_len(),
            len
        );
        Err(ProgramError::InvalidArgument)
    }
}

pub fn assert_account_owner(account: &AccountInfo, owner: &Pubkey) -> ProgramResult {
    if account.owner == owner {
        Ok(())
    } else {
        msg!(
            "Assert account owner error. Got {} Expected {}",
            account.owner,
            owner
        );
        Err(ProgramError::InvalidArgument)
    }
}

pub fn assert_and_get_pool_and_mining<'a>(
    program_id: &Pubkey,
    mining_owner: &Pubkey,
    mining: &AccountInfo,
    reward_pool: &AccountInfo,
    deposit_authority: &AccountInfo,
    reward_pool_data: &'a mut [u8],
    mining_data: &'a mut [u8],
) -> Result<(WrappedRewardPool<'a>, WrappedMining<'a>), ProgramError> {
    let wrapped_mining = WrappedMining::from_bytes_mut(mining_data)?;
    let wrapped_reward_pool = WrappedRewardPool::from_bytes_mut(reward_pool_data)?;
    let mining_pubkey = Pubkey::create_program_address(
        &[
            b"mining".as_ref(),
            mining_owner.as_ref(),
            reward_pool.key.as_ref(),
            &[wrapped_mining.mining.bump()],
        ],
        program_id,
    )?;

    assert_account_key(mining, &mining_pubkey)?;
    assert_account_key(
        deposit_authority,
        &wrapped_reward_pool.pool.deposit_authority,
    )?;
    assert_account_key(reward_pool, &wrapped_mining.mining.reward_pool)?;

    if mining_owner != &wrapped_mining.mining.owner {
        msg!(
            "Assert account error. Got {} Expected {}",
            mining_owner,
            wrapped_mining.mining.owner
        );

        return Err(ProgramError::InvalidArgument);
    }

    Ok((wrapped_reward_pool, wrapped_mining))
}
