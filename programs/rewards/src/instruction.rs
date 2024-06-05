//! Instruction types

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::pubkey::Pubkey;
use solana_program::{system_program, sysvar};

use crate::utils::LockupPeriod;

/// Instructions supported by the program
#[derive(Debug, BorshDeserialize, BorshSerialize, PartialEq, Eq)]
pub enum RewardsInstruction {
    /// Creates and initializes a reward pool account
    ///
    /// Accounts:
    /// [R] Root account
    /// [W] Reward pool account
    /// [WS] Payer
    /// [R] System program
    InitializePool {
        /// Account responsible for charging users
        deposit_authority: Pubkey,
        /// Account can fill the reward vault
        fill_authority: Pubkey,
    },

    /// Creates a new vault account and adds it to the reward pool
    ///
    /// Accounts:
    /// [R] Root account
    /// [W] Reward pool account
    /// [R] Reward mint account
    /// [W] Vault account
    /// [WS] Payer
    /// [R] Token program
    /// [R] System program
    /// [R] Rent sysvar
    AddVault,

    /// Fills the reward pool with rewards
    ///
    /// Accounts:
    /// [W] Reward pool account
    /// [R] Mint of rewards account
    /// [W] Vault for rewards account
    /// [RS] Transfer  account
    /// [W] From account
    /// [R] Token program
    FillVault {
        /// Amount to fill
        amount: u64,
        /// Rewards distribution ends at given date
        distribution_ends_at: u64,
    },

    /// Initializes mining account for the specified user
    ///
    /// Accounts:
    /// [W] Reward pool account
    /// [W] Mining
    /// [R] User
    /// [WS] Payer
    /// [R] System program
    InitializeMining,

    /// Deposits amount of supply to the mining account
    ///
    /// Accounts:
    /// [W] Reward pool account
    /// [W] Mining
    /// [R] Mint of rewards account
    /// [RS] Deposit authority
    DepositMining {
        /// Amount to deposit
        amount: u64,
        /// Lockup Period
        lockup_period: LockupPeriod,
        /// Specifies mint addr
        reward_mint_addr: Pubkey,
        /// Specifies the owner of the Mining Account
        owner: Pubkey,
    },

    /// Withdraws amount of supply to the mining account
    ///
    /// Accounts:
    /// [W] Reward pool account
    /// [W] Mining
    /// [R] User
    /// [RS] Deposit authority
    WithdrawMining {
        /// Amount to withdraw
        amount: u64,
        /// Specifies the owner of the Mining Account
        owner: Pubkey,
    },

    /// Claims amount of rewards
    ///
    /// Accounts:
    /// [R] Reward pool account
    /// [R] Mint of rewards account
    /// [W] Vault for rewards account
    /// [W] Mining
    /// [RS] User
    /// [W] User reward token account
    /// [R] Token program
    Claim,

    /// Creates and initializes a reward root
    ///
    /// Accounts:
    /// [WS] Root account
    /// [WS] Authority
    /// [R] System program
    InitializeRoot,

    /// Restakes deposit
    ///
    /// Accounts:
    /// [W] Reward pool account
    /// [W] Mining
    /// [R] Mint of rewards account
    /// [R] User
    /// [RS] Deposit authority
    RestakeDeposit {
        /// Requested lockup period for restaking
        lockup_period: LockupPeriod,
        /// Amount of tokens to be restaked
        amount: u64,
        /// Deposit start_ts
        deposit_start_ts: u64,
    },

    /// Fills the reward pool with rewards
    ///
    /// Accounts:
    /// [W] Reward pool account
    /// [R] Mint of rewards account
    /// [W] Vault for rewards account
    /// [RS] Distribute rewards authority
    DistributeRewards,
}

/// Creates 'InitializePool' instruction.
pub fn initialize_pool(
    program_id: &Pubkey,
    root_account: &Pubkey,
    reward_pool: &Pubkey,
    payer: &Pubkey,
    deposit_authority: &Pubkey,
    fill_authority: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*root_account, false),
        AccountMeta::new(*reward_pool, false),
        AccountMeta::new(*payer, true),
        AccountMeta::new_readonly(system_program::id(), false),
    ];

    Instruction::new_with_borsh(
        *program_id,
        &RewardsInstruction::InitializePool {
            deposit_authority: *deposit_authority,
            fill_authority: *fill_authority,
        },
        accounts,
    )
}

/// Creates 'AddVault' instruction.
pub fn add_vault(
    program_id: &Pubkey,
    rewards_root: &Pubkey,
    reward_pool: &Pubkey,
    reward_mint: &Pubkey,
    vault: &Pubkey,
    payer: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*rewards_root, false),
        AccountMeta::new(*reward_pool, false),
        AccountMeta::new_readonly(*reward_mint, false),
        AccountMeta::new(*vault, false),
        AccountMeta::new(*payer, true),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];

    Instruction::new_with_borsh(*program_id, &RewardsInstruction::AddVault, accounts)
}

