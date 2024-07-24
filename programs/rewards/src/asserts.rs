//! Asserts for account verifications
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey, rent::Rent, sysvar::Sysvar,
};

use crate::error::MplxRewardsError;

/// Assert signer.
pub fn assert_signer(account: &AccountInfo) -> ProgramResult {
    if account.is_signer {
        return Ok(());
    }

    Err(ProgramError::MissingRequiredSignature)
}

/// Assert unitilialized
pub fn assert_uninitialized(account: &AccountInfo) -> ProgramResult {
    let AccountInfo {
        lamports,
        data,
        owner,
        ..
    } = account;

    if **lamports.borrow() == 0 && data.borrow().is_empty() && *owner == &Pubkey::default() {
        return Ok(());
    }

    Err(ProgramError::AccountAlreadyInitialized)
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
