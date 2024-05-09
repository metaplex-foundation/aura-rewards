use crate::error::MplxRewardsError;
use crate::state::{RewardVault, MAX_REWARDS, PRECISION};
use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::entrypoint::ProgramResult;
use solana_program::msg;
use solana_program::program_error::ProgramError;
use solana_program::program_pack::{IsInitialized, Pack, Sealed};
use solana_program::pubkey::Pubkey;
use std::slice::Iter;

/// Mining
#[derive(Debug, BorshDeserialize, BorshSerialize, BorshSchema, Default)]
pub struct Mining {
    /// Anchor id(For Anchor legacy contract compatibility)
    pub anchor_id: [u8; 8],
    /// Reward pool address
    pub reward_pool: Pubkey,
    /// Saved bump for mining account
    pub bump: u8,
    /// Share
    pub share: u64,
    /// Mining owner
    pub owner: Pubkey,
    /// Reward indexes
    pub indexes: Vec<RewardIndex>,
}

impl Mining {
    /// Initialize a Reward Pool
    pub fn initialize(reward_pool: Pubkey, bump: u8, owner: Pubkey) -> Mining {
        Mining {
            anchor_id: Default::default(),
            reward_pool,
            bump,
            share: 0,
            owner,
            indexes: vec![],
        }
    }

    /// Returns reward index
    pub fn reward_index_mut(&mut self, reward_mint: Pubkey) -> &mut RewardIndex {
        match self
            .indexes
            .iter()
            .position(|mi| mi.reward_mint == reward_mint)
        {
            Some(i) => &mut self.indexes[i],
            None => {
                self.indexes.push(RewardIndex {
                    reward_mint,
                    ..Default::default()
                });
                self.indexes.last_mut().unwrap()
            }
        }
    }

    /// Claim reward
    pub fn claim(&mut self, reward_mint: Pubkey) {
        let reward_index = self.reward_index_mut(reward_mint);
        reward_index.rewards = 0;
    }

    /// Refresh rewards
    pub fn refresh_rewards(&mut self, vaults: Iter<RewardVault>) -> ProgramResult {
        let share = self.share;

        for vault in vaults {
            let reward_index = self.reward_index_mut(vault.reward_mint);

            if vault.index_with_precision > reward_index.index_with_precision {
                let rewards = vault
                    .index_with_precision
                    .checked_sub(reward_index.index_with_precision)
                    .ok_or(MplxRewardsError::MathOverflow)?
                    .checked_mul(share as u128)
                    .ok_or(MplxRewardsError::MathOverflow)?
                    .checked_div(PRECISION)
                    .ok_or(MplxRewardsError::MathOverflow)?;

                if rewards > 0 {
                    reward_index.rewards = reward_index
                        .rewards
                        .checked_add(rewards as u64)
                        .ok_or(MplxRewardsError::MathOverflow)?;
                }
            }

            reward_index.index_with_precision = vault.index_with_precision;
        }

        Ok(())
    }
}

impl Sealed for Mining {}
impl Pack for Mining {
    const LEN: usize = 8 + (32 + 1 + 8 + 32 + (4 + RewardIndex::LEN * MAX_REWARDS));

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let mut slice = dst;
        self.serialize(&mut slice).unwrap()
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let mut src_mut = src;
        Self::deserialize(&mut src_mut).map_err(|err| {
            msg!("Failed to deserialize");
            msg!("{}", err.to_string());
            ProgramError::InvalidAccountData
        })
    }
}

impl IsInitialized for Mining {
    fn is_initialized(&self) -> bool {
        self.owner != Pubkey::default()
    }
}

/// Reward index
#[derive(Debug, BorshSerialize, BorshDeserialize, BorshSchema, Default, Clone)]
pub struct RewardIndex {
    /// Reward mint
    pub reward_mint: Pubkey,
    /// Index with precision
    pub index_with_precision: u128,
    /// Rewards amount
    pub rewards: u64,
}

impl RewardIndex {
    /// 32 + 16 + 8
    pub const LEN: usize = 56;
}
