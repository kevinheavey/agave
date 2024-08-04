//! Converting custom error codes to enums.

use num_traits::FromPrimitive;

/// Obsolete, will be deleted.
pub trait DecodeError<E> {
    fn decode_custom_error_to_enum(custom: u32) -> Option<E>
    where
        E: FromPrimitive,
    {
        E::from_u32(custom)
    }
    fn type_of() -> &'static str;
}
