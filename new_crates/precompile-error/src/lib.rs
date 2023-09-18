use {solana_decode_error::DecodeError, thiserror::Error};

/// Precompile errors
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum PrecompileError {
    #[error("public key is not valid")]
    InvalidPublicKey,
    #[error("id is not valid")]
    InvalidRecoveryId,
    #[error("signature is not valid")]
    InvalidSignature,
    #[error("offset not valid")]
    InvalidDataOffsets,
    #[error("instruction is incorrect size")]
    InvalidInstructionDataSize,
}
impl<T> DecodeError<T> for PrecompileError {
    fn type_of() -> &'static str {
        "PrecompileError"
    }
}
