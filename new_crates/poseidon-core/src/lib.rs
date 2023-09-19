//! Hashing with the [Poseidon] hash function.
//!
//! [Poseidon]: https://www.poseidon-hash.info/

use solana_poseidon_syscall_error::PoseidonSyscallError;

/// Length of Poseidon hash result.
pub const HASH_BYTES: usize = 32;

/// Configuration parameters for the Poseidon hash function.
///
/// The parameters of each configuration consist of:
///
/// - **Elliptic curve type**: This defines the prime field in which the
///   cryptographic operations are conducted.
/// - **S-Box**: The substitution box used in the cryptographic rounds.
/// - **Full rounds**: The number of full transformation rounds in the hash
///   function.
/// - **Partial rounds**: The number of partial transformation rounds in the
///   hash function.
///
/// Each configuration variant's name is composed of its elliptic curve type
/// followed by its S-Box specification.
#[repr(u64)]
pub enum Parameters {
    /// Configuration using the Barreto–Naehrig curve with an embedding degree
    /// of 12, defined over a 254-bit prime field.
    ///
    /// Configuration Details:
    /// - **S-Box**: \( x^5 \)
    /// - **Width**: \( 2 \leq t \leq 13 \)
    /// - **Inputs**: \( 1 \leq n \leq 12 \)
    /// - **Full rounds**: 8
    /// - **Partial rounds**: Depending on width: [56, 57, 56, 60, 60, 63, 64,
    ///   63, 60, 66, 60, 65]
    Bn254X5 = 0,
}

impl TryFrom<u64> for Parameters {
    type Error = PoseidonSyscallError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            x if x == Parameters::Bn254X5 as u64 => Ok(Parameters::Bn254X5),
            _ => Err(PoseidonSyscallError::InvalidParameters),
        }
    }
}

impl From<Parameters> for u64 {
    fn from(value: Parameters) -> Self {
        match value {
            Parameters::Bn254X5 => 0,
        }
    }
}

/// Endianness of inputs and result.
#[repr(u64)]
pub enum Endianness {
    /// Big-endian inputs and result.
    BigEndian = 0,
    /// Little-endian inputs and result.
    LittleEndian,
}

impl TryFrom<u64> for Endianness {
    type Error = PoseidonSyscallError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            x if x == Endianness::BigEndian as u64 => Ok(Endianness::BigEndian),
            x if x == Endianness::LittleEndian as u64 => Ok(Endianness::LittleEndian),
            _ => Err(PoseidonSyscallError::InvalidEndianness),
        }
    }
}

impl From<Endianness> for u64 {
    fn from(value: Endianness) -> Self {
        match value {
            Endianness::BigEndian => 0,
            Endianness::LittleEndian => 1,
        }
    }
}

/// Poseidon hash result.
#[repr(transparent)]
pub struct PoseidonHash(pub [u8; HASH_BYTES]);

impl PoseidonHash {
    pub fn new(hash_array: [u8; HASH_BYTES]) -> Self {
        Self(hash_array)
    }

    pub fn to_bytes(&self) -> [u8; HASH_BYTES] {
        self.0
    }
}
