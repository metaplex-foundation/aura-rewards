use crate::state::{RewardVault, MAX_REWARDS};
use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::msg;
use solana_program::program_error::ProgramError;
use solana_program::program_pack::{IsInitialized, Pack, Sealed};
use solana_program::pubkey::Pubkey;

/// Deprecated Reward pool
#[derive(Debug, BorshDeserialize, BorshSerialize, BorshSchema, Default)]

pub struct DeprecatedRewardPool {
    /// Anchor id (For Anchor legacy contract compatibility)
    pub anchor_id: [u8; 8],
    /// Rewards root account (ex-Config program account)
    pub rewards_root: Pubkey,
    /// Saved bump for reward pool account
    pub bump: u8,
    /// Liquidity mint
    pub liquidity_mint: Pubkey,
    /// Reward total share
    pub total_share: u64,
    /// A set of all possible rewards that we can get for this pool
    pub vaults: Vec<RewardVault>,
    /// The address responsible for the charge of rewards for users.
    /// It executes deposits on the rewards pools.
    pub deposit_authority: Pubkey,
}

impl Sealed for DeprecatedRewardPool {}
impl Pack for DeprecatedRewardPool {
    const LEN: usize = 8 + (32 + 1 + 32 + 8 + (4 + RewardVault::LEN * MAX_REWARDS) + 32);

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let mut slice = dst;
        self.serialize(&mut slice).unwrap()
    }

    fn unpack_from_slice(src: &[u8]) -> Result<DeprecatedRewardPool, ProgramError> {
        let mut src_mut = src;
        Self::deserialize(&mut src_mut).map_err(|err| {
            msg!("Failed to deserialize");
            msg!("{}", err.to_string());
            ProgramError::InvalidAccountData
        })
    }
}

impl IsInitialized for DeprecatedRewardPool {
    fn is_initialized(&self) -> bool {
        self.rewards_root != Pubkey::default()
    }
}
