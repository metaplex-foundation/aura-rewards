//! Instruction types

use borsh::{BorshDeserialize, BorshSerialize};
use shank::{ShankContext, ShankInstruction};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program, sysvar,
};

use crate::utils::LockupPeriod;

/// Instructions supported by the program
#[derive(Debug, BorshDeserialize, BorshSerialize, PartialEq, Eq, ShankInstruction, ShankContext)]
#[rustfmt::skip]
pub enum RewardsInstruction {
    /// Creates and initializes a reward pool account
    #[account(0, writable, name = "reward_pool", desc = "The address of the reward pool")]
    #[account(1, name = "reward_mint", desc = "The address of the reward mint")]
    #[account(2, writable, name = "vault", desc = "The address of the reward vault")]
    #[account(3, writable, signer, name = "payer")]
    #[account(4, signer, name = "deposit_authority", desc = "Account responsible for charging mining owners")]
    #[account(5, name = "rent", desc = "The address of the Rent program")]
    #[account(6, name = "token_program", desc = "The address of the Token program where rewards are minted")]
    #[account(7, name = "system_program", desc = "The system program")]
    InitializePool {
        /// Account can fill the reward vault
        fill_authority: Pubkey,
        /// Account can distribute rewards for stakers
        distribute_authority: Pubkey,
    },

    #[account(0, writable, name = "reward_pool", desc = "The address of the reward pool")]
    #[account(1, name = "reward_mint", desc = "The address of the reward mint")]
    #[account(2, writable, name = "vault", desc = "The address of the reward vault")]
    #[account(3, signer, name = "fill_authority", desc = "The address of the wallet who is responsible for filling pool's vault with rewards")]
    #[account(4, writable, name = "source_token_account", desc = "The address of the TA from which tokens will be spent")]
    #[account(5, name = "token_program", desc = "The address of the Token program where rewards are minted")]
    FillVault {
        /// Amount to fill
        rewards: u64,
        /// Rewards distribution ends at given date
        distribution_ends_at: u64,
    },

    /// Initializes mining account for the specified mining owner
    #[account(0, writable, name = "reward_pool", desc = "The address of the reward pool")]
    #[account(1, writable, name = "mining", desc = "The address of the mining account which belongs to the user and stores info about user's rewards")]
    #[account(2, writable, signer, name = "payer")]
    #[account(3, name = "system_program", desc = "The system program")]
    InitializeMining {
        /// Represent the end-user, owner of the mining
        mining_owner: Pubkey,
    },

    /// Deposits amount of supply to the mining account
    #[account(0, writable, name = "reward_pool", desc = "The address of the reward pool")]
    #[account(1, writable, name = "mining", desc = "The address of the mining account which belongs to the user and stores info about user's rewards")]
    #[account(2, signer, name = "deposit_authority", desc = "The address of the Staking program's Registrar, which is PDA and is responsible for signing CPIs")]
    #[account(3, name = "delegate_mining", desc = "The address of Mining Account that might be used as a delegate in delegated staking model")]
    DepositMining {
        /// Amount to deposit
        amount: u64,
        /// Lockup Period
        lockup_period: LockupPeriod,
        /// Specifies the owner of the Mining Account
        mining_owner: Pubkey,
        delegate: Pubkey,
    },

    /// Withdraws amount of supply to the mining account
    #[account(0, writable, name = "reward_pool", desc = "The address of the reward pool")]
    #[account(1, writable, name = "mining", desc = "The address of the mining account which belongs to the user and stores info about user's rewards")]
    #[account(2, signer, name = "deposit_authority", desc = "The address of the Staking program's Registrar, which is PDA and is responsible for signing CPIs")]
    #[account(3, name = "delegate_mining", desc = "The address of Mining Account that might be used as a delegate in delegated staking model")]
    WithdrawMining {
        /// Amount to withdraw
        amount: u64,
        /// Specifies the owner of the Mining Account
        mining_owner: Pubkey,
        delegate: Pubkey,
    },

    /// Claims amount of rewards
    #[account(0, name = "reward_pool", desc = "The address of the reward pool")]
    #[account(1, name = "reward_mint", desc = "The address of the reward mint")]
    #[account(2, writable, name = "vault", desc = "The address of the reward vault")]
    #[account(3, writable, name = "mining", desc = "The address of the mining account which belongs to the user and stores info about user's rewards")]
    #[account(4, signer, name = "mining_owner", desc = "The end user the mining accounts belongs to")]
    #[account(5, signer, name = "deposit_authority", desc = "The address of the Staking program's Registrar, which is PDA and is responsible for signing CPIs")]
    #[account(6, writable, name = "mining_owner_reward_token_account", desc = "ATA where tokens will be claimed to")]
    #[account(7, name = "token_program", desc = "The address of the Token program where rewards are minted")]
    Claim,

