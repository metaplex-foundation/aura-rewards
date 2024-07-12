use crate::{
    asserts::{assert_account_key, verify_delegate_mining_requirements},
    state::{Mining, RewardPool},
    utils::{find_mining_program_address, AccountLoader, LockupPeriod},
};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    program_pack::Pack, pubkey::Pubkey,
};

/// Instruction context
pub struct DepositMiningContext<'a, 'b> {
    reward_pool: &'a AccountInfo<'b>,
    mining: &'a AccountInfo<'b>,
    deposit_authority: &'a AccountInfo<'b>,
    delegate_mining: &'a AccountInfo<'b>,
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
        let delegate_mining = AccountLoader::next_with_owner(account_info_iter, program_id)?;

        Ok(DepositMiningContext {
            reward_pool,
            mining,
            deposit_authority,
            delegate_mining,
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
        let mut reward_pool = RewardPool::unpack(&self.reward_pool.data.borrow())?;
        let mut mining = Mining::unpack(&self.mining.data.borrow())?;

        {
            let (mining_pubkey, _) =
                find_mining_program_address(program_id, mining_owner, self.reward_pool.key);
            assert_account_key(self.mining, &mining_pubkey)?;
            assert_account_key(self.deposit_authority, &reward_pool.deposit_authority)?;
            assert_account_key(self.reward_pool, &mining.reward_pool)?;
            if mining_owner != &mining.owner {
                msg!(
                    "Assert account error. Got {} Expected {}",
                    *mining_owner,
                    mining.owner
                );
                return Err(ProgramError::InvalidArgument);
            }
        }

        let mut delegate_mining =
            verify_delegate_mining_requirements(self.delegate_mining, self.mining)?;
        reward_pool.deposit(&mut mining, amount, lockup_period, delegate_mining.as_mut())?;

        RewardPool::pack(reward_pool, *self.reward_pool.data.borrow_mut())?;
        Mining::pack(mining, *self.mining.data.borrow_mut())?;

        if let Some(delegate_mining) = delegate_mining {
            Mining::pack(delegate_mining, *self.delegate_mining.data.borrow_mut())?;
        }

        Ok(())
    }
}
