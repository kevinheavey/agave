#![allow(incomplete_features)]
#![cfg_attr(RUSTC_WITH_SPECIALIZATION, feature(specialization))]

// Allows macro expansion of `use ::solana_program::*` to work within this crate
extern crate self as solana_program;

#[deprecated(since = "2.1.0", note = "Use `solana-decode-error` crate instead")]
pub use solana_decode_error as decode_error;
/// Same as [`declare_id`] except that it reports that this ID has been deprecated.
pub use solana_sdk_macro::program_declare_deprecated_id as declare_deprecated_id;
#[cfg(target_arch = "wasm32")]
pub use wasm_bindgen::prelude::wasm_bindgen;
pub use {
    solana_account_info as account_info, solana_debug_account_data,
    solana_entrypoint as entrypoint, solana_hash as hash, solana_instruction as instruction,
    solana_log as log, solana_program_error as program_error,
    solana_program_memory as program_memory, solana_pubkey as pubkey,
    solana_sdk_macro::{program_declare_id as declare_id, program_pubkey as pubkey},
    solana_syscalls as syscalls,
};
