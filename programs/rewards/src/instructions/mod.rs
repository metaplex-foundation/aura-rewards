//! Program processor
use crate::instruction::RewardsInstruction;
use borsh::BorshDeserialize;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey};

mod change_delegate;
mod claim;
mod close_mining;
mod deposit_mining;
mod distribute_rewards;
mod extend_stake;
mod fill_vault;
mod initialize_mining;
mod initialize_pool;
mod penalties;
mod withdraw_mining;

pub(crate) use change_delegate::*;
pub(crate) use claim::*;
pub(crate) use close_mining::*;
pub(crate) use deposit_mining::*;
pub(crate) use distribute_rewards::*;
pub(crate) use extend_stake::*;
pub(crate) use fill_vault::*;
pub(crate) use initialize_mining::*;
pub(crate) use initialize_pool::*;
pub(crate) use penalties::*;
pub(crate) use withdraw_mining::*;

pub fn process_instruction<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = RewardsInstruction::try_from_slice(instruction_data)?;

    match instruction {
        RewardsInstruction::InitializePool {
            fill_authority,
            distribute_authority,
        } => {
            msg!("RewardsInstruction: InitializePool");
            process_initialize_pool(program_id, accounts, fill_authority, distribute_authority)
        }
        RewardsInstruction::FillVault {
            rewards,
            distribution_ends_at,
        } => {
            msg!("RewardsInstruction: FillVault");
            process_fill_vault(program_id, accounts, rewards, distribution_ends_at)
        }
        RewardsInstruction::InitializeMining { mining_owner } => {
            msg!("RewardsInstruction: InitializeMining");
            process_initialize_mining(program_id, accounts, &mining_owner)
        }
        RewardsInstruction::DepositMining {
            amount,
            lockup_period,
            mining_owner,
            delegate,
        } => {
            msg!("RewardsInstruction: DepositMining");
            process_deposit_mining(
                program_id,
                accounts,
                amount,
                lockup_period,
                &mining_owner,
                &delegate,
            )
        }
        RewardsInstruction::WithdrawMining {
            amount,
            mining_owner,
            delegate,
        } => {
            msg!("RewardsInstruction: WithdrawMining");
            process_withdraw_mining(program_id, accounts, amount, &mining_owner, &delegate)
        }
        RewardsInstruction::Claim => {
            msg!("RewardsInstruction: Claim");
            process_claim(program_id, accounts)
        }
        RewardsInstruction::ExtendStake {
            old_lockup_period,
            new_lockup_period,
            deposit_start_ts,
            base_amount,
            additional_amount,
            mining_owner,
            delegate,
        } => {
            msg!("RewardsInstruction: ExtendStake");
            process_extend_stake(
                program_id,
                accounts,
                old_lockup_period,
                new_lockup_period,
                deposit_start_ts,
                base_amount,
                additional_amount,
                &mining_owner,
                &delegate,
            )
        }
        RewardsInstruction::DistributeRewards => {
            msg!("RewardsInstruction: FillVault");
            process_distribute_rewards(program_id, accounts)
        }
        RewardsInstruction::CloseMining => {
            msg!("RewardsInstruction: CloseAccount");
            process_close_mining(program_id, accounts)
        }
        RewardsInstruction::ChangeDelegate {
            staked_amount,
            new_delegate,
        } => {
            msg!("RewardsInstruction: ChangeDelegate");
            process_change_delegate(program_id, accounts, staked_amount, &new_delegate)
        }
        RewardsInstruction::Slash {
            mining_owner,
            slash_amount_in_native,
            slash_amount_multiplied_by_period,
            stake_expiration_date,
        } => {
            msg!("RewardsInstruction: Slash");
            process_slash(
                program_id,
                accounts,
                &mining_owner,
                slash_amount_in_native,
                slash_amount_multiplied_by_period,
                stake_expiration_date,
            )
        }
        RewardsInstruction::DecreaseRewards {
            mining_owner,
            decreased_weighted_stake_number,
        } => {
            msg!("RewardsInstruction: DecreaseRewards");
            process_decrease_rewards(
                program_id,
                accounts,
                &mining_owner,
                decreased_weighted_stake_number,
            )
        }
    }
}
