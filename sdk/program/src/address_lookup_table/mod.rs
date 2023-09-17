//! The [address lookup table program][np].
//!
//! [np]: https://docs.solana.com/developing/runtime-facilities/programs#address-lookup-table-program

pub mod error;
pub mod state;

pub use solana_address_lookup_table_core::{instruction, program, AddressLookupTableAccount};
