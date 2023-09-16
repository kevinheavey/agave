use {
    solana_instruction::{AccountMeta, ProcessedSiblingInstruction},
    solana_pubkey::Pubkey
};


define_syscall!(fn sol_get_return_data(data: *mut u8, length: u64, program_id: *mut Pubkey) -> u64);

