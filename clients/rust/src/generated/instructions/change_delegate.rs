//! This code was AUTOGENERATED using the kinobi library.
//! Please DO NOT EDIT THIS FILE, instead use visitors
//! to add features, then rerun kinobi to update it.
//!
//! [https://github.com/metaplex-foundation/kinobi]
//!

use borsh::BorshDeserialize;
use borsh::BorshSerialize;

/// Accounts.
pub struct ChangeDelegate {
    /// The address of the reward pool
    pub reward_pool: solana_program::pubkey::Pubkey,
    /// The address of the mining account which belongs to the user and stores info about user's rewards
    pub mining: solana_program::pubkey::Pubkey,
    /// The address of the Staking program's Registrar, which is PDA and is responsible for signing CPIs
    pub deposit_authority: solana_program::pubkey::Pubkey,
    /// The end user the mining accounts belongs to
    pub mining_owner: solana_program::pubkey::Pubkey,
    /// The address of the old delegate mining account
    pub old_delegate_mining: solana_program::pubkey::Pubkey,
    /// The address of the new delegate mining account
    pub new_delegate_mining: solana_program::pubkey::Pubkey,
}

impl ChangeDelegate {
    pub fn instruction(
        &self,
        args: ChangeDelegateInstructionArgs,
    ) -> solana_program::instruction::Instruction {
        self.instruction_with_remaining_accounts(args, &[])
    }
    #[allow(clippy::vec_init_then_push)]
    pub fn instruction_with_remaining_accounts(
        &self,
        args: ChangeDelegateInstructionArgs,
        remaining_accounts: &[solana_program::instruction::AccountMeta],
    ) -> solana_program::instruction::Instruction {
        let mut accounts = Vec::with_capacity(6 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.reward_pool,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.mining,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.deposit_authority,
            true,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.mining_owner,
            true,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.old_delegate_mining,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.new_delegate_mining,
            false,
        ));
        accounts.extend_from_slice(remaining_accounts);
        let mut data = ChangeDelegateInstructionData::new().try_to_vec().unwrap();
        let mut args = args.try_to_vec().unwrap();
        data.append(&mut args);

