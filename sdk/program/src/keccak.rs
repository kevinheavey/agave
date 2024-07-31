//! Hashing with the [keccak] (SHA-3) hash function.
//!
//! [keccak]: https://keccak.team/keccak.html

#[cfg(feature = "borsh")]
use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use {
    sha3::{Digest, Keccak256},
    solana_sanitize::Sanitize,
    std::{convert::TryFrom, fmt, str::{FromStr, from_utf8}},
    thiserror::Error,
};

pub const HASH_BYTES: usize = 32;
/// Maximum string length of a base58 encoded hash
const MAX_BASE58_LEN: usize = 44;
#[cfg_attr(feature = "frozen-abi", derive(AbiExample))]
#[cfg_attr(
    feature = "borsh",
    derive(BorshSerialize, BorshDeserialize, BorshSchema),
    borsh(crate = "borsh")
)]
#[derive(Serialize, Deserialize, Clone, Copy, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct Hash(pub [u8; HASH_BYTES]);

#[derive(Clone, Default)]
pub struct Hasher {
    hasher: Keccak256,
}

impl Hasher {
    pub fn hash(&mut self, val: &[u8]) {
        self.hasher.update(val);
    }
    pub fn hashv(&mut self, vals: &[&[u8]]) {
        for val in vals {
            self.hash(val);
        }
    }
    pub fn result(self) -> Hash {
        Hash(self.hasher.finalize().into())
    }
}

impl Sanitize for Hash {}

impl AsRef<[u8]> for Hash {
    fn as_ref(&self) -> &[u8] {
        &self.0[..]
    }
}

impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut out = [0u8; MAX_BASE58_LEN];
        let mut len = 0;
        five8::encode_32(&self.0, Some(&mut len), &mut out);
        write!(f, "{}", from_utf8(&out[..len as usize]).unwrap())
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut out = [0u8; MAX_BASE58_LEN];
        let mut len = 0;
        five8::encode_32(&self.0, Some(&mut len), &mut out);
        write!(f, "{}", from_utf8(&out[..len as usize]).unwrap())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ParseHashError {
    #[error("string decoded to wrong size for hash")]
    WrongSize,
    #[error("failed to decoded string to hash")]
    Invalid,
}

impl FromStr for Hash {
    type Err = ParseHashError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use five8_core::DecodeError::{TooLong, TooShort, OutputTooLong};
        if s.len() > MAX_BASE58_LEN {
            return Err(ParseHashError::WrongSize);
        }
        let mut out = [0u8; HASH_BYTES];
        five8::decode_32(s, &mut out).map_err(|e| match e {
            TooLong | TooShort | OutputTooLong => {
                ParseHashError::WrongSize
            }
            _ => ParseHashError::Invalid,
        })?;
        Ok(Self(out))
    }
}

impl Hash {
    pub fn new(hash_slice: &[u8]) -> Self {
        Hash(<[u8; HASH_BYTES]>::try_from(hash_slice).unwrap())
    }

    pub const fn new_from_array(hash_array: [u8; HASH_BYTES]) -> Self {
        Self(hash_array)
    }

    /// unique Hash for tests and benchmarks.
    pub fn new_unique() -> Self {
        use solana_atomic_u64::AtomicU64;
        static I: AtomicU64 = AtomicU64::new(1);

        let mut b = [0u8; HASH_BYTES];
        let i = I.fetch_add(1);
        b[0..8].copy_from_slice(&i.to_le_bytes());
        Self::new(&b)
    }

    pub fn to_bytes(self) -> [u8; HASH_BYTES] {
        self.0
    }
}

/// Return a Keccak256 hash for the given data.
pub fn hashv(vals: &[&[u8]]) -> Hash {
    // Perform the calculation inline, calling this from within a program is
    // not supported
    #[cfg(not(target_os = "solana"))]
    {
        let mut hasher = Hasher::default();
        hasher.hashv(vals);
        hasher.result()
    }
    // Call via a system call to perform the calculation
    #[cfg(target_os = "solana")]
    {
        let mut hash_result = [0; HASH_BYTES];
        unsafe {
            crate::syscalls::sol_keccak256(
                vals as *const _ as *const u8,
                vals.len() as u64,
                &mut hash_result as *mut _ as *mut u8,
            );
        }
        Hash::new_from_array(hash_result)
    }
}

/// Return a Keccak256 hash for the given data.
pub fn hash(val: &[u8]) -> Hash {
    hashv(&[val])
}

/// Return the hash of the given hash extended with the given value.
pub fn extend_and_hash(id: &Hash, val: &[u8]) -> Hash {
    let mut hash_data = id.as_ref().to_vec();
    hash_data.extend_from_slice(val);
    hash(&hash_data)
}
