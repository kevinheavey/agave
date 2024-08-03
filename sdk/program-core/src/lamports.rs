//! Defines the [`LamportsError`] type.

use {crate::instruction::InstructionError, core::fmt};

#[derive(Debug)]
pub enum LamportsError {
    /// arithmetic underflowed
    ArithmeticUnderflow,

    /// arithmetic overflowed
    ArithmeticOverflow,
}

impl std::error::Error for LamportsError {}

impl fmt::Display for LamportsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LamportsError::ArithmeticUnderflow => f.write_str("Arithmetic underflowed"),
            LamportsError::ArithmeticOverflow => f.write_str("Arithmetic overflowed"),
        }
    }
}

impl From<LamportsError> for InstructionError {
    fn from(error: LamportsError) -> Self {
        match error {
            LamportsError::ArithmeticOverflow => InstructionError::ArithmeticOverflow,
            LamportsError::ArithmeticUnderflow => InstructionError::ArithmeticOverflow,
        }
    }
}
