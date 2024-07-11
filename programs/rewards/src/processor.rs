//! Program processor
use crate::{
    instruction::RewardsInstruction,
    instructions::{
        ChangeDelegateContext, ClaimContext, CloseMiningContext, DepositMiningContext,
        DistributeRewardsContext, ExtendStakeContext, FillVaultContext, InitializeMiningContext,
        InitializePoolContext, WithdrawMiningContext,
    },
};
use borsh::BorshDeserialize;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey};

/// default processor function
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    input: &[u8],
) -> ProgramResult {
    let instruction = RewardsInstruction::try_from_slice(input)?;

    match instruction {
        RewardsInstruction::InitializePool {
            fill_authority,
            distribute_authority,
        } => {
            msg!("RewardsInstruction: InitializePool");
            InitializePoolContext::new(program_id, accounts)?.process(
                program_id,
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
        } => {
            msg!("RewardsInstruction: DepositMining");
            DepositMiningContext::new(program_id, accounts)?.process(
                program_id,
                amount,
                lockup_period,
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
        RewardsInstruction::ExtendStake {
            old_lockup_period,
            new_lockup_period,
            deposit_start_ts,
            base_amount,
            additional_amount,
        } => {
            msg!("RewardsInstruction: ExtendStake");
            ExtendStakeContext::new(program_id, accounts)?.process(
                program_id,
                old_lockup_period,
                new_lockup_period,
                deposit_start_ts,
                base_amount,
                additional_amount,
            )
        }
        RewardsInstruction::DistributeRewards => {
            msg!("RewardsInstruction: DistributeRewards");
            DistributeRewardsContext::new(program_id, accounts)?.process()
        }
        RewardsInstruction::CloseMining => {
            msg!("RewardsInstruction: CloseAccount");
            CloseMiningContext::new(program_id, accounts)?.process()
        }
        RewardsInstruction::ChangeDelegate { staked_amount } => {
            msg!("RewardsInstruction: ChangeDelegate");
            ChangeDelegateContext::new(program_id, accounts)?.process(program_id, staked_amount)
        }
    }
}