    /// Extends stake
    #[account(0, writable, name = "reward_pool", desc = "The address of the reward pool")]
    #[account(1, writable, name = "mining", desc = "The address of the mining account which belongs to the user and stores info about user's rewards")]
    #[account(2, name = "reward_mint", desc = "The address of the reward mint")]
    #[account(3, signer, name = "deposit_authority", desc = "The address of the Staking program's Registrar, which is PDA and is responsible for signing CPIs")]
    #[account(4, name = "delegate_mining", desc = "The address of Mining Account that might be used as a delegate in delegated staking model")]
    ExtendStake {
        /// Lockup period before restaking. Actually it's only needed
        /// for Flex to AnyPeriod edge case
        old_lockup_period: LockupPeriod,
        /// Requested lockup period for restaking
        new_lockup_period: LockupPeriod,
        /// Deposit start_ts
        deposit_start_ts: u64,
        /// Amount of tokens to be restaked, this
        /// number cannot be decreased. It reflects the number of staked tokens
        /// before the extend_stake function call
        base_amount: u64,
        /// In case user wants to increase it's staked number of tokens,
        /// the addition amount might be provided
        additional_amount: u64,
        /// The wallet who owns the mining account
        mining_owner: Pubkey,
        /// Wallet addres of delegate
        delegate: Pubkey,
    },

    /// Distributes tokens among mining owners
    #[account(0, writable, name = "reward_pool", desc = "The address of the reward pool")]
    #[account(1, signer, name = "distribute_authority", desc = "The address of Authority who is eligble for distributiong rewards for users")]
    DistributeRewards,

    /// Closes mining account and transfers all lamports to the target account
    #[account(0, name = "mining", desc = "The address of the user's mining account")]
    #[account(1, signer, name = "mining_owner", desc = "The end user the mining accounts belongs to")]
    #[account(2, writable, name = "target_account", desc = "The address where lamports from account closing will be transferred")]
    #[account(3, signer, name = "deposit_authority")]
    #[account(4, writable, name = "reward_pool", desc = "The address of the reward pool")]
    CloseMining,

    /// Changes delegate mining account
    #[account(0, writable, name = "reward_pool", desc = "The address of the reward pool")]
    #[account(1, writable, name = "mining", desc = "The address of the mining account which belongs to the user and stores info about user's rewards")]
    #[account(2, signer, name = "deposit_authority", desc = "The address of the Staking program's Registrar, which is PDA and is responsible for signing CPIs")]
    #[account(3, signer, name = "mining_owner", desc = "The end user the mining accounts belongs to")]
    #[account(4, writable, name = "old_delegate_mining", desc = "The address of the old delegate mining account")]
    #[account(5, writable, name = "new_delegate_mining", desc = "The address of the new delegate mining account")]
    ChangeDelegate {
        /// Amount of staked tokens
        staked_amount: u64,
        new_delegate: Pubkey,
    },

    /// Prevents the mining account from rewards withdrawing
    #[account(0, signer, name = "deposit_authority", desc = "The address of the Staking program's Registrar, which is PDA and is responsible for signing CPIs")]
    #[account(1, name = "reward_pool", desc = "The address of the reward pool")]
    #[account(2, writable, name = "mining", desc = "The address of the mining account which belongs to the user and stores info about user's rewards")]
    RestrictTokenFlow {
        mining_owner: Pubkey,
    },

    #[account(0, signer, name = "deposit_authority", desc = "The address of the Staking program's Registrar, which is PDA and is responsible for signing CPIs")]
    #[account(1, name = "reward_pool", desc = "The address of the reward pool")]
    #[account(2, writable, name = "mining", desc = "The address of the mining account which belongs to the user and stores info about user's rewards")]
    AllowTokenFlow {
        mining_owner: Pubkey,
    },

    #[account(0, signer, name = "deposit_authority", desc = "The address of the Staking program's Registrar, which is PDA and is responsible for signing CPIs")]
    #[account(1, name = "reward_pool", desc = "The address of the reward pool")]
    #[account(2, writable, name = "mining", desc = "The address of the mining account which belongs to the user and stores info about user's rewards")]
    RestrictBatchMinting {
        restrict_batch_minting_until_ts: u64,
        mining_owner: Pubkey,
    },