/// Creates 'FillVault' instruction.
#[allow(clippy::too_many_arguments)]
pub fn fill_vault(
    program_id: &Pubkey,
    reward_pool: &Pubkey,
    reward_mint: &Pubkey,
    vault: &Pubkey,
    authority: &Pubkey,
    from: &Pubkey,
    amount: u64,
    distribution_ends_at: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*reward_pool, false),
        AccountMeta::new_readonly(*reward_mint, false),
        AccountMeta::new(*vault, false),
        AccountMeta::new_readonly(*authority, true),
        AccountMeta::new(*from, false),
        AccountMeta::new_readonly(spl_token::id(), false),
    ];

    Instruction::new_with_borsh(
        *program_id,
        &RewardsInstruction::FillVault {
            amount,
            distribution_ends_at,
        },
        accounts,
    )
}

/// Creates 'InitializeMining' instruction.
pub fn initialize_mining(
    program_id: &Pubkey,
    reward_pool: &Pubkey,
    mining: &Pubkey,
    user: &Pubkey,
    payer: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*reward_pool, false),
        AccountMeta::new(*mining, false),
        AccountMeta::new_readonly(*user, false),
        AccountMeta::new(*payer, true),
        AccountMeta::new_readonly(system_program::id(), false),
    ];

    Instruction::new_with_borsh(*program_id, &RewardsInstruction::InitializeMining, accounts)
}

/// Creates 'DepositMining' instruction.
#[allow(clippy::too_many_arguments)]
pub fn deposit_mining(
    program_id: &Pubkey,
    reward_pool: &Pubkey,
    mining: &Pubkey,
    deposit_authority: &Pubkey,
    amount: u64,
    lockup_period: LockupPeriod,
    reward_mint_addr: &Pubkey,
    owner: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*reward_pool, false),
        AccountMeta::new(*mining, false),
        AccountMeta::new_readonly(*deposit_authority, true),
    ];

    Instruction::new_with_borsh(
        *program_id,
        &RewardsInstruction::DepositMining {
            amount,
            lockup_period,
            reward_mint_addr: *reward_mint_addr,
            owner: *owner,
        },
        accounts,
    )
}

/// Creates 'WithdrawMining' instruction.
pub fn withdraw_mining(
    program_id: &Pubkey,
    reward_pool: &Pubkey,
    mining: &Pubkey,
    deposit_authority: &Pubkey,
    amount: u64,
    owner: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*reward_pool, false),
        AccountMeta::new(*mining, false),
        AccountMeta::new_readonly(*deposit_authority, true),
    ];

    Instruction::new_with_borsh(
        *program_id,
        &RewardsInstruction::WithdrawMining {
            amount,
            owner: *owner,
        },
        accounts,
    )
}

/// Creates 'Claim' instruction.
#[allow(clippy::too_many_arguments)]
pub fn claim(
    program_id: &Pubkey,
    reward_pool: &Pubkey,
    reward_mint: &Pubkey,
    vault: &Pubkey,
    mining: &Pubkey,
    user: &Pubkey,
    user_reward_token: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*reward_pool, false),
        AccountMeta::new_readonly(*reward_mint, false),
        AccountMeta::new(*vault, false),
        AccountMeta::new(*mining, false),
        AccountMeta::new_readonly(*user, true),
        AccountMeta::new(*user_reward_token, false),
        AccountMeta::new_readonly(spl_token::id(), false),
    ];

    Instruction::new_with_borsh(*program_id, &RewardsInstruction::Claim, accounts)
}

/// Creates 'InitializeRoot' instruction.
pub fn initialize_root(
    program_id: &Pubkey,
    rewards_root: &Pubkey,
    authority: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*rewards_root, true),
        AccountMeta::new(*authority, true),
        AccountMeta::new_readonly(system_program::id(), false),
    ];

    Instruction::new_with_borsh(*program_id, &RewardsInstruction::InitializeRoot, accounts)
}

/// Creates 'RestakeDeposit" instruction.
#[allow(clippy::too_many_arguments)]
pub fn restake_deposit(
    program_id: &Pubkey,
    reward_pool: &Pubkey,
    mining: &Pubkey,
    user: &Pubkey,
    mint_account: &Pubkey,
    deposit_authority: &Pubkey,
    lockup_period: LockupPeriod,
    amount: u64,
    deposit_start_ts: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*reward_pool, false),
        AccountMeta::new(*mining, false),
        AccountMeta::new_readonly(*mint_account, false),
        AccountMeta::new_readonly(*user, false),
        AccountMeta::new_readonly(*deposit_authority, true),
    ];

    Instruction::new_with_borsh(
        *program_id,
        &RewardsInstruction::RestakeDeposit {
            lockup_period,
            amount,
            deposit_start_ts,
        },
        accounts,
    )
}
