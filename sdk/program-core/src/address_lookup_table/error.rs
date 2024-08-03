use core::fmt;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AddressLookupError {
    /// Attempted to lookup addresses from a table that does not exist
    LookupTableAccountNotFound,

    /// Attempted to lookup addresses from an account owned by the wrong program
    InvalidAccountOwner,

    /// Attempted to lookup addresses from an invalid account
    InvalidAccountData,

    /// Address lookup contains an invalid index
    InvalidLookupIndex,
}

impl std::error::Error for AddressLookupError {}

impl fmt::Display for AddressLookupError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AddressLookupError::LookupTableAccountNotFound => {
                f.write_str("Attempted to lookup addresses from a table that does not exist")
            }
            AddressLookupError::InvalidAccountOwner => f.write_str(
                "Attempted to lookup addresses from an account owned by the wrong program",
            ),
            AddressLookupError::InvalidAccountData => {
                f.write_str("Attempted to lookup addresses from an invalid account")
            }
            AddressLookupError::InvalidLookupIndex => {
                f.write_str("Address lookup contains an invalid index")
            }
        }
    }
}
