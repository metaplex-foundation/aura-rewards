use crate::state::AccountType;
use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::msg;
use solana_program::program_error::ProgramError;
use solana_program::program_pack::{IsInitialized, Pack, Sealed};
use solana_program::pubkey::Pubkey;

/// Rewards Root
#[derive(Debug, BorshDeserialize, BorshSerialize, BorshSchema, Default)]
pub struct RewardsRoot {
    /// Account type - RewardsRoot
    pub account_type: AccountType,
    /// Authority address
    pub authority: Pubkey,
}

impl RewardsRoot {
    /// Init root account
    pub fn init(authority: Pubkey) -> RewardsRoot {
        RewardsRoot {
            account_type: AccountType::RewardsRoot,
            authority,
        }
    }
}

impl Sealed for RewardsRoot {}
impl Pack for RewardsRoot {
    const LEN: usize = 1 + 32 + 32;

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let mut slice = dst;
        self.serialize(&mut slice).unwrap()
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let mut src_mut = src;
        Self::deserialize(&mut src_mut).map_err(|_| {
            msg!("Failed to deserialize");
            msg!("Actual LEN: {}", std::mem::size_of::<RewardsRoot>());
            ProgramError::InvalidAccountData
        })
    }
}

impl IsInitialized for RewardsRoot {
    fn is_initialized(&self) -> bool {
        self.account_type == AccountType::RewardsRoot
    }
}
