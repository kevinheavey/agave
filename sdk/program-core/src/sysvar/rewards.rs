//! This sysvar is deprecated and unused.

#[cfg(feature = "bincode")]
use crate::sysvar::Sysvar;

crate::declare_sysvar_id!("SysvarRewards111111111111111111111111111111", Rewards);

#[repr(C)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Default, PartialEq)]
pub struct Rewards {
    pub validator_point_value: f64,
    pub unused: f64,
}
impl Rewards {
    pub fn new(validator_point_value: f64) -> Self {
        Self {
            validator_point_value,
            unused: 0.0,
        }
    }
}

#[cfg(feature = "bincode")]
impl Sysvar for Rewards {}
