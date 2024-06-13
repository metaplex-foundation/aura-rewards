use crate::id;
use crate::state::{InitRewardPoolParams, RewardPool, RewardsRoot};
use crate::utils::{
    assert_account_key, create_account, find_reward_pool_program_address, AccountLoader,
};
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint_deprecated::ProgramResult;
use solana_program::program_error::ProgramError;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;

/// Instruction context
pub struct InitializePoolContext<'a, 'b> {
    rewards_root: &'a AccountInfo<'b>,
    reward_pool: &'a AccountInfo<'b>,
    deposit_authority: &'a AccountInfo<'b>,
    payer: &'a AccountInfo<'b>,
}

impl<'a, 'b> InitializePoolContext<'a, 'b> {
    /// New instruction context
    pub fn new(
        _program_id: &Pubkey,
        accounts: &'a [AccountInfo<'b>],
    ) -> Result<InitializePoolContext<'a, 'b>, ProgramError> {
        let account_info_iter = &mut accounts.iter().enumerate();

        let rewards_root = AccountLoader::next_with_owner(account_info_iter, &id())?;
        let reward_pool = AccountLoader::next_uninitialized(account_info_iter)?;
        let deposit_authority = AccountLoader::next_unchecked(account_info_iter)?;
        let payer = AccountLoader::next_signer(account_info_iter)?;

        Ok(InitializePoolContext {
            rewards_root,
            reward_pool,
            deposit_authority,
            payer,
        })
    }

    /// Process instruction
    pub fn process(&self, program_id: &Pubkey) -> ProgramResult {
        let bump = {
            let (reward_pool_pubkey, bump) =
                find_reward_pool_program_address(program_id, self.rewards_root.key);
            assert_account_key(self.reward_pool, &reward_pool_pubkey)?;
            bump
        };

        {
            let rewards_root = RewardsRoot::unpack(&self.rewards_root.data.borrow())?;
            assert_account_key(self.payer, &rewards_root.authority)?;
        }

        let reward_pool_seeds = &[
            "reward_pool".as_bytes(),
            self.rewards_root.key.as_ref(),
            &[bump],
        ];

        create_account::<RewardPool>(
            program_id,
            self.payer.clone(),
            self.reward_pool.clone(),
            &[reward_pool_seeds],
        )?;

        let reward_pool = RewardPool::init(InitRewardPoolParams {
            rewards_root: *self.rewards_root.key,
            bump,
            deposit_authority: *self.deposit_authority.key,
        });
        RewardPool::pack(reward_pool, *self.reward_pool.data.borrow_mut())?;

        Ok(())
    }
}
