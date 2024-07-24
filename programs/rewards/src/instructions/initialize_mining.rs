use crate::{
    asserts::{assert_account_key, assert_uninitialized},
    state::{Mining, WrappedMining, TREE_MAX_SIZE},
    utils::{find_mining_program_address, AccountLoader},
};
use sokoban::AVLTree;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke_signed,
    program_error::ProgramError, pubkey::Pubkey, rent::Rent, system_instruction, system_program,
    sysvar::Sysvar,
};

/// Instruction context
pub struct InitializeMiningContext<'a, 'b> {
    reward_pool: &'a AccountInfo<'b>,
    mining: &'a AccountInfo<'b>,
    payer: &'a AccountInfo<'b>,
}

impl<'a, 'b> InitializeMiningContext<'a, 'b> {
    /// New instruction context
    pub fn new(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'b>],
    ) -> Result<InitializeMiningContext<'a, 'b>, ProgramError> {
        let account_info_iter = &mut accounts.iter().enumerate();

        let reward_pool = AccountLoader::next_with_owner(account_info_iter, program_id)?;
        let mining = AccountLoader::next_uninitialized(account_info_iter)?;
        let payer = AccountLoader::next_signer(account_info_iter)?;
        let _system_program =
            AccountLoader::next_with_key(account_info_iter, &system_program::id())?;

        Ok(InitializeMiningContext {
            reward_pool,
            mining,
            payer,
        })
    }

    /// Process instruction
    pub fn process(&self, program_id: &Pubkey, mining_owner: &Pubkey) -> ProgramResult {
        assert_uninitialized(self.mining)?;

        let bump = {
            let (pubkey, bump) =
                find_mining_program_address(program_id, mining_owner, self.reward_pool.key);
            assert_account_key(self.mining, &pubkey)?;
            bump
        };

        let signers_seeds = &[
            "mining".as_bytes(),
            &mining_owner.to_bytes(),
            &self.reward_pool.key.to_bytes(),
            &[bump],
        ];

        // TODO: refactor account creation
        let mining_acc_size = Mining::LEN + std::mem::size_of::<AVLTree<u64, u64, TREE_MAX_SIZE>>();
        let rent = Rent::get()?;
        let ix = system_instruction::create_account(
            &self.payer.key,
            &self.mining.key,
            rent.minimum_balance(mining_acc_size),
            (mining_acc_size) as u64,
            program_id,
        );
        invoke_signed(
            &ix,
            &[self.payer.clone(), self.mining.clone()],
            &[signers_seeds],
        )?;

        let mining_data = &mut self.mining.data.borrow_mut();
        let wrapped_mining = WrappedMining::from_bytes_mut(mining_data)?;
        let mining = &mut Mining::initialize(*self.reward_pool.key, bump, *mining_owner);
        *wrapped_mining.mining = *mining;
        wrapped_mining.weighted_stake_diffs.initialize();

        Ok(())
    }
}
