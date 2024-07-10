use crate::{
    error::MplxRewardsError,
    state::{Mining, RewardPool, DELEGATE_MINIMAL_OWNED_WEIGHTED_STAKE},
    utils::{assert_and_deserialize_pool_and_mining, AccountLoader, LockupPeriod},
};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    program_pack::Pack, pubkey::Pubkey,
};

/// Instruction context
pub struct DepositMiningContext<'a, 'b> {
    reward_pool: &'a AccountInfo<'b>,
    mining: &'a AccountInfo<'b>,
    deposit_authority: &'a AccountInfo<'b>,
    delegate: &'a AccountInfo<'b>,
}

impl<'a, 'b> DepositMiningContext<'a, 'b> {
    /// New instruction context
    pub fn new(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'b>],
    ) -> Result<DepositMiningContext<'a, 'b>, ProgramError> {
        let account_info_iter = &mut accounts.iter().enumerate();

        let reward_pool = AccountLoader::next_with_owner(account_info_iter, program_id)?;
        let mining = AccountLoader::next_with_owner(account_info_iter, program_id)?;
        let deposit_authority = AccountLoader::next_signer(account_info_iter)?;
        let delegate = AccountLoader::next_with_owner(account_info_iter, program_id)?;

        Ok(DepositMiningContext {
            reward_pool,
            mining,
            deposit_authority,
            delegate,
        })
    }

    /// Process instruction
    pub fn process(
        &self,
        program_id: &Pubkey,
        amount: u64,
        lockup_period: LockupPeriod,
        mining_owner: &Pubkey,
    ) -> ProgramResult {
        let (mut reward_pool, mut mining) = assert_and_deserialize_pool_and_mining(
            program_id,
            mining_owner,
            self.reward_pool,
            self.mining,
            self.deposit_authority,
        )?;

        let mut delegate_mining = if mining.owner != *self.delegate.key {
            let delegate_mining = Mining::unpack(&self.delegate.data.borrow())?;
            if delegate_mining
                .share
                .checked_sub(delegate_mining.stake_from_others)
                .ok_or(MplxRewardsError::MathOverflow)?
                < DELEGATE_MINIMAL_OWNED_WEIGHTED_STAKE
            {
                return Err(MplxRewardsError::InsufficientWeightedStake.into());
            }

            Some(delegate_mining)
        } else {
            None
        };

        reward_pool.deposit(&mut mining, amount, lockup_period, delegate_mining.as_mut())?;

        RewardPool::pack(reward_pool, *self.reward_pool.data.borrow_mut())?;
        Mining::pack(mining, *self.mining.data.borrow_mut())?;

        Ok(())
    }
}
