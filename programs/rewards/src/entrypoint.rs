//! Program entrypoint
use crate::{error::MplxRewardsError, instructions::process_instruction};
use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult,
    program_error::PrintProgramError, pubkey::Pubkey,
};

entrypoint!(program_entrypoint);
fn program_entrypoint<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    instruction_data: &[u8],
) -> ProgramResult {
    if let Err(error) = process_instruction(program_id, accounts, instruction_data) {
        // Catch the error so we can print it
        error.print::<MplxRewardsError>();
        return Err(error);
    }
    Ok(())
}
