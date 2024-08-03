use {
    super::v0::{LoadedAddresses, MessageAddressTableLookup},
    core::fmt,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AddressLoaderError {
    /// Address loading from lookup tables is disabled
    Disabled,

    /// Failed to load slot hashes sysvar
    SlotHashesSysvarNotFound,

    /// Attempted to lookup addresses from a table that does not exist
    LookupTableAccountNotFound,

    /// Attempted to lookup addresses from an account owned by the wrong program
    InvalidAccountOwner,

    /// Attempted to lookup addresses from an invalid account
    InvalidAccountData,

    /// Address lookup contains an invalid index
    InvalidLookupIndex,
}

impl std::error::Error for AddressLoaderError {}

impl fmt::Display for AddressLoaderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AddressLoaderError::Disabled => {
                f.write_str("Address loading from lookup tables is disabled")
            }
            AddressLoaderError::SlotHashesSysvarNotFound => {
                f.write_str("Failed to load slot hashes sysvar")
            }
            AddressLoaderError::LookupTableAccountNotFound => {
                f.write_str("Attempted to lookup addresses from a table that does not exist")
            }
            AddressLoaderError::InvalidAccountOwner => f.write_str(
                "Attempted to lookup addresses from an account owned by the wrong program",
            ),
            AddressLoaderError::InvalidAccountData => {
                f.write_str("Attempted to lookup addresses from an invalid account")
            }
            AddressLoaderError::InvalidLookupIndex => {
                f.write_str("Address lookup contains an invalid index")
            }
        }
    }
}

pub trait AddressLoader: Clone {
    fn load_addresses(
        self,
        lookups: &[MessageAddressTableLookup],
    ) -> Result<LoadedAddresses, AddressLoaderError>;
}

#[derive(Clone)]
pub enum SimpleAddressLoader {
    Disabled,
    Enabled(LoadedAddresses),
}

impl AddressLoader for SimpleAddressLoader {
    fn load_addresses(
        self,
        _lookups: &[MessageAddressTableLookup],
    ) -> Result<LoadedAddresses, AddressLoaderError> {
        match self {
            Self::Disabled => Err(AddressLoaderError::Disabled),
            Self::Enabled(loaded_addresses) => Ok(loaded_addresses),
        }
    }
}
