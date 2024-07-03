use crate::{
    asserts::assert_account_key,
    error::MplxRewardsError,
    state::{Mining, RewardPool},
    traits::SolanaAccount,
    utils::AccountLoader,
};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey, system_program,
};

/// Instruction context
pub struct CloseMiningContext<'a, 'b> {
    mining: &'a AccountInfo<'b>,
    mining_owner: &'a AccountInfo<'b>,
    target_account: &'a AccountInfo<'b>,
    deposit_authority: &'a AccountInfo<'b>,
    reward_pool: &'a AccountInfo<'b>,
}

impl<'a, 'b> CloseMiningContext<'a, 'b> {
    /// New instruction context
    pub fn new(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'b>],
    ) -> Result<CloseMiningContext<'a, 'b>, ProgramError> {
        let account_info_iter = &mut accounts.iter().enumerate();

        let mining = AccountLoader::next_with_owner(account_info_iter, program_id)?;
        let mining_owner = AccountLoader::next_signer(account_info_iter)?;
        let target_account =
            AccountLoader::next_with_owner(account_info_iter, &system_program::id())?;
        let deposit_authority = AccountLoader::next_signer(account_info_iter)?;
        let reward_pool = AccountLoader::next_with_owner(account_info_iter, program_id)?;

        Ok(CloseMiningContext {
            mining,
            mining_owner,
            target_account,
            deposit_authority,
            reward_pool,
        })
    }

    /// Process instruction
    pub fn process(&self) -> ProgramResult {
        let reward_pool = RewardPool::load(self.reward_pool)?;
        assert_account_key(self.deposit_authority, &reward_pool.deposit_authority)?;

        let mining = Mining::load(self.mining)?;
        assert_account_key(self.mining_owner, &mining.owner)?;

        if mining.index.unclaimed_rewards != 0 {
            return Err(MplxRewardsError::RewardsMustBeClaimed.into());
        }

        let dest_starting_lamports = self.target_account.lamports();

        **self.target_account.lamports.borrow_mut() = dest_starting_lamports
            .checked_add(self.mining.lamports())
            .ok_or(MplxRewardsError::MathOverflow)?;
        **self.mining.lamports.borrow_mut() = 0;

        let mut source_data = self.mining.data.borrow_mut();
        source_data.fill(0);

        Ok(())
    }
}
