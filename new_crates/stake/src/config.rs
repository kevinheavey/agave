//! config for staking
//!  carries variables that the stake program cares about

#[deprecated(
    since = "1.16.7",
    note = "Please use `solana_sdk::stake::state::{DEFAULT_SLASH_PENALTY, DEFAULT_WARMUP_COOLDOWN_RATE}` instead"
)]
pub use super::state::{DEFAULT_SLASH_PENALTY, DEFAULT_WARMUP_COOLDOWN_RATE};
use solana_sdk_macro::program_declare_id_lite;

// stake config ID
program_declare_id_lite!("StakeConfig11111111111111111111111111111111");

#[deprecated(
    since = "1.16.7",
    note = "Please use `solana_sdk::stake::state::warmup_cooldown_rate()` instead"
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Config {
    /// how much stake we can activate/deactivate per-epoch as a fraction of currently effective stake
    pub warmup_cooldown_rate: f64,
    /// percentage of stake lost when slash, expressed as a portion of std::u8::MAX
    pub slash_penalty: u8,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            warmup_cooldown_rate: DEFAULT_WARMUP_COOLDOWN_RATE,
            slash_penalty: DEFAULT_SLASH_PENALTY,
        }
    }
}
