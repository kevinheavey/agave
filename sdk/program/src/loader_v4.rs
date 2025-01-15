#[deprecated(since = "2.2.0", note = "Use solana-loader-v4-interface instead")]
pub use solana_loader_v4_interface::{
    state::{LoaderV4State, LoaderV4Status},
    DEPLOYMENT_COOLDOWN_IN_SLOTS,
};
pub use solana_sdk_ids::loader_v4::{check_id, id, ID};
