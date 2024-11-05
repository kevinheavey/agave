//! The [stake native program][np].
//!
//! [np]: https://docs.solanalabs.com/runtime/sysvars#stakehistory

#[allow(deprecated)]
pub mod config;
pub mod instruction;
pub mod stake_flags;
pub mod state;
pub mod tools;

pub mod program {
    pub use solana_sdk_ids::stake::{check_id, id, ID};
}

/// The minimum number of epochs before stake account that is delegated to a delinquent vote
/// account may be unstaked with `StakeInstruction::DeactivateDelinquent`
pub const MINIMUM_DELINQUENT_EPOCHS_FOR_DEACTIVATION: usize = 5;
