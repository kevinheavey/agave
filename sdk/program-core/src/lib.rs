#![allow(incomplete_features)]
#![cfg_attr(RUSTC_WITH_SPECIALIZATION, feature(specialization))]

// Allows macro expansion of `use ::solana_program_core::*` to work within this crate
extern crate self as solana_program_core;

pub mod account_info;
pub mod address_lookup_table;
pub mod bpf_loader;
pub mod bpf_loader_deprecated;
pub mod bpf_loader_upgradeable;
pub mod clock;
pub mod compute_units;
pub mod debug_account_data;
pub mod ed25519_program;
pub mod entrypoint;
pub mod epoch_rewards;
pub mod epoch_schedule;
pub mod fee_calculator;
pub mod hash;
pub mod instruction;
pub mod keccak;
pub mod lamports;
pub mod last_restart_slot;
pub mod loader_upgradeable_instruction;
pub mod log;
pub mod message;
pub mod nonce;
pub mod program;
pub mod program_error;
pub mod program_option;
pub mod program_pack;
pub mod program_stubs;
#[cfg(feature = "bincode")]
pub mod program_utils;
pub mod pubkey;
pub mod rent;
pub mod secp256k1_program;
pub mod serialize_utils;
pub mod slot_hashes;
pub mod slot_history;
pub mod stable_layout;
pub mod stake_history;
pub mod syscalls;
pub mod system_instruction;
pub mod system_program;
pub mod sysvar;
pub mod wasm;

/// The [config native program][np].
///
/// [np]: https://docs.solanalabs.com/runtime/programs#config-program
pub mod config {
    pub mod program {
        crate::declare_id!("Config1111111111111111111111111111111111111");
    }
}

pub use solana_msg::msg;
/// Same as [`declare_id`] except that it reports that this ID has been deprecated.
pub use solana_sdk_macro::program_core_declare_deprecated_id as declare_deprecated_id;
/// Convenience macro to declare a static public key and functions to interact with it.
///
/// Input: a single literal base58 string representation of a program's ID.
///
/// # Example
///
/// ```
/// # // wrapper is used so that the macro invocation occurs in the item position
/// # // rather than in the statement position which isn't allowed.
/// use std::str::FromStr;
/// use solana_program_core::{declare_id, pubkey::Pubkey};
///
/// # mod item_wrapper {
/// #   use solana_program_core::declare_id;
/// declare_id!("My11111111111111111111111111111111111111111");
/// # }
/// # use item_wrapper::id;
///
/// let my_id = Pubkey::from_str("My11111111111111111111111111111111111111111").unwrap();
/// assert_eq!(id(), my_id);
/// ```
pub use solana_sdk_macro::program_core_declare_id as declare_id;
/// Convenience macro to define a static public key.
///
/// Input: a single literal base58 string representation of a Pubkey.
///
/// # Example
///
/// ```
/// use std::str::FromStr;
/// use solana_program_core::{pubkey, pubkey::Pubkey};
///
/// static ID: Pubkey = pubkey!("My11111111111111111111111111111111111111111");
///
/// let my_id = Pubkey::from_str("My11111111111111111111111111111111111111111").unwrap();
/// assert_eq!(ID, my_id);
/// ```
pub use solana_sdk_macro::program_core_pubkey as pubkey;
#[cfg(feature = "serde")]
#[macro_use]
extern crate serde_derive;

#[cfg_attr(feature = "frozen-abi", macro_use)]
#[cfg(feature = "frozen-abi")]
extern crate solana_frozen_abi_macro;

// This module is purposefully listed after all other exports: because of an
// interaction within rustdoc between the reexports inside this module of
// `solana_program`'s top-level modules, and `solana_sdk`'s glob re-export of
// `solana_program`'s top-level modules, if this module is not lexically last
// rustdoc fails to generate documentation for the re-exports within
// `solana_sdk`.
#[cfg(not(target_os = "solana"))]
pub mod example_mocks;
