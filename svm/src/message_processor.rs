#![allow(deprecated)]
use {
    solana_program_runtime::invoke_context::InvokeContext,
    solana_sdk::{transaction::TransactionError, transaction_context::IndexOfAccount},
    solana_svm_message_processor::process_message,
    solana_svm_transaction::svm_message::SVMMessage,
    solana_timings::ExecuteTimings,
};

#[deprecated(
    since = "2.2.0",
    note = "Use solana-svm-message-processor crate instead."
)]
#[derive(Debug, Default, Clone, serde_derive::Deserialize, serde_derive::Serialize)]
pub struct MessageProcessor {}

#[cfg(feature = "frozen-abi")]
impl ::solana_frozen_abi::abi_example::AbiExample for MessageProcessor {
    fn example() -> Self {
        // MessageProcessor's fields are #[serde(skip)]-ed and not Serialize
        // so, just rely on Default anyway.
        MessageProcessor::default()
    }
}

impl MessageProcessor {
    /// Process a message.
    /// This method calls each instruction in the message over the set of loaded accounts.
    /// For each instruction it calls the program entrypoint method and verifies that the result of
    /// the call does not violate the bank's accounting rules.
    /// It returns a TransactionError if any instruction fails.
    pub fn process_message(
        message: &impl SVMMessage,
        program_indices: &[Vec<IndexOfAccount>],
        invoke_context: &mut InvokeContext,
        execute_timings: &mut ExecuteTimings,
        accumulated_consumed_units: &mut u64,
    ) -> Result<(), TransactionError> {
        process_message(
            message,
            program_indices,
            invoke_context,
            execute_timings,
            accumulated_consumed_units,
        )
    }
}
