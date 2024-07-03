use crate::{
    asserts::assert_account_key,
    state::{Mining, RewardPool},
    traits::SolanaAccount,
    utils::{AccountLoader, LockupPeriod},
};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey, system_program,
};

/// Instruction context
pub struct DepositMiningContext<'a, 'b> {
    reward_pool: &'a AccountInfo<'b>,
    mining: &'a AccountInfo<'b>,
    deposit_authority: &'a AccountInfo<'b>,
    mining_owner: &'a AccountInfo<'b>,
    system_program: &'a AccountInfo<'b>,
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
        let mining_owner = AccountLoader::next_signer(account_info_iter)?;
        let system_program =
            AccountLoader::next_with_key(account_info_iter, &system_program::id())?;

        Ok(DepositMiningContext {
            reward_pool,
            mining,
            deposit_authority,
            mining_owner,
            system_program,
        })
    }

    /// Process instruction
    pub fn process(
        &self,
        program_id: &Pubkey,
        amount: u64,
        lockup_period: LockupPeriod,
    ) -> ProgramResult {
        let mut reward_pool = RewardPool::load(self.reward_pool)?;
        let mut mining = Mining::load(self.mining)?;

        reward_pool.resize_if_needed(self.reward_pool, self.mining_owner, self.system_program)?;
        mining.resize_if_needed(self.mining, self.mining_owner, self.system_program)?;

        {
            let mining_pubkey = Pubkey::create_program_address(
                &[
                    b"mining".as_ref(),
                    self.mining_owner.key.as_ref(),
                    self.reward_pool.key.as_ref(),
                    &[mining.bump],
                ],
                program_id,
            )?;
            assert_account_key(self.mining, &mining_pubkey)?;
            assert_account_key(self.deposit_authority, &reward_pool.deposit_authority)?;
            assert_account_key(self.reward_pool, &mining.reward_pool)?;
            if self.mining_owner.key != &mining.owner {
                msg!(
                    "Assert account error. Got {} Expected {}",
                    *self.mining_owner.key,
                    mining.owner
                );
                return Err(ProgramError::InvalidArgument);
            }
        }

        if reward_pool.calculator.weighted_stake_diffs.len()
            % RewardCalculator::WEIGHTED_STAKE_DIFFS_DEFAULT_ELEMENTS_NUMBER
            == 0
            && !reward_pool.calculator.weighted_stake_diffs.is_empty()
        {
            let new_size = self.reward_pool.data_len()
                + reward_pool.calculator.weighted_stake_diffs.len()
                    / RewardCalculator::WEIGHTED_STAKE_DIFFS_DEFAULT_ELEMENTS_NUMBER
                + 1;
            resize_or_reallocate_account(
                self.reward_pool,
                self.mining_owner,
                self.system_program,
                new_size,
            )?;
        }

        reward_pool.deposit(&mut mining, amount, lockup_period)?;

        reward_pool.save(self.reward_pool)?;
        mining.save(self.mining)?;

        Ok(())
    }
}
