use mplx_rewards::instruction::RewardsInstruction;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program, sysvar,
};

/// Creates 'InitializePool' instruction.
pub fn initialize_pool(
    root_account: &Pubkey,
    reward_pool: &Pubkey,
    liquidity_mint: &Pubkey,
    authority: &Pubkey,
    payer: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*root_account, false),
        AccountMeta::new(*reward_pool, false),
        AccountMeta::new_readonly(*liquidity_mint, false),
        AccountMeta::new_readonly(*authority, false),
        AccountMeta::new(*payer, true),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];

    Instruction::new_with_borsh(
        mplx_rewards::ID,
        &RewardsInstruction::InitializePool,
        accounts,
    )
}

/// Creates 'AddVault' instruction.
pub fn add_vault(
    rewards_root: &Pubkey,
    reward_pool: &Pubkey,
    reward_mint: &Pubkey,
    vault: &Pubkey,
    fee_account: &Pubkey,
    payer: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*rewards_root, false),
        AccountMeta::new(*reward_pool, false),
        AccountMeta::new_readonly(*reward_mint, false),
        AccountMeta::new(*vault, false),
        AccountMeta::new_readonly(*fee_account, false),
        AccountMeta::new(*payer, true),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];

    Instruction::new_with_borsh(mplx_rewards::ID, &RewardsInstruction::AddVault, accounts)
}

/// Creates 'FillVault' instruction.
#[allow(clippy::too_many_arguments)]
pub fn fill_vault(
    reward_pool: &Pubkey,
    reward_mint: &Pubkey,
    vault: &Pubkey,
    fee_account: &Pubkey,
    authority: &Pubkey,
    from: &Pubkey,
    amount: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*reward_pool, false),
        AccountMeta::new_readonly(*reward_mint, false),
        AccountMeta::new(*vault, false),
        AccountMeta::new(*fee_account, false),
        AccountMeta::new_readonly(*authority, true),
        AccountMeta::new(*from, false),
    ];

    Instruction::new_with_borsh(
        mplx_rewards::ID,
        &RewardsInstruction::FillVault { amount },
        accounts,
    )
}

/// Creates 'InitializeMining' instruction.
pub fn initialize_mining(
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
    ];

    Instruction::new_with_borsh(
        mplx_rewards::ID,
        &RewardsInstruction::InitializeMining,
        accounts,
    )
}

/// Creates 'DepositMining' instruction.
pub fn deposit_mining(
    reward_pool: &Pubkey,
    mining: &Pubkey,
    user: &Pubkey,
    deposit_authority: &Pubkey,
    amount: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*reward_pool, false),
        AccountMeta::new(*mining, false),
        AccountMeta::new_readonly(*user, false),
        AccountMeta::new_readonly(*deposit_authority, true),
    ];

    Instruction::new_with_borsh(
        mplx_rewards::ID,
        &RewardsInstruction::DepositMining { amount },
        accounts,
    )
}

/// Creates 'WithdrawMining' instruction.
pub fn withdraw_mining(
    reward_pool: &Pubkey,
    mining: &Pubkey,
    user: &Pubkey,
    deposit_authority: &Pubkey,
    amount: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*reward_pool, false),
        AccountMeta::new(*mining, false),
        AccountMeta::new_readonly(*user, false),
        AccountMeta::new_readonly(*deposit_authority, true),
    ];

    Instruction::new_with_borsh(
        mplx_rewards::ID,
        &RewardsInstruction::WithdrawMining { amount },
        accounts,
    )
}

/// Creates 'Claim' instruction.
#[allow(clippy::too_many_arguments)]
pub fn claim(
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
    ];

    Instruction::new_with_borsh(mplx_rewards::ID, &RewardsInstruction::Claim, accounts)
}

/// Creates 'InitializeRoot' instruction.
pub fn initialize_root(rewards_root: &Pubkey, authority: &Pubkey) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*rewards_root, true),
        AccountMeta::new(*authority, true),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];

    Instruction::new_with_borsh(
        mplx_rewards::ID,
        &RewardsInstruction::InitializeRoot,
        accounts,
    )
}
