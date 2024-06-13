//! Instruction types

use borsh::{BorshDeserialize, BorshSerialize};
use shank::{ShankContext, ShankInstruction};
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::pubkey::Pubkey;
use solana_program::{system_program, sysvar};

use crate::utils::LockupPeriod;

/// Instructions supported by the program
#[derive(
    Debug, BorshDeserialize, BorshSerialize, PartialEq, Eq, ShankContext, ShankInstruction,
)]
pub enum RewardsInstruction {
    /// Creates and initializes a reward pool account
    ///
    /// Accounts:
    /// [W] Reward pool account
    /// [R] Reward mint account
    /// [W] Vault account
    /// [WS] Payer
    /// [R] Rent sysvar
    /// [R] Token program
    /// [R] System program
    InitializePool {
        /// Account responsible for charging mining owners
        deposit_authority: Pubkey,
        /// Account can fill the reward vault
        fill_authority: Pubkey,
        /// Account can distribute rewards for stakers
        distribute_authority: Pubkey,
    },

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

    /// Initializes mining account for the specified mining owner
    ///
    /// Accounts:
    /// [W] Reward pool account
    /// [W] Mining
    /// [WS] Payer
    /// [R] System program
    InitializeMining {
        /// Represent the end-user, owner of the mining
        mining_owner: Pubkey,
    },

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
        /// Specifies the owner of the Mining Account
        owner: Pubkey,
    },

    /// Withdraws amount of supply to the mining account
    ///
    /// Accounts:
    /// [W] Reward pool account
    /// [W] Mining
    /// [R] Mining owner
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
    /// [RS] Mining owner
    /// [RS] Deposit authority
    /// [W] Mining owner reward token account
    /// [R] Token program
    Claim,

    /// Restakes deposit
    ///
    /// Accounts:
    /// [W] Reward pool account
    /// [W] Mining
    /// [R] Mint of rewards account
    /// [R] Mining owner
    /// [RS] Deposit authority
    RestakeDeposit {
        /// Lockup period before restaking. Actually it's only needed
        /// for Flex to AnyPeriod edge case
        old_lockup_period: LockupPeriod,
        /// Requested lockup period for restaking
        new_lockup_period: LockupPeriod,
        /// Deposit start_ts
        deposit_start_ts: u64,
        /// Amount of tokens to be restaked, this
        /// number cannot be decreased. It reflects the number of staked tokens
        /// before the restake function call
        base_amount: u64,
        /// In case user wants to increase it's staked number of tokens,
        /// the addition amount might be provided
        additional_amount: u64,
        /// The wallet who owns the mining account
        mining_owner: Pubkey,
    },

    /// Distributes tokens among mining owners
    ///
    /// Accounts:
    /// [W] Reward pool account
    /// [R] Mint of rewards account
    /// [W] Vault for rewards account
    /// [RS] Distribute rewards authority
    DistributeRewards,
}

/// Creates 'InitializePool' instruction.
#[allow(clippy::too_many_arguments)]
pub fn initialize_pool(
    program_id: &Pubkey,
    reward_pool: &Pubkey,
    reward_mint: &Pubkey,
    vault: &Pubkey,
    payer: &Pubkey,
    deposit_authority: &Pubkey,
    fill_authority: &Pubkey,
    distribute_authority: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*reward_pool, false),
        AccountMeta::new_readonly(*reward_mint, false),
        AccountMeta::new(*vault, false),
        AccountMeta::new(*payer, true),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(system_program::id(), false),
    ];

    Instruction::new_with_borsh(
        *program_id,
        &RewardsInstruction::InitializePool {
            deposit_authority: *deposit_authority,
            fill_authority: *fill_authority,
            distribute_authority: *distribute_authority,
        },
        accounts,
    )
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
    payer: &Pubkey,
    mining_owner: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*reward_pool, false),
        AccountMeta::new(*mining, false),
        AccountMeta::new(*payer, true),
        AccountMeta::new_readonly(system_program::id(), false),
    ];

    Instruction::new_with_borsh(
        *program_id,
        &RewardsInstruction::InitializeMining {
            mining_owner: *mining_owner,
        },
        accounts,
    )
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
    mining_owner: &Pubkey,
    deposit_authority: &Pubkey,
    mining_owner_reward_token: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*reward_pool, false),
        AccountMeta::new_readonly(*reward_mint, false),
        AccountMeta::new(*vault, false),
        AccountMeta::new(*mining, false),
        AccountMeta::new_readonly(*mining_owner, true),
        AccountMeta::new_readonly(*deposit_authority, true),
        AccountMeta::new(*mining_owner_reward_token, false),
        AccountMeta::new_readonly(spl_token::id(), false),
    ];

    Instruction::new_with_borsh(*program_id, &RewardsInstruction::Claim, accounts)
}

/// Creates 'RestakeDeposit" instruction.
#[allow(clippy::too_many_arguments)]
pub fn restake_deposit(
    program_id: &Pubkey,
    reward_pool: &Pubkey,
    mining: &Pubkey,
    deposit_authority: &Pubkey,
    old_lockup_period: LockupPeriod,
    new_lockup_period: LockupPeriod,
    deposit_start_ts: u64,
    base_amount: u64,
    additional_amount: u64,
    mining_owner: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*reward_pool, false),
        AccountMeta::new(*mining, false),
        AccountMeta::new_readonly(*deposit_authority, true),
    ];

    Instruction::new_with_borsh(
        *program_id,
        &RewardsInstruction::RestakeDeposit {
            old_lockup_period,
            new_lockup_period,
            deposit_start_ts,
            base_amount,
            additional_amount,
            mining_owner: *mining_owner,
        },
        accounts,
    )
}

/// Creates 'RestakeDeposit" instruction.
#[allow(clippy::too_many_arguments)]
pub fn distribute_rewards(
    program_id: &Pubkey,
    reward_pool: &Pubkey,
    reward_mint: &Pubkey,
    reward_vault: &Pubkey,
    distribute_authority: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*reward_pool, false),
        AccountMeta::new_readonly(*reward_mint, false),
        AccountMeta::new(*reward_vault, false),
        AccountMeta::new_readonly(*distribute_authority, true),
    ];

    Instruction::new_with_borsh(
        *program_id,
        &RewardsInstruction::DistributeRewards,
        accounts,
    )
}
