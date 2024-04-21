#![allow(clippy::arithmetic_side_effects, clippy::op_ref)]
//! Syscall operations for curve25519

pub mod curve_syscall_traits;
pub mod edwards;
#[cfg(not(target_os = "solana"))]
pub mod errors;
pub mod ristretto;
pub mod scalar;
