#![no_std]

use pinocchio::{
    address::{declare_id, Address},
    entrypoint,
    error::ProgramError,
    AccountView, ProgramResult,
};

entrypoint!(process_instruction);

pub mod errors;
pub mod helpers;
pub mod instructions;
pub mod state;

pub use instructions::*;

declare_id!("22222222222222222222222222222222222222222222");

fn process_instruction(
    _program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    match instruction_data.split_first() {
        Some((Make::DISCRIMINATOR, data)) => Make::try_from((data, accounts))?.process(),
        Some((Take::DISCRIMINATOR, _)) => Take::try_from(accounts)?.process(),
        Some((Refund::DISCRIMINATOR, _)) => Refund::try_from(accounts)?.process(),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}
