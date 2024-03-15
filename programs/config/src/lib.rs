#![allow(clippy::arithmetic_side_effects)]
pub mod config_instruction;
pub mod config_processor;
pub mod date_instruction;

pub use {
    solana_config_program_api::{create_config_account, get_config_data, ConfigKeys, ConfigState},
    solana_sdk::config::program::id,
};
