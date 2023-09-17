/// Return the remaining compute units the program may consume
#[inline]
pub fn sol_remaining_compute_units() -> u64 {
    #[cfg(target_os = "solana")]
    unsafe {
        solana_syscall_core::sol_remaining_compute_units()
    }

    #[cfg(not(target_os = "solana"))]
    {
        solana_msg_and_friends::program_stubs::sol_remaining_compute_units()
    }
}
