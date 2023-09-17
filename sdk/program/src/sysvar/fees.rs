//! Current cluster fees.
//!
//! The _fees sysvar_ provides access to the [`Fees`] type, which contains the
//! current [`FeeCalculator`].
//!
//! [`Fees`] implements [`Sysvar::get`] and can be loaded efficiently without
//! passing the sysvar account ID to the program.
//!
//! This sysvar is deprecated and will not be available in the future.
//! Transaction fees should be determined with the [`getFeeForMessage`] RPC
//! method. For additional context see the [Comprehensive Compute Fees
//! proposal][ccf].
//!
//! [`getFeeForMessage`]: https://docs.solana.com/developing/clients/jsonrpc-api#getfeeformessage
//! [ccf]: https://docs.solana.com/proposals/comprehensive-compute-fees
//!
//! See also the Solana [documentation on the fees sysvar][sdoc].
//!
//! [sdoc]: https://docs.solana.com/developing/runtime-facilities/sysvars#fees

#![allow(deprecated)]

pub use solana_sysvar_core::fees::{check_id, id};
use {
    crate::fee_calculator::FeeCalculator,
    solana_msg_and_friends::program_error::ProgramError,
    solana_sdk_macro::CloneZeroed,
    solana_sysvar_core::{impl_sysvar_get, Sysvar},
};

/// Transaction fees.
#[deprecated(
    since = "1.9.0",
    note = "Please do not use, will no longer be available in the future"
)]
#[repr(C)]
#[derive(Serialize, Deserialize, Debug, CloneZeroed, Default, PartialEq, Eq)]
pub struct Fees {
    pub fee_calculator: FeeCalculator,
}

impl Fees {
    pub fn new(fee_calculator: &FeeCalculator) -> Self {
        #[allow(deprecated)]
        Self {
            fee_calculator: *fee_calculator,
        }
    }
}

impl Sysvar for Fees {
    impl_sysvar_get!(sol_get_fees_sysvar);
}

impl solana_sysvar_core::SysvarId for Fees {
    fn id() -> solana_pubkey::Pubkey {
        #[allow(deprecated)]
        id()
    }

    fn check_id(pubkey: &solana_pubkey::Pubkey) -> bool {
        #[allow(deprecated)]
        check_id(pubkey)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clone() {
        let fees = Fees {
            fee_calculator: FeeCalculator {
                lamports_per_signature: 1,
            },
        };
        let cloned_fees = fees.clone();
        assert_eq!(cloned_fees, fees);
    }

    #[test]
    fn test_sysvar_id() {
        assert!(
            solana_sysvar_core::is_sysvar_id(&id()),
            "sysvar::is_sysvar_id() doesn't know about Fees"
        );
    }
}
