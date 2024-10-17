//! Program entrypoint
use crate::{error::MplxRewardsError, instructions::process_instruction};

#[cfg(not(feature = "testing"))]
use solana_program::instruction::get_stack_height;
use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult,
    program_error::PrintProgramError, pubkey::Pubkey,
};

entrypoint!(program_entrypoint);

pub const TRANSACTION_LEVEL_STACK_HEIGHT: usize = 1;

fn program_entrypoint<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    instruction_data: &[u8],
) -> ProgramResult {
    if let Err(error) = process_instruction(program_id, accounts, instruction_data) {
        #[cfg(not(feature = "testing"))]
        if get_stack_height() == TRANSACTION_LEVEL_STACK_HEIGHT {
            return Err(MplxRewardsError::ForbiddenInvocation.into());
        }
        // Catch the error so we can print it
        error.print::<MplxRewardsError>();
        return Err(error);
    }
    Ok(())
}
