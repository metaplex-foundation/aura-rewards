use crate::traits::SolanaAccount;
use crate::{
    asserts::get_delegate_mining,
    error::MplxRewardsError,
    state::{Mining, RewardPool},
    utils::{assert_and_deserialize_pool_and_mining, AccountLoader},
};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

/// Instruction context
pub struct ChangeDelegateContext<'a, 'b> {
    reward_pool: &'a AccountInfo<'b>,
    mining: &'a AccountInfo<'b>,
    deposit_authority: &'a AccountInfo<'b>,
    mining_owner: &'a AccountInfo<'b>,
    old_delegate_mining: &'a AccountInfo<'b>,
    new_delegate_mining: &'a AccountInfo<'b>,
}

impl<'a, 'b> ChangeDelegateContext<'a, 'b> {
    /// New instruction context
    pub fn new(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'b>],
    ) -> Result<ChangeDelegateContext<'a, 'b>, ProgramError> {
        let account_info_iter = &mut accounts.iter().enumerate();

        let reward_pool = AccountLoader::next_with_owner(account_info_iter, program_id)?;
        let mining = AccountLoader::next_with_owner(account_info_iter, program_id)?;
        let deposit_authority = AccountLoader::next_signer(account_info_iter)?;
        let mining_owner = AccountLoader::next_signer(account_info_iter)?;
        let old_delegate_mining = AccountLoader::next_with_owner(account_info_iter, program_id)?;
        let new_delegate_mining = AccountLoader::next_with_owner(account_info_iter, program_id)?;

        Ok(ChangeDelegateContext {
            reward_pool,
            mining,
            deposit_authority,
            mining_owner,
            old_delegate_mining,
            new_delegate_mining,
        })
    }

    /// Process instruction
    #[allow(clippy::too_many_arguments)]
    pub fn process(&self, program_id: &Pubkey, staked_amount: u64) -> ProgramResult {
        if self.new_delegate_mining.key == self.old_delegate_mining.key {
            return Err(MplxRewardsError::DelegatesAreTheSame.into());
        }

        let (mut reward_pool, mut mining) = assert_and_deserialize_pool_and_mining(
            program_id,
            self.mining_owner.key,
            self.reward_pool,
            self.mining,
            self.deposit_authority,
        )?;

        // if new_delegate_mining.is_none that means that new_delegate == self
        let mut new_delegate_mining = get_delegate_mining(self.new_delegate_mining, self.mining)?;
        // if old_delegate_mining.is_none that means that old_delegate == self
        let mut old_delegate_mining = get_delegate_mining(self.old_delegate_mining, self.mining)?;

        reward_pool.change_delegate(
            &mut mining,
            new_delegate_mining.as_mut(),
            old_delegate_mining.as_mut(),
            staked_amount,
        )?;

        reward_pool.save(self.reward_pool)?;
        mining.save(self.mining)?;

        if let Some(new_delegate_mining) = new_delegate_mining {
            new_delegate_mining.save(self.new_delegate_mining)?;
        }

        if let Some(old_delegate_mining) = old_delegate_mining {
            old_delegate_mining.save(self.old_delegate_mining)?;
        }

        Ok(())
    }
}
