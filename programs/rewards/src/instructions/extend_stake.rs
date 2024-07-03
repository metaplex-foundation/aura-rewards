use std::cmp::min;

use crate::{
    asserts::assert_account_key,
    state::{Mining, RewardPool},
    traits::SolanaAccount,
    utils::{AccountLoader, LockupPeriod},
};
use solana_program::system_program;
use solana_program::{
    account_info::AccountInfo, clock::SECONDS_PER_DAY, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey,
};

/// Instruction context
pub struct ExtendStakeContext<'a, 'b> {
    reward_pool: &'a AccountInfo<'b>,
    mining: &'a AccountInfo<'b>,
    deposit_authority: &'a AccountInfo<'b>,
    mining_owner: &'a AccountInfo<'b>,
    system_program: &'a AccountInfo<'b>,
}

impl<'a, 'b> ExtendStakeContext<'a, 'b> {
    /// New instruction context
    pub fn new(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'b>],
    ) -> Result<ExtendStakeContext<'a, 'b>, ProgramError> {
        let account_info_iter = &mut accounts.iter().enumerate();

        let reward_pool = AccountLoader::next_with_owner(account_info_iter, program_id)?;
        let mining = AccountLoader::next_with_owner(account_info_iter, program_id)?;
        let deposit_authority = AccountLoader::next_signer(account_info_iter)?;
        let mining_owner = AccountLoader::next_signer(account_info_iter)?;
        let system_program =
            AccountLoader::next_with_key(account_info_iter, &system_program::id())?;

        Ok(ExtendStakeContext {
            reward_pool,
            mining,
            deposit_authority,
            mining_owner,
            system_program,
        })
    }

    /// Process instruction
    #[allow(clippy::too_many_arguments)]
    pub fn process(
        &self,
        program_id: &Pubkey,
        old_lockup_period: LockupPeriod,
        new_lockup_period: LockupPeriod,
        deposit_start_ts: u64,
        base_amount: u64,
        additional_amount: u64,
    ) -> ProgramResult {
        let mut reward_pool = RewardPool::load(self.reward_pool)?;
        let mut mining = Mining::load(self.mining)?;
        let deposit_start_ts = deposit_start_ts - (deposit_start_ts % SECONDS_PER_DAY);

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
                    self.mining_owner.key,
                    mining.owner
                );
                return Err(ProgramError::InvalidArgument);
            }
        }

        reward_pool.extend(
            &mut mining,
            old_lockup_period,
            new_lockup_period,
            deposit_start_ts,
            base_amount,
            additional_amount,
        )?;

        reward_pool.save(self.reward_pool)?;
        mining.save(self.mining)?;

        Ok(())
    }
}
