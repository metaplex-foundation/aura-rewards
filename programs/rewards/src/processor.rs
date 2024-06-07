//! Program processor

use crate::instruction::RewardsInstruction;
use crate::instructions::*;
use borsh::BorshDeserialize;
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::msg;
use solana_program::pubkey::Pubkey;

/// default processor function
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    input: &[u8],
) -> ProgramResult {
    let instruction = RewardsInstruction::try_from_slice(input)?;

    match instruction {
        RewardsInstruction::InitializePool {
            deposit_authority,
            fill_authority,
            distribute_authority,
        } => {
            msg!("RewardsInstruction: InitializePool");
            InitializePoolContext::new(program_id, accounts)?.process(
                program_id,
                deposit_authority,
                fill_authority,
                distribute_authority,
            )
        }
        RewardsInstruction::FillVault {
            amount,
            distribution_ends_at,
        } => {
            msg!("RewardsInstruction: FillVault");
            FillVaultContext::new(program_id, accounts)?.process(
                program_id,
                amount,
                distribution_ends_at,
            )
        }
        RewardsInstruction::InitializeMining { mining_owner } => {
            msg!("RewardsInstruction: InitializeMining");
            InitializeMiningContext::new(program_id, accounts)?.process(program_id, &mining_owner)
        }
        RewardsInstruction::DepositMining {
            amount,
            lockup_period,
            owner,
        } => {
            msg!("RewardsInstruction: DepositMining");
            DepositMiningContext::new(program_id, accounts)?.process(
                program_id,
                amount,
                lockup_period,
                &owner,
            )
        }
        RewardsInstruction::WithdrawMining { amount, owner } => {
            msg!("RewardsInstruction: WithdrawMining");
            WithdrawMiningContext::new(program_id, accounts)?.process(program_id, amount, &owner)
        }
        RewardsInstruction::Claim => {
            msg!("RewardsInstruction: Claim");
            ClaimContext::new(program_id, accounts)?.process(program_id)
        }
        RewardsInstruction::RestakeDeposit {
            lockup_period,
            amount,
            deposit_start_ts,
        } => {
            msg!("RewardsInstruction: RestakeDeposit");
            RestakeDepositContext::new(program_id, accounts)?.process(
                program_id,
                lockup_period,
                amount,
                deposit_start_ts,
            )
        }
        RewardsInstruction::DistributeRewards => {
            msg!("RewardsInstruction: FillVault");
            DistributeRewardsContext::new(program_id, accounts)?.process(program_id)
        }
    }
}
