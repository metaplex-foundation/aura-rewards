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
        RewardsInstruction::InitializePool => {
            msg!("RewardsInstruction: InitializePool");
            InitializePoolContext::new(program_id, accounts)?.process(program_id)
        }
        RewardsInstruction::AddVault => {
            msg!("RewardsInstruction: AddVault");
            AddVaultContext::new(program_id, accounts)?.process(program_id)
        }
        RewardsInstruction::FillVault { amount } => {
            msg!("RewardsInstruction: FillVault");
            FillVaultContext::new(program_id, accounts)?.process(program_id, amount)
        }
        RewardsInstruction::InitializeMining => {
            msg!("RewardsInstruction: InitializeMining");
            InitializeMiningContext::new(program_id, accounts)?.process(program_id)
        }
        RewardsInstruction::DepositMining {
            amount,
            lockup_period,
        } => {
            msg!("RewardsInstruction: DepositMining");
            DepositMiningContext::new(program_id, accounts)?.process(
                program_id,
                amount,
                lockup_period,
            )
        }
        RewardsInstruction::WithdrawMining { amount } => {
            msg!("RewardsInstruction: WithdrawMining");
            WithdrawMiningContext::new(program_id, accounts)?.process(program_id, amount)
        }
        RewardsInstruction::Claim => {
            msg!("RewardsInstruction: Claim");
            ClaimContext::new(program_id, accounts)?.process(program_id)
        }
        RewardsInstruction::InitializeRoot => {
            msg!("RewardsInstruction: InitializeRoot");
            InitializeRootContext::new(program_id, accounts)?.process(program_id)
        }
    }
}