    #[account(0, signer, name = "deposit_authority", desc = "The address of the Staking program's Registrar, which is PDA and is responsible for signing CPIs")]
    #[account(1, writable, name = "reward_pool", desc = "The address of the reward pool")]
    #[account(2, writable, name = "mining", desc = "The address of the mining account which belongs to the user and stores info about user's rewards")]
    Slash {
        mining_owner: Pubkey,
        // number of tokens that had been slashed
        slash_amount_in_native: u64,
        // weighted stake part for the slashed number of tokens multiplied by the period
        slash_amount_multiplied_by_period: u64,
        // None if it's Flex period, because it's already expired
        stake_expiration_date: Option<u64>,
    },

    #[account(0, signer, name = "deposit_authority", desc = "The address of the Staking program's Registrar, which is PDA and is responsible for signing CPIs")]
    #[account(1, writable, name = "reward_pool", desc = "The address of the reward pool")]
    #[account(2, writable, name = "mining", desc = "The address of the mining account which belongs to the user and stores info about user's rewards")]
    DecreaseRewards {
        mining_owner: Pubkey,
        // The number by which weighted stake should be decreased
        decreased_weighted_stake_number: u64,
    },
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
        AccountMeta::new_readonly(*deposit_authority, true),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(system_program::id(), false),
    ];

    Instruction::new_with_borsh(
        *program_id,
        &RewardsInstruction::InitializePool {
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
    rewards: u64,
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
            rewards,
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
    delegate_mining: &Pubkey,
    amount: u64,
    lockup_period: LockupPeriod,
    mining_owner: &Pubkey,
    delegate: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*reward_pool, false),
        AccountMeta::new(*mining, false),
        AccountMeta::new_readonly(*deposit_authority, true),
        AccountMeta::new(*delegate_mining, false),
    ];

    Instruction::new_with_borsh(
        *program_id,
        &RewardsInstruction::DepositMining {
            amount,
            lockup_period,
            mining_owner: *mining_owner,
            delegate: *delegate,
        },
        accounts,
    )
}

/// Creates 'WithdrawMining' instruction.
#[allow(clippy::too_many_arguments)]
pub fn withdraw_mining(
    program_id: &Pubkey,
    reward_pool: &Pubkey,
    mining: &Pubkey,
    deposit_authority: &Pubkey,
    delegate_mining: &Pubkey,
    amount: u64,
    mining_owner: &Pubkey,
    delegate: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*reward_pool, false),
        AccountMeta::new(*mining, false),
        AccountMeta::new_readonly(*deposit_authority, true),
        AccountMeta::new(*delegate_mining, false),
    ];

    Instruction::new_with_borsh(
        *program_id,
        &RewardsInstruction::WithdrawMining {
            amount,
            mining_owner: *mining_owner,
            delegate: *delegate,
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

/// Creates 'ExtendStake" instruction.
#[allow(clippy::too_many_arguments)]
pub fn extend_stake(
    program_id: &Pubkey,
    reward_pool: &Pubkey,
    mining: &Pubkey,
    deposit_authority: &Pubkey,
    delegate_mining: &Pubkey,
    old_lockup_period: LockupPeriod,
    new_lockup_period: LockupPeriod,
    deposit_start_ts: u64,
    base_amount: u64,
    additional_amount: u64,
    mining_owner: &Pubkey,
    delegate: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*reward_pool, false),
        AccountMeta::new(*mining, false),
        AccountMeta::new_readonly(*deposit_authority, true),
        AccountMeta::new(*delegate_mining, false),
    ];

    Instruction::new_with_borsh(
        *program_id,
        &RewardsInstruction::ExtendStake {
            old_lockup_period,
            new_lockup_period,
            deposit_start_ts,
            base_amount,
            additional_amount,
            mining_owner: *mining_owner,
            delegate: *delegate,
        },
        accounts,
    )
}

/// Creates 'Distribute Rewards" instruction.
#[allow(clippy::too_many_arguments)]
pub fn distribute_rewards(
    program_id: &Pubkey,
    reward_pool: &Pubkey,
    distribute_authority: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*reward_pool, false),
        AccountMeta::new_readonly(*distribute_authority, true),
    ];

    Instruction::new_with_borsh(
        *program_id,
        &RewardsInstruction::DistributeRewards,
        accounts,
    )
}

