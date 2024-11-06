#[deprecated(since = "2.2.0", note = "Use `solana-keypair` crate instead")]
pub use solana_keypair::*;
#[deprecated(since = "2.2.0", note = "Use `solana-seed-derivable` crate instead")]
pub use solana_seed_derivable::keypair_from_seed_and_derivation_path;
#[deprecated(since = "2.2.0", note = "Use `solana-seed-phrase` crate instead")]
pub use solana_seed_phrase::generate_seed_from_seed_phrase_and_passphrase;
#[deprecated(since = "2.2.0", note = "Use `solana-signer` crate instead")]
pub use solana_signer::*;
