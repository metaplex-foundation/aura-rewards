use borsh::{BorshDeserialize, BorshSerialize};
use num_traits::FromPrimitive;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
};

use crate::{error::MplxRewardsError, state::AccountType};

/// A trait for generic blobs of data that have size.
pub trait DataBlob: BorshSerialize + BorshDeserialize {
    /// Get the size of an empty instance of the data blob.
    fn get_initial_size() -> usize;
    /// Get the current size of the data blob.
    fn get_size(&self) -> usize;
}

/// A trait for Solana accounts.
pub trait SolanaAccount: BorshSerialize + BorshDeserialize {
    /// Get the discriminator key for the account.
    fn account_type() -> AccountType;

    /// Load the account from the given account info.
    fn load(account: &AccountInfo) -> Result<Self, ProgramError> {
        let key = load_account_type(account)?;

        if key != Self::account_type() {
            return Err(MplxRewardsError::DeserializationError.into());
        }

        let mut bytes: &[u8] = &(*account.data).borrow()[..];
        Self::deserialize(&mut bytes).map_err(|error| {
            msg!("Error: {}", error);
            MplxRewardsError::DeserializationError.into()
        })
    }

    /// Save the account to the given account info starting at the offset.
    fn save(&self, account: &AccountInfo) -> ProgramResult {
        borsh::to_writer(&mut account.data.borrow_mut()[..], self).map_err(|error| {
            msg!("Error: {}", error);
            MplxRewardsError::SerializationError.into()
        })
    }
}

/// Load the one byte key from the account data at the given offset.
pub fn load_account_type(account: &AccountInfo) -> Result<AccountType, ProgramError> {
    let offset = 0;
    let key = AccountType::from_u8((*account.data).borrow()[offset])
        .ok_or(MplxRewardsError::DeserializationError)?;

    Ok(key)
}