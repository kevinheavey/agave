//! Hashing with the [Poseidon] hash function.
//!
//! [Poseidon]: https://www.poseidon-hash.info/

use thiserror::Error;

#[derive(Error, Debug)]
pub enum PoseidonSyscallError {
    #[error("Invalid parameters.")]
    InvalidParameters,
    #[error("Invalid endianness.")]
    InvalidEndianness,
    #[error("Invalid number of inputs. Maximum allowed is 12.")]
    InvalidNumberOfInputs,
    #[error("Input is an empty slice.")]
    EmptyInput,
    #[error(
        "Invalid length of the input. The length matching the modulus of the prime field is 32."
    )]
    InvalidInputLength,
    #[error("Input is larger than the modulus of the prime field.")]
    InputLargerThanModulus,
    #[error("Failed to convert a vector of bytes into an array.")]
    VecToArray,
    #[error("Failed to convert the number of inputs from u64 to u8.")]
    U64Tou8,
    #[error("Invalid width. Choose a width between 2 and 16 for 1 to 15 inputs.")]
    InvalidWidthCircom,
    #[error("Unexpected error")]
    Unexpected,
}

impl From<u64> for PoseidonSyscallError {
    fn from(error: u64) -> Self {
        match error {
            1 => PoseidonSyscallError::InvalidParameters,
            2 => PoseidonSyscallError::InvalidEndianness,
            3 => PoseidonSyscallError::InvalidNumberOfInputs,
            4 => PoseidonSyscallError::EmptyInput,
            5 => PoseidonSyscallError::InvalidInputLength,
            6 => PoseidonSyscallError::InputLargerThanModulus,
            7 => PoseidonSyscallError::VecToArray,
            8 => PoseidonSyscallError::U64Tou8,
            9 => PoseidonSyscallError::InvalidWidthCircom,
            _ => PoseidonSyscallError::Unexpected,
        }
    }
}

impl From<PoseidonSyscallError> for u64 {
    fn from(error: PoseidonSyscallError) -> Self {
        match error {
            PoseidonSyscallError::InvalidParameters => 1,
            PoseidonSyscallError::InvalidEndianness => 2,
            PoseidonSyscallError::InvalidNumberOfInputs => 3,
            PoseidonSyscallError::EmptyInput => 4,
            PoseidonSyscallError::InvalidInputLength => 5,
            PoseidonSyscallError::InputLargerThanModulus => 6,
            PoseidonSyscallError::VecToArray => 7,
            PoseidonSyscallError::U64Tou8 => 8,
            PoseidonSyscallError::InvalidWidthCircom => 9,
            PoseidonSyscallError::Unexpected => 10,
        }
    }
}

#[cfg(not(target_os = "solana"))]
impl From<light_poseidon::PoseidonError> for PoseidonSyscallError {
    fn from(error: light_poseidon::PoseidonError) -> Self {
        use light_poseidon::PoseidonError;
        match error {
            PoseidonError::InvalidNumberOfInputs {
                inputs: _,
                max_limit: _,
                width: _,
            } => PoseidonSyscallError::InvalidNumberOfInputs,
            PoseidonError::EmptyInput => PoseidonSyscallError::EmptyInput,
            PoseidonError::InvalidInputLength {
                len: _,
                modulus_bytes_len: _,
            } => PoseidonSyscallError::InvalidInputLength,
            PoseidonError::InputLargerThanModulus => PoseidonSyscallError::InputLargerThanModulus,
            PoseidonError::VecToArray => PoseidonSyscallError::VecToArray,
            PoseidonError::U64Tou8 => PoseidonSyscallError::U64Tou8,
            PoseidonError::InvalidWidthCircom {
                width: _,
                max_limit: _,
            } => PoseidonSyscallError::InvalidWidthCircom,
        }
    }
}
