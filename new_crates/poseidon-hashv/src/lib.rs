//! Hashing with the [Poseidon] hash function.
//!
//! [Poseidon]: https://www.poseidon-hash.info/

use {
    solana_poseidon_core::{Parameters, Endianness, PoseidonHash},
    solana_poseidon_syscall_error::PoseidonSyscallError,
};


/// Return a Poseidon hash for the given data with the given elliptic curve and
/// endianness.
///
/// # Examples
///
/// ```rust
/// use solana_program::poseidon::{hashv, Endianness, Parameters};
///
/// # fn test() {
/// let input1 = [1u8; 32];
/// let input2 = [2u8; 32];
///
/// let hash = hashv(Parameters::Bn254X5, Endianness::BigEndian, &[&input1, &input2]).unwrap();
/// assert_eq!(
///     hash.to_bytes(),
///     [
///         13, 84, 225, 147, 143, 138, 140, 28, 125, 235, 94, 3, 85, 242, 99, 25, 32, 123,
///         132, 254, 156, 162, 206, 27, 38, 231, 53, 200, 41, 130, 25, 144
///     ]
/// );
///
/// let hash = hashv(Parameters::Bn254X5, Endianness::LittleEndian, &[&input1, &input2]).unwrap();
/// assert_eq!(
///     hash.to_bytes(),
///     [
///         144, 25, 130, 41, 200, 53, 231, 38, 27, 206, 162, 156, 254, 132, 123, 32, 25, 99,
///         242, 85, 3, 94, 235, 125, 28, 140, 138, 143, 147, 225, 84, 13
///     ]
/// );
/// # }
/// ```
#[allow(unused_variables)]
pub fn hashv(
    // This parameter is not used currently, because we support only one curve
    // (BN254). It should be used in case we add more curves in the future.
    parameters: Parameters,
    endianness: Endianness,
    vals: &[&[u8]],
) -> Result<PoseidonHash, PoseidonSyscallError> {
    // Perform the calculation inline, calling this from within a program is
    // not supported.
    #[cfg(not(target_os = "solana"))]
    {
        use {
            ark_bn254::Fr,
            light_poseidon::{Poseidon, PoseidonBytesHasher},
        };

        let mut hasher =
            Poseidon::<Fr>::new_circom(vals.len()).map_err(PoseidonSyscallError::from)?;
        let res = match endianness {
            Endianness::BigEndian => hasher.hash_bytes_be(vals),
            Endianness::LittleEndian => hasher.hash_bytes_le(vals),
        }
        .map_err(PoseidonSyscallError::from)?;

        Ok(PoseidonHash(res))
    }
    // Call via a system call to perform the calculation.
    #[cfg(target_os = "solana")]
    {
        let mut hash_result = [0; HASH_BYTES];
        let result = unsafe {
            solana_syscall_core::sol_poseidon(
                parameters.into(),
                endianness.into(),
                vals as *const _ as *const u8,
                vals.len() as u64,
                &mut hash_result as *mut _ as *mut u8,
            )
        };

        match result {
            0 => Ok(PoseidonHash::new(hash_result)),
            e => Err(PoseidonSyscallError::from(e)),
        }
    }
}
