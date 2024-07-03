use crate::{
    asserts::assert_account_key,
    state::{Mining, RewardPool},
    traits::SolanaAccount,
    utils::AccountLoader,
};

use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

/// Instruction context
pub struct WithdrawMiningContext<'a, 'b> {
    reward_pool: &'a AccountInfo<'b>,
    mining: &'a AccountInfo<'b>,
    deposit_authority: &'a AccountInfo<'b>,
}

impl<'a, 'b> WithdrawMiningContext<'a, 'b> {
    /// New instruction context
    pub fn new(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'b>],
    ) -> Result<WithdrawMiningContext<'a, 'b>, ProgramError> {
        let account_info_iter = &mut accounts.iter().enumerate();

        let reward_pool = AccountLoader::next_with_owner(account_info_iter, program_id)?;
        let mining = AccountLoader::next_with_owner(account_info_iter, program_id)?;
        let deposit_authority = AccountLoader::next_signer(account_info_iter)?;

        Ok(WithdrawMiningContext {
            reward_pool,
            mining,
            deposit_authority,
        })
    }

    /// Process instruction
    pub fn process(
        &self,
        program_id: &Pubkey,
        amount: u64,
        mining_owner: &Pubkey,
    ) -> ProgramResult {
        let mut reward_pool = RewardPool::load(&self.reward_pool)?;
        let mut mining = Mining::load(&self.mining)?;

        let mining_pubkey = Pubkey::create_program_address(
            &[
                b"mining".as_ref(),
                mining_owner.as_ref(),
                self.reward_pool.key.as_ref(),
                &[mining.bump],
            ],
            program_id,
        )?;
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
        reward_pool.withdraw(&mut mining, amount)?;

        reward_pool.save(self.reward_pool);
        mining.save(self.mining);

        Ok(())
    }
}
