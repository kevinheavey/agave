//! Information about the last restart slot (hard fork).

use {crate::clock::Slot, solana_sdk_macro::CloneZeroed};

#[repr(C)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, CloneZeroed, PartialEq, Eq, Default)]
pub struct LastRestartSlot {
    /// The last restart `Slot`.
    pub last_restart_slot: Slot,
}