        solana_program::instruction::Instruction {
            program_id: crate::MPLX_REWARDS_ID,
            accounts,
            data,
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct ChangeDelegateInstructionData {
    discriminator: u8,
}

impl ChangeDelegateInstructionData {
    pub fn new() -> Self {
        Self { discriminator: 9 }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ChangeDelegateInstructionArgs {
    pub staked_amount: u64,
}

/// Instruction builder for `ChangeDelegate`.
///
/// ### Accounts:
///
///   0. `[writable]` reward_pool
///   1. `[writable]` mining
///   2. `[signer]` deposit_authority
///   3. `[signer]` mining_owner
///   4. `[writable]` old_delegate_mining
///   5. `[writable]` new_delegate_mining
#[derive(Default)]
pub struct ChangeDelegateBuilder {
    reward_pool: Option<solana_program::pubkey::Pubkey>,
    mining: Option<solana_program::pubkey::Pubkey>,
    deposit_authority: Option<solana_program::pubkey::Pubkey>,
    mining_owner: Option<solana_program::pubkey::Pubkey>,
    old_delegate_mining: Option<solana_program::pubkey::Pubkey>,
    new_delegate_mining: Option<solana_program::pubkey::Pubkey>,
    staked_amount: Option<u64>,
    __remaining_accounts: Vec<solana_program::instruction::AccountMeta>,
}

impl ChangeDelegateBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    /// The address of the reward pool
    #[inline(always)]
    pub fn reward_pool(&mut self, reward_pool: solana_program::pubkey::Pubkey) -> &mut Self {
        self.reward_pool = Some(reward_pool);
        self
    }
    /// The address of the mining account which belongs to the user and stores info about user's rewards
    #[inline(always)]
    pub fn mining(&mut self, mining: solana_program::pubkey::Pubkey) -> &mut Self {
        self.mining = Some(mining);
        self
    }
    /// The address of the Staking program's Registrar, which is PDA and is responsible for signing CPIs
    #[inline(always)]
    pub fn deposit_authority(
        &mut self,
        deposit_authority: solana_program::pubkey::Pubkey,
    ) -> &mut Self {
        self.deposit_authority = Some(deposit_authority);
        self
    }
    /// The end user the mining accounts belongs to
    #[inline(always)]
    pub fn mining_owner(&mut self, mining_owner: solana_program::pubkey::Pubkey) -> &mut Self {
        self.mining_owner = Some(mining_owner);
        self
    }
    /// The address of the old delegate mining account
    #[inline(always)]
    pub fn old_delegate_mining(
        &mut self,
        old_delegate_mining: solana_program::pubkey::Pubkey,
    ) -> &mut Self {
        self.old_delegate_mining = Some(old_delegate_mining);
        self
    }
    /// The address of the new delegate mining account
    #[inline(always)]
    pub fn new_delegate_mining(
        &mut self,
        new_delegate_mining: solana_program::pubkey::Pubkey,
    ) -> &mut Self {
        self.new_delegate_mining = Some(new_delegate_mining);
        self
    }
    #[inline(always)]
    pub fn staked_amount(&mut self, staked_amount: u64) -> &mut Self {
        self.staked_amount = Some(staked_amount);
        self
    }
    /// Add an aditional account to the instruction.
    #[inline(always)]
    pub fn add_remaining_account(
        &mut self,
        account: solana_program::instruction::AccountMeta,
    ) -> &mut Self {
        self.__remaining_accounts.push(account);
        self
    }
    /// Add additional accounts to the instruction.
    #[inline(always)]
    pub fn add_remaining_accounts(
        &mut self,
        accounts: &[solana_program::instruction::AccountMeta],
    ) -> &mut Self {
        self.__remaining_accounts.extend_from_slice(accounts);
        self
    }
    #[allow(clippy::clone_on_copy)]
    pub fn instruction(&self) -> solana_program::instruction::Instruction {
        let accounts = ChangeDelegate {
            reward_pool: self.reward_pool.expect("reward_pool is not set"),
            mining: self.mining.expect("mining is not set"),
            deposit_authority: self
                .deposit_authority
                .expect("deposit_authority is not set"),
            mining_owner: self.mining_owner.expect("mining_owner is not set"),
            old_delegate_mining: self
                .old_delegate_mining
                .expect("old_delegate_mining is not set"),
            new_delegate_mining: self
                .new_delegate_mining
                .expect("new_delegate_mining is not set"),
        };
        let args = ChangeDelegateInstructionArgs {
            staked_amount: self
                .staked_amount
                .clone()
                .expect("staked_amount is not set"),
        };

        accounts.instruction_with_remaining_accounts(args, &self.__remaining_accounts)
    }
}

/// `change_delegate` CPI accounts.
pub struct ChangeDelegateCpiAccounts<'a, 'b> {
    /// The address of the reward pool
    pub reward_pool: &'b solana_program::account_info::AccountInfo<'a>,
    /// The address of the mining account which belongs to the user and stores info about user's rewards
    pub mining: &'b solana_program::account_info::AccountInfo<'a>,
    /// The address of the Staking program's Registrar, which is PDA and is responsible for signing CPIs
    pub deposit_authority: &'b solana_program::account_info::AccountInfo<'a>,
    /// The end user the mining accounts belongs to
    pub mining_owner: &'b solana_program::account_info::AccountInfo<'a>,
    /// The address of the old delegate mining account
    pub old_delegate_mining: &'b solana_program::account_info::AccountInfo<'a>,
    /// The address of the new delegate mining account
    pub new_delegate_mining: &'b solana_program::account_info::AccountInfo<'a>,
}

/// `change_delegate` CPI instruction.
pub struct ChangeDelegateCpi<'a, 'b> {
    /// The program to invoke.
    pub __program: &'b solana_program::account_info::AccountInfo<'a>,
    /// The address of the reward pool
    pub reward_pool: &'b solana_program::account_info::AccountInfo<'a>,
    /// The address of the mining account which belongs to the user and stores info about user's rewards
    pub mining: &'b solana_program::account_info::AccountInfo<'a>,
    /// The address of the Staking program's Registrar, which is PDA and is responsible for signing CPIs
    pub deposit_authority: &'b solana_program::account_info::AccountInfo<'a>,
    /// The end user the mining accounts belongs to
    pub mining_owner: &'b solana_program::account_info::AccountInfo<'a>,
    /// The address of the old delegate mining account
    pub old_delegate_mining: &'b solana_program::account_info::AccountInfo<'a>,
    /// The address of the new delegate mining account
    pub new_delegate_mining: &'b solana_program::account_info::AccountInfo<'a>,
    /// The arguments for the instruction.
    pub __args: ChangeDelegateInstructionArgs,
}

impl<'a, 'b> ChangeDelegateCpi<'a, 'b> {
    pub fn new(
        program: &'b solana_program::account_info::AccountInfo<'a>,
        accounts: ChangeDelegateCpiAccounts<'a, 'b>,
        args: ChangeDelegateInstructionArgs,
    ) -> Self {
        Self {
            __program: program,
            reward_pool: accounts.reward_pool,
            mining: accounts.mining,
            deposit_authority: accounts.deposit_authority,
            mining_owner: accounts.mining_owner,
            old_delegate_mining: accounts.old_delegate_mining,
            new_delegate_mining: accounts.new_delegate_mining,
            __args: args,
        }
    }
    #[inline(always)]
    pub fn invoke(&self) -> solana_program::entrypoint::ProgramResult {
        self.invoke_signed_with_remaining_accounts(&[], &[])
    }
    #[inline(always)]
    pub fn invoke_with_remaining_accounts(
        &self,
        remaining_accounts: &[(
            &'b solana_program::account_info::AccountInfo<'a>,
            bool,
            bool,
        )],
    ) -> solana_program::entrypoint::ProgramResult {
        self.invoke_signed_with_remaining_accounts(&[], remaining_accounts)
    }
    #[inline(always)]
    pub fn invoke_signed(
        &self,
        signers_seeds: &[&[&[u8]]],
    ) -> solana_program::entrypoint::ProgramResult {
        self.invoke_signed_with_remaining_accounts(signers_seeds, &[])
    }
    #[allow(clippy::clone_on_copy)]
    #[allow(clippy::vec_init_then_push)]
    pub fn invoke_signed_with_remaining_accounts(
        &self,
        signers_seeds: &[&[&[u8]]],
        remaining_accounts: &[(
            &'b solana_program::account_info::AccountInfo<'a>,
            bool,
            bool,
        )],
    ) -> solana_program::entrypoint::ProgramResult {
        let mut accounts = Vec::with_capacity(6 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.reward_pool.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.mining.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.deposit_authority.key,
            true,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.mining_owner.key,
            true,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.old_delegate_mining.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.new_delegate_mining.key,
            false,
        ));
        remaining_accounts.iter().for_each(|remaining_account| {
            accounts.push(solana_program::instruction::AccountMeta {
                pubkey: *remaining_account.0.key,
                is_signer: remaining_account.1,
                is_writable: remaining_account.2,
            })
        });
        let mut data = ChangeDelegateInstructionData::new().try_to_vec().unwrap();
        let mut args = self.__args.try_to_vec().unwrap();
        data.append(&mut args);

        let instruction = solana_program::instruction::Instruction {
            program_id: crate::MPLX_REWARDS_ID,
            accounts,
            data,
        };
        let mut account_infos = Vec::with_capacity(6 + 1 + remaining_accounts.len());
        account_infos.push(self.__program.clone());
        account_infos.push(self.reward_pool.clone());
        account_infos.push(self.mining.clone());
        account_infos.push(self.deposit_authority.clone());
        account_infos.push(self.mining_owner.clone());
        account_infos.push(self.old_delegate_mining.clone());
        account_infos.push(self.new_delegate_mining.clone());
        remaining_accounts
            .iter()
            .for_each(|remaining_account| account_infos.push(remaining_account.0.clone()));

        if signers_seeds.is_empty() {
            solana_program::program::invoke(&instruction, &account_infos)
        } else {
            solana_program::program::invoke_signed(&instruction, &account_infos, signers_seeds)
        }
    }
}

/// Instruction builder for `ChangeDelegate` via CPI.
///
/// ### Accounts:
///
///   0. `[writable]` reward_pool
///   1. `[writable]` mining
///   2. `[signer]` deposit_authority
///   3. `[signer]` mining_owner
///   4. `[writable]` old_delegate_mining
///   5. `[writable]` new_delegate_mining
pub struct ChangeDelegateCpiBuilder<'a, 'b> {
    instruction: Box<ChangeDelegateCpiBuilderInstruction<'a, 'b>>,
}

impl<'a, 'b> ChangeDelegateCpiBuilder<'a, 'b> {
    pub fn new(program: &'b solana_program::account_info::AccountInfo<'a>) -> Self {
        let instruction = Box::new(ChangeDelegateCpiBuilderInstruction {
            __program: program,
            reward_pool: None,
            mining: None,
            deposit_authority: None,
            mining_owner: None,
            old_delegate_mining: None,
            new_delegate_mining: None,
            staked_amount: None,
            __remaining_accounts: Vec::new(),
        });
        Self { instruction }
    }
    /// The address of the reward pool
    #[inline(always)]
    pub fn reward_pool(
        &mut self,
        reward_pool: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.reward_pool = Some(reward_pool);
        self
    }
    /// The address of the mining account which belongs to the user and stores info about user's rewards
    #[inline(always)]
    pub fn mining(
        &mut self,
        mining: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.mining = Some(mining);
        self
    }
    /// The address of the Staking program's Registrar, which is PDA and is responsible for signing CPIs
    #[inline(always)]
    pub fn deposit_authority(
        &mut self,
        deposit_authority: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.deposit_authority = Some(deposit_authority);
        self
    }
    /// The end user the mining accounts belongs to
    #[inline(always)]
    pub fn mining_owner(
        &mut self,
        mining_owner: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.mining_owner = Some(mining_owner);
        self
    }
    /// The address of the old delegate mining account
    #[inline(always)]
    pub fn old_delegate_mining(
        &mut self,
        old_delegate_mining: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.old_delegate_mining = Some(old_delegate_mining);
        self
    }
    /// The address of the new delegate mining account
    #[inline(always)]
    pub fn new_delegate_mining(
        &mut self,
        new_delegate_mining: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.new_delegate_mining = Some(new_delegate_mining);
        self
    }
    #[inline(always)]
    pub fn staked_amount(&mut self, staked_amount: u64) -> &mut Self {
        self.instruction.staked_amount = Some(staked_amount);
        self
    }
    /// Add an additional account to the instruction.
    #[inline(always)]
    pub fn add_remaining_account(
        &mut self,
        account: &'b solana_program::account_info::AccountInfo<'a>,
        is_writable: bool,
        is_signer: bool,
    ) -> &mut Self {
        self.instruction
            .__remaining_accounts
            .push((account, is_writable, is_signer));
        self
    }
    /// Add additional accounts to the instruction.
    ///
    /// Each account is represented by a tuple of the `AccountInfo`, a `bool` indicating whether the account is writable or not,
    /// and a `bool` indicating whether the account is a signer or not.
    #[inline(always)]
    pub fn add_remaining_accounts(
        &mut self,
        accounts: &[(
            &'b solana_program::account_info::AccountInfo<'a>,
            bool,
            bool,
        )],
    ) -> &mut Self {
        self.instruction
            .__remaining_accounts
            .extend_from_slice(accounts);
        self
    }
    #[inline(always)]
    pub fn invoke(&self) -> solana_program::entrypoint::ProgramResult {
        self.invoke_signed(&[])
    }
    #[allow(clippy::clone_on_copy)]
    #[allow(clippy::vec_init_then_push)]
    pub fn invoke_signed(
        &self,
        signers_seeds: &[&[&[u8]]],
    ) -> solana_program::entrypoint::ProgramResult {
        let args = ChangeDelegateInstructionArgs {
            staked_amount: self
                .instruction
                .staked_amount
                .clone()
                .expect("staked_amount is not set"),
        };
        let instruction = ChangeDelegateCpi {
            __program: self.instruction.__program,

            reward_pool: self
                .instruction
                .reward_pool
                .expect("reward_pool is not set"),

            mining: self.instruction.mining.expect("mining is not set"),

            deposit_authority: self
                .instruction
                .deposit_authority
                .expect("deposit_authority is not set"),

            mining_owner: self
                .instruction
                .mining_owner
                .expect("mining_owner is not set"),

            old_delegate_mining: self
                .instruction
                .old_delegate_mining
                .expect("old_delegate_mining is not set"),

            new_delegate_mining: self
                .instruction
                .new_delegate_mining
                .expect("new_delegate_mining is not set"),
            __args: args,
        };
        instruction.invoke_signed_with_remaining_accounts(
            signers_seeds,
            &self.instruction.__remaining_accounts,
        )
    }
}

struct ChangeDelegateCpiBuilderInstruction<'a, 'b> {
    __program: &'b solana_program::account_info::AccountInfo<'a>,
    reward_pool: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    mining: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    deposit_authority: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    mining_owner: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    old_delegate_mining: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    new_delegate_mining: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    staked_amount: Option<u64>,
    /// Additional instruction accounts `(AccountInfo, is_writable, is_signer)`.
    __remaining_accounts: Vec<(
        &'b solana_program::account_info::AccountInfo<'a>,
        bool,
        bool,
    )>,
}