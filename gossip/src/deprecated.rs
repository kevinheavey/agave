use solana_sdk::clock::Slot;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, AbiExample, AbiEnumVisitor)]
enum CompressionType {
    Uncompressed,
    GZip,
    BZip2,
}

impl Default for CompressionType {
    fn default() -> Self {
        Self::Uncompressed
    }
}

#[cfg_attr(feature = "frozen-abi", derive(AbiExample))]
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct EpochIncompleteSlots {
    first: Slot,
    compression: CompressionType,
    compressed_list: Vec<u8>,
}
