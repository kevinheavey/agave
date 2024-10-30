#![cfg_attr(feature = "frozen-abi", feature(min_specialization))]
#![deny(clippy::arithmetic_side_effects)]
#![deny(clippy::indexing_slicing)]

#[macro_use]
extern crate solana_metrics;

pub use solana_rbpf;
pub mod mem_pool;
#[deprecated(since = "2.2.0", note = "Use `solana-invoke-context` crate instead")]
pub use solana_invoke_context::{invoke_context, loaded_programs, stable_log, sysvar_cache};
