//! State for durable transaction nonces.

mod current;
pub use current::DurableNonce;
use solana_pubkey::Pubkey;

#[derive(Debug, Eq, PartialEq)]
pub enum AuthorizeNonceError {
    MissingRequiredSignature(/*account authority:*/ Pubkey),
    Uninitialized,
}
