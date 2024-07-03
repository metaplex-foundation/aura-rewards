//! This code was AUTOGENERATED using the kinobi library.
//! Please DO NOT EDIT THIS FILE, instead use visitors
//! to add features, then rerun kinobi to update it.
//!
//! [https://github.com/metaplex-foundation/kinobi]

use borsh::{BorshDeserialize, BorshSerialize};

/// Accounts.
pub struct FillVault {
    /// The address of the reward pool
    pub reward_pool: solana_program::pubkey::Pubkey,
    /// The address of the reward mint
    pub reward_mint: solana_program::pubkey::Pubkey,
    /// The address of the reward vault
    pub vault: solana_program::pubkey::Pubkey,
    /// The address of the wallet who is responsible for filling pool's vault with rewards
    pub fill_authority: solana_program::pubkey::Pubkey,
    /// The address of the TA from which tokens will be spent
    pub source_token_account: solana_program::pubkey::Pubkey,
    /// The address of the Token program where rewards are minted
    pub token_program: solana_program::pubkey::Pubkey,
}

impl FillVault {
    pub fn instruction(
        &self,
        args: FillVaultInstructionArgs,
    ) -> solana_program::instruction::Instruction {
        self.instruction_with_remaining_accounts(args, &[])
    }
    #[allow(clippy::vec_init_then_push)]
    pub fn instruction_with_remaining_accounts(
        &self,
        args: FillVaultInstructionArgs,
        remaining_accounts: &[solana_program::instruction::AccountMeta],
    ) -> solana_program::instruction::Instruction {
        let mut accounts = Vec::with_capacity(6 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.reward_pool,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.reward_mint,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.vault, false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.fill_authority,
            true,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.source_token_account,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.token_program,
            false,
        ));
        accounts.extend_from_slice(remaining_accounts);
        let mut data = FillVaultInstructionData::new().try_to_vec().unwrap();
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
pub struct FillVaultInstructionData {
    discriminator: u8,
}

impl FillVaultInstructionData {
    pub fn new() -> Self {
        Self { discriminator: 1 }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FillVaultInstructionArgs {
    pub amount: u64,
    pub distribution_ends_at: u64,
}

/// Instruction builder for `FillVault`.
///
/// ### Accounts:
///
///   0. `[writable]` reward_pool
///   1. `[]` reward_mint
///   2. `[writable]` vault
///   3. `[signer]` fill_authority
///   4. `[writable]` source_token_account
///   5. `[optional]` token_program (default to `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA`)
#[derive(Default)]
pub struct FillVaultBuilder {
    reward_pool: Option<solana_program::pubkey::Pubkey>,
    reward_mint: Option<solana_program::pubkey::Pubkey>,
    vault: Option<solana_program::pubkey::Pubkey>,
    fill_authority: Option<solana_program::pubkey::Pubkey>,
    source_token_account: Option<solana_program::pubkey::Pubkey>,
    token_program: Option<solana_program::pubkey::Pubkey>,
    amount: Option<u64>,
    distribution_ends_at: Option<u64>,
    __remaining_accounts: Vec<solana_program::instruction::AccountMeta>,
}

impl FillVaultBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    /// The address of the reward pool
    #[inline(always)]
    pub fn reward_pool(&mut self, reward_pool: solana_program::pubkey::Pubkey) -> &mut Self {
        self.reward_pool = Some(reward_pool);
        self
    }
    /// The address of the reward mint
    #[inline(always)]
    pub fn reward_mint(&mut self, reward_mint: solana_program::pubkey::Pubkey) -> &mut Self {
        self.reward_mint = Some(reward_mint);
        self
    }
    /// The address of the reward vault
    #[inline(always)]
    pub fn vault(&mut self, vault: solana_program::pubkey::Pubkey) -> &mut Self {
        self.vault = Some(vault);
        self
    }
    /// The address of the wallet who is responsible for filling pool's vault with rewards
    #[inline(always)]
    pub fn fill_authority(&mut self, fill_authority: solana_program::pubkey::Pubkey) -> &mut Self {
        self.fill_authority = Some(fill_authority);
        self
    }
    /// The address of the TA from which tokens will be spent
    #[inline(always)]
    pub fn source_token_account(
        &mut self,
        source_token_account: solana_program::pubkey::Pubkey,
    ) -> &mut Self {
        self.source_token_account = Some(source_token_account);
        self
    }
    /// `[optional account, default to 'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA']`
    /// The address of the Token program where rewards are minted
    #[inline(always)]
    pub fn token_program(&mut self, token_program: solana_program::pubkey::Pubkey) -> &mut Self {
        self.token_program = Some(token_program);
        self
    }
    #[inline(always)]
    pub fn amount(&mut self, amount: u64) -> &mut Self {
        self.amount = Some(amount);
        self
    }
    #[inline(always)]
    pub fn distribution_ends_at(&mut self, distribution_ends_at: u64) -> &mut Self {
        self.distribution_ends_at = Some(distribution_ends_at);
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
        let accounts = FillVault {
            reward_pool: self.reward_pool.expect("reward_pool is not set"),
            reward_mint: self.reward_mint.expect("reward_mint is not set"),
            vault: self.vault.expect("vault is not set"),
            fill_authority: self.fill_authority.expect("fill_authority is not set"),
            source_token_account: self
                .source_token_account
                .expect("source_token_account is not set"),
            token_program: self.token_program.unwrap_or(solana_program::pubkey!(
                "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
            )),
        };
        let args = FillVaultInstructionArgs {
            amount: self.amount.clone().expect("amount is not set"),
            distribution_ends_at: self
                .distribution_ends_at
                .clone()
                .expect("distribution_ends_at is not set"),
        };

        accounts.instruction_with_remaining_accounts(args, &self.__remaining_accounts)
    }
}

/// `fill_vault` CPI accounts.
pub struct FillVaultCpiAccounts<'a, 'b> {
    /// The address of the reward pool
    pub reward_pool: &'b solana_program::account_info::AccountInfo<'a>,
    /// The address of the reward mint
    pub reward_mint: &'b solana_program::account_info::AccountInfo<'a>,
    /// The address of the reward vault
    pub vault: &'b solana_program::account_info::AccountInfo<'a>,
    /// The address of the wallet who is responsible for filling pool's vault with rewards
    pub fill_authority: &'b solana_program::account_info::AccountInfo<'a>,
    /// The address of the TA from which tokens will be spent
    pub source_token_account: &'b solana_program::account_info::AccountInfo<'a>,
    /// The address of the Token program where rewards are minted
    pub token_program: &'b solana_program::account_info::AccountInfo<'a>,
}

/// `fill_vault` CPI instruction.
pub struct FillVaultCpi<'a, 'b> {
    /// The program to invoke.
    pub __program: &'b solana_program::account_info::AccountInfo<'a>,
    /// The address of the reward pool
    pub reward_pool: &'b solana_program::account_info::AccountInfo<'a>,
    /// The address of the reward mint
    pub reward_mint: &'b solana_program::account_info::AccountInfo<'a>,
    /// The address of the reward vault
    pub vault: &'b solana_program::account_info::AccountInfo<'a>,
    /// The address of the wallet who is responsible for filling pool's vault with rewards
    pub fill_authority: &'b solana_program::account_info::AccountInfo<'a>,
    /// The address of the TA from which tokens will be spent
    pub source_token_account: &'b solana_program::account_info::AccountInfo<'a>,
    /// The address of the Token program where rewards are minted
    pub token_program: &'b solana_program::account_info::AccountInfo<'a>,
    /// The arguments for the instruction.
    pub __args: FillVaultInstructionArgs,
}

impl<'a, 'b> FillVaultCpi<'a, 'b> {
    pub fn new(
        program: &'b solana_program::account_info::AccountInfo<'a>,
        accounts: FillVaultCpiAccounts<'a, 'b>,
        args: FillVaultInstructionArgs,
    ) -> Self {
        Self {
            __program: program,
            reward_pool: accounts.reward_pool,
            reward_mint: accounts.reward_mint,
            vault: accounts.vault,
            fill_authority: accounts.fill_authority,
            source_token_account: accounts.source_token_account,
            token_program: accounts.token_program,
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
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.reward_mint.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.vault.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.fill_authority.key,
            true,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.source_token_account.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.token_program.key,
            false,
        ));
        remaining_accounts.iter().for_each(|remaining_account| {
            accounts.push(solana_program::instruction::AccountMeta {
                pubkey: *remaining_account.0.key,
                is_signer: remaining_account.1,
                is_writable: remaining_account.2,
            })
        });
        let mut data = FillVaultInstructionData::new().try_to_vec().unwrap();
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
        account_infos.push(self.reward_mint.clone());
        account_infos.push(self.vault.clone());
        account_infos.push(self.fill_authority.clone());
        account_infos.push(self.source_token_account.clone());
        account_infos.push(self.token_program.clone());
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

/// Instruction builder for `FillVault` via CPI.
///
/// ### Accounts:
///
///   0. `[writable]` reward_pool
///   1. `[]` reward_mint
///   2. `[writable]` vault
///   3. `[signer]` fill_authority
///   4. `[writable]` source_token_account
///   5. `[]` token_program
pub struct FillVaultCpiBuilder<'a, 'b> {
    instruction: Box<FillVaultCpiBuilderInstruction<'a, 'b>>,
}

impl<'a, 'b> FillVaultCpiBuilder<'a, 'b> {
    pub fn new(program: &'b solana_program::account_info::AccountInfo<'a>) -> Self {
        let instruction = Box::new(FillVaultCpiBuilderInstruction {
            __program: program,
            reward_pool: None,
            reward_mint: None,
            vault: None,
            fill_authority: None,
            source_token_account: None,
            token_program: None,
            amount: None,
            distribution_ends_at: None,
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
    /// The address of the reward mint
    #[inline(always)]
    pub fn reward_mint(
        &mut self,
        reward_mint: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.reward_mint = Some(reward_mint);
        self
    }
    /// The address of the reward vault
    #[inline(always)]
    pub fn vault(&mut self, vault: &'b solana_program::account_info::AccountInfo<'a>) -> &mut Self {
        self.instruction.vault = Some(vault);
        self
    }
    /// The address of the wallet who is responsible for filling pool's vault with rewards
    #[inline(always)]
    pub fn fill_authority(
        &mut self,
        fill_authority: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.fill_authority = Some(fill_authority);
        self
    }
    /// The address of the TA from which tokens will be spent
    #[inline(always)]
    pub fn source_token_account(
        &mut self,
        source_token_account: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.source_token_account = Some(source_token_account);
        self
    }
    /// The address of the Token program where rewards are minted
    #[inline(always)]
    pub fn token_program(
        &mut self,
        token_program: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.token_program = Some(token_program);
        self
    }
    #[inline(always)]
    pub fn amount(&mut self, amount: u64) -> &mut Self {
        self.instruction.amount = Some(amount);
        self
    }
    #[inline(always)]
    pub fn distribution_ends_at(&mut self, distribution_ends_at: u64) -> &mut Self {
        self.instruction.distribution_ends_at = Some(distribution_ends_at);
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
    /// Each account is represented by a tuple of the `AccountInfo`, a `bool` indicating whether the
    /// account is writable or not, and a `bool` indicating whether the account is a signer or
    /// not.
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
        let args = FillVaultInstructionArgs {
            amount: self.instruction.amount.clone().expect("amount is not set"),
            distribution_ends_at: self
                .instruction
                .distribution_ends_at
                .clone()
                .expect("distribution_ends_at is not set"),
        };
        let instruction = FillVaultCpi {
            __program: self.instruction.__program,

            reward_pool: self
                .instruction
                .reward_pool
                .expect("reward_pool is not set"),

            reward_mint: self
                .instruction
                .reward_mint
                .expect("reward_mint is not set"),

            vault: self.instruction.vault.expect("vault is not set"),

            fill_authority: self
                .instruction
                .fill_authority
                .expect("fill_authority is not set"),

            source_token_account: self
                .instruction
                .source_token_account
                .expect("source_token_account is not set"),

            token_program: self
                .instruction
                .token_program
                .expect("token_program is not set"),
            __args: args,
        };
        instruction.invoke_signed_with_remaining_accounts(
            signers_seeds,
            &self.instruction.__remaining_accounts,
        )
    }
}

struct FillVaultCpiBuilderInstruction<'a, 'b> {
    __program: &'b solana_program::account_info::AccountInfo<'a>,
    reward_pool: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    reward_mint: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    vault: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    fill_authority: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    source_token_account: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    token_program: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    amount: Option<u64>,
    distribution_ends_at: Option<u64>,
    /// Additional instruction accounts `(AccountInfo, is_writable, is_signer)`.
    __remaining_accounts: Vec<(
        &'b solana_program::account_info::AccountInfo<'a>,
        bool,
        bool,
    )>,
}
