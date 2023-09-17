//! Durable transaction nonces.

pub mod state;
pub use state::State;

pub use solana_nonce_core::NONCED_TX_MARKER_IX_INDEX;