/// Creates 'Distribute Rewards" instruction.
#[allow(clippy::too_many_arguments)]
pub fn close_mining(
    program_id: &Pubkey,
    mining: &Pubkey,
    mining_owner: &Pubkey,
    target_account: &Pubkey,
    deposit_authority: &Pubkey,
    reward_pool: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*mining, false),
        AccountMeta::new_readonly(*mining_owner, true),
        AccountMeta::new(*target_account, false),
        AccountMeta::new_readonly(*deposit_authority, true),
        AccountMeta::new_readonly(*reward_pool, false),
    ];

    Instruction::new_with_borsh(*program_id, &RewardsInstruction::CloseMining, accounts)
}

/// Creates 'Change Delegate" instruction.
#[allow(clippy::too_many_arguments)]
pub fn change_delegate(
    program_id: &Pubkey,
    reward_pool: &Pubkey,
    mining: &Pubkey,
    deposit_authority: &Pubkey,
    mining_owner: &Pubkey,
    old_delegate_mining: &Pubkey,
    new_delegate_mining: &Pubkey,
    new_delegate: &Pubkey,
    staked_amount: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*reward_pool, false),
        AccountMeta::new(*mining, false),
        AccountMeta::new_readonly(*deposit_authority, true),
        AccountMeta::new_readonly(*mining_owner, true),
        AccountMeta::new(*old_delegate_mining, false),
        AccountMeta::new(*new_delegate_mining, false),
    ];

    Instruction::new_with_borsh(
        *program_id,
        &RewardsInstruction::ChangeDelegate {
            staked_amount,
            new_delegate: *new_delegate,
        },
        accounts,
    )
}

pub fn restrict_tokenflow(
    program_id: &Pubkey,
    deposit_authority: &Pubkey,
    reward_pool: &Pubkey,
    mining: &Pubkey,
    mining_owner: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*deposit_authority, true),
        AccountMeta::new_readonly(*reward_pool, false),
        AccountMeta::new(*mining, false),
    ];

    Instruction::new_with_borsh(
        *program_id,
        &RewardsInstruction::RestrictTokenFlow {
            mining_owner: *mining_owner,
        },
        accounts,
    )
}

pub fn allow_tokenflow(
    program_id: &Pubkey,
    deposit_authority: &Pubkey,
    reward_pool: &Pubkey,
    mining: &Pubkey,
    mining_owner: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*deposit_authority, true),
        AccountMeta::new_readonly(*reward_pool, false),
        AccountMeta::new(*mining, false),
    ];

    Instruction::new_with_borsh(
        *program_id,
        &RewardsInstruction::AllowTokenFlow {
            mining_owner: *mining_owner,
        },
        accounts,
    )
}

pub fn restrict_batch_minting(
    program_id: &Pubkey,
    deposit_authority: &Pubkey,
    reward_pool: &Pubkey,
    mining: &Pubkey,
    mining_owner: &Pubkey,
    restrict_batch_minting_until_ts: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*deposit_authority, true),
        AccountMeta::new_readonly(*reward_pool, false),
        AccountMeta::new(*mining, false),
    ];

    Instruction::new_with_borsh(
        *program_id,
        &RewardsInstruction::RestrictBatchMinting {
            restrict_batch_minting_until_ts,
            mining_owner: *mining_owner,
        },
        accounts,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn slash(
    program_id: &Pubkey,
    deposit_authority: &Pubkey,
    reward_pool: &Pubkey,
    mining: &Pubkey,
    mining_owner: &Pubkey,
    slash_amount_in_native: u64,
    slash_amount_multiplied_by_period: u64,
    stake_expiration_date: Option<u64>,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*deposit_authority, true),
        AccountMeta::new(*reward_pool, false),
        AccountMeta::new(*mining, false),
    ];

    Instruction::new_with_borsh(
        *program_id,
        &RewardsInstruction::Slash {
            mining_owner: *mining_owner,
            slash_amount_in_native,
            slash_amount_multiplied_by_period,
            stake_expiration_date,
        },
        accounts,
    )
}

pub fn decrease_rewards(
    program_id: &Pubkey,
    deposit_authority: &Pubkey,
    reward_pool: &Pubkey,
    mining: &Pubkey,
    mining_owner: &Pubkey,
    decreased_weighted_stake_number: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*deposit_authority, true),
        AccountMeta::new(*reward_pool, false),
        AccountMeta::new(*mining, false),
    ];

    Instruction::new_with_borsh(
        *program_id,
        &RewardsInstruction::DecreaseRewards {
            mining_owner: *mining_owner,
            decreased_weighted_stake_number,
        },
        accounts,
    )
}
