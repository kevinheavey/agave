use solana_hash::{hashv, Hash};

const DURABLE_NONCE_HASH_PREFIX: &[u8] = "DURABLE_NONCE".as_bytes();

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DurableNonce(Hash);

impl DurableNonce {
    pub fn from_blockhash(blockhash: &Hash) -> Self {
        Self(hashv(&[DURABLE_NONCE_HASH_PREFIX, blockhash.as_ref()]))
    }

    /// Hash value used as recent_blockhash field in Transactions.
    pub fn as_hash(&self) -> &Hash {
        &self.0
    }
}
