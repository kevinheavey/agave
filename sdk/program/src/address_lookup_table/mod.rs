//! The [address lookup table program][np].
//!
//! [np]: https://docs.solana.com/developing/runtime-facilities/programs#address-lookup-table-program

pub mod error;
pub mod instruction;
pub mod state;

pub mod program {
    pub use solana_native_programs::address_lookup_table::{check_id, id, ID};
}

/// The definition of address lookup table accounts.
///
/// As used by the `crate::message::v0` message format.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AddressLookupTableAccount {
    pub key: solana_pubkey::Pubkey,
    pub addresses: Vec<solana_pubkey::Pubkey>,
}
