use {crate::AccountMeta, solana_define_syscall::define_syscall, solana_pubkey::Pubkey};

/// Use to query and convey information about the sibling instruction components
/// when calling the `sol_get_processed_sibling_instruction` syscall.
#[repr(C)]
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct ProcessedSiblingInstruction {
    /// Length of the instruction data
    pub data_len: u64,
    /// Number of AccountMeta structures
    pub accounts_len: u64,
}

define_syscall!(fn sol_get_processed_sibling_instruction(index: u64, meta: *mut ProcessedSiblingInstruction, program_id: *mut Pubkey, data: *mut u8, accounts: *mut AccountMeta) -> u64);
define_syscall!(fn sol_get_stack_height() -> u64);
