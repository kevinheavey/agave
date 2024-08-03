//! Converting custom error codes to enums.

#[cfg(feature = "num-traits")]
use num_traits::FromPrimitive;

/// Obsolete, will be deleted.
pub trait DecodeError<E> {
    #[allow(unused)]
    fn decode_custom_error_to_enum(custom: u32) -> Option<E> {
        None
    }
    fn type_of() -> &'static str;
}
