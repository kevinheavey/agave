//! Instructions and constructors for the system program.
//!
//! The system program is responsible for the creation of accounts and [nonce
//! accounts][na]. It is responsible for transferring lamports from accounts
//! owned by the system program, including typical user wallet accounts.
//!
//! [na]: https://docs.solana.com/implemented-proposals/durable-tx-nonces
//!
//! Account creation typically involves three steps: [`allocate`] space,
//! [`transfer`] lamports for rent, [`assign`] to its owning program. The
//! [`create_account`] function does all three at once. All new accounts must
//! contain enough lamports to be [rent exempt], or else the creation
//! instruction will fail.
//!
//! [rent exempt]: https://docs.solana.com/developing/programming-model/accounts#rent-exemption
//!
//! The accounts created by the system program can either be user-controlled,
//! where the secret keys are held outside the blockchain,
//! or they can be [program derived addresses][pda],
//! where write access to accounts is granted by an owning program.
//!
//! [pda]: crate::pubkey::Pubkey::find_program_address
//!
//! The system program ID is defined in [`system_program`].
//!
//! Most of the functions in this module construct an [`Instruction`], that must
//! be submitted to the runtime for execution, either via RPC, typically with
//! [`RpcClient`], or through [cross-program invocation][cpi].
//!
//! When invoking through CPI, the [`invoke`] or [`invoke_signed`] instruction
//! requires all account references to be provided explicitly as [`AccountInfo`]
//! values. The account references required are specified in the documentation
//! for the [`SystemInstruction`] variants for each system program instruction,
//! and these variants are linked from the documentation for their constructors.
//!
//! [`RpcClient`]: https://docs.rs/solana-client/latest/solana_client/rpc_client/struct.RpcClient.html
//! [cpi]: crate::program
//! [`invoke`]: crate::program::invoke
//! [`invoke_signed`]: crate::program::invoke_signed
//! [`AccountInfo`]: crate::account_info::AccountInfo

#[allow(deprecated)]
use {
    crate::nonce,
    solana_instruction::{AccountMeta, Instruction},
    solana_pubkey::Pubkey,
    solana_sysvar_core::{recent_blockhashes, rent},
};

pub use solana_system_instruction_core::{
    advance_nonce_account, allocate, allocate_with_seed, assign, assign_with_seed,
    authorize_nonce_account, create_account, create_account_with_seed, transfer, transfer_many,
    transfer_with_seed, upgrade_nonce_account, withdraw_nonce_account, SystemError,
    SystemInstruction, MAX_PERMITTED_ACCOUNTS_DATA_ALLOCATIONS_PER_TRANSACTION,
    MAX_PERMITTED_DATA_LENGTH,
};

pub fn create_nonce_account_with_seed(
    from_pubkey: &Pubkey,
    nonce_pubkey: &Pubkey,
    base: &Pubkey,
    seed: &str,
    authority: &Pubkey,
    lamports: u64,
) -> Vec<Instruction> {
    vec![
        create_account_with_seed(
            from_pubkey,
            nonce_pubkey,
            base,
            seed,
            lamports,
            nonce::State::size() as u64,
            &solana_native_programs::system_program::id(),
        ),
        Instruction::new_with_bincode(
            solana_native_programs::system_program::id(),
            &SystemInstruction::InitializeNonceAccount(*authority),
            vec![
                AccountMeta::new(*nonce_pubkey, false),
                #[allow(deprecated)]
                AccountMeta::new_readonly(recent_blockhashes::id(), false),
                AccountMeta::new_readonly(rent::id(), false),
            ],
        ),
    ]
}

/// Create an account containing a durable transaction nonce.
///
/// This function produces a vector of [`Instruction`]s which must be submitted
/// in a [`Transaction`] or [invoked] to take effect, containing a serialized
/// [`SystemInstruction::CreateAccount`] and
/// [`SystemInstruction::InitializeNonceAccount`].
///
/// [`Transaction`]: https://docs.rs/solana-sdk/latest/solana_sdk/transaction/struct.Transaction.html
/// [invoked]: crate::program::invoke
///
/// A [durable transaction nonce][dtn] is a special account that enables
/// execution of transactions that have been signed in the past.
///
/// Standard Solana transactions include a [recent blockhash][rbh] (sometimes
/// referred to as a _[nonce]_). During execution the Solana runtime verifies
/// the recent blockhash is approximately less than two minutes old, and that in
/// those two minutes no other identical transaction with the same blockhash has
/// been executed. These checks prevent accidental replay of transactions.
/// Consequently, it is not possible to sign a transaction, wait more than two
/// minutes, then successfully execute that transaction.
///
/// [dtn]: https://docs.solana.com/implemented-proposals/durable-tx-nonces
/// [rbh]: crate::message::Message::recent_blockhash
/// [nonce]: https://en.wikipedia.org/wiki/Cryptographic_nonce
///
/// Durable transaction nonces are an alternative to the standard recent
/// blockhash nonce. They are stored in accounts on chain, and every time they
/// are used their value is changed to a new value for their next use. The
/// runtime verifies that each durable nonce value is only used once, and there
/// are no restrictions on how "old" the nonce is. Because they are stored on
/// chain and require additional instructions to use, transacting with durable
/// transaction nonces is more expensive than with standard transactions.
///
/// The value of the durable nonce is itself a blockhash and is accessible via
/// the [`blockhash`] field of [`nonce::state::Data`], which is deserialized
/// from the nonce account data.
///
/// [`blockhash`]: crate::nonce::state::Data::blockhash
/// [`nonce::state::Data`]: crate::nonce::state::Data
///
/// The basic durable transaction nonce lifecycle is
///
/// 1) Create the nonce account with the `create_nonce_account` instruction.
/// 2) Submit specially-formed transactions that include the
///    [`advance_nonce_account`] instruction.
/// 3) Destroy the nonce account by withdrawing its lamports with the
///    [`withdraw_nonce_account`] instruction.
///
/// Nonce accounts have an associated _authority_ account, which is stored in
/// their account data, and can be changed with the [`authorize_nonce_account`]
/// instruction. The authority must sign transactions that include the
/// `advance_nonce_account`, `authorize_nonce_account` and
/// `withdraw_nonce_account` instructions.
///
/// Nonce accounts are owned by the system program.
///
/// This constructor creates a [`SystemInstruction::CreateAccount`] instruction
/// and a [`SystemInstruction::InitializeNonceAccount`] instruction.
///
/// # Required signers
///
/// The `from_pubkey` and `nonce_pubkey` signers must sign the transaction.
///
/// # Examples
///
/// Create a nonce account from an off-chain client:
///
/// ```
/// # use solana_program::example_mocks::solana_sdk;
/// # use solana_program::example_mocks::solana_rpc_client;
/// use solana_rpc_client::rpc_client::RpcClient;
/// use solana_sdk::{
/// #   pubkey::Pubkey,
///     signature::{Keypair, Signer},
///     system_instruction,
///     transaction::Transaction,
///     nonce::State,
/// };
/// use anyhow::Result;
///
/// fn submit_create_nonce_account_tx(
///     client: &RpcClient,
///     payer: &Keypair,
/// ) -> Result<()> {
///
///     let nonce_account = Keypair::new();
///
///     let nonce_rent = client.get_minimum_balance_for_rent_exemption(State::size())?;
///     let instr = system_instruction::create_nonce_account(
///         &payer.pubkey(),
///         &nonce_account.pubkey(),
///         &payer.pubkey(), // Make the fee payer the nonce account authority
///         nonce_rent,
///     );
///
///     let mut tx = Transaction::new_with_payer(&instr, Some(&payer.pubkey()));
///
///     let blockhash = client.get_latest_blockhash()?;
///     tx.try_sign(&[&nonce_account, payer], blockhash)?;
///
///     client.send_and_confirm_transaction(&tx)?;
///
///     Ok(())
/// }
/// #
/// # let client = RpcClient::new(String::new());
/// # let payer = Keypair::new();
/// # submit_create_nonce_account_tx(&client, &payer)?;
/// #
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn create_nonce_account(
    from_pubkey: &Pubkey,
    nonce_pubkey: &Pubkey,
    authority: &Pubkey,
    lamports: u64,
) -> Vec<Instruction> {
    vec![
        create_account(
            from_pubkey,
            nonce_pubkey,
            lamports,
            nonce::State::size() as u64,
            &solana_native_programs::system_program::id(),
        ),
        Instruction::new_with_bincode(
            solana_native_programs::system_program::id(),
            &SystemInstruction::InitializeNonceAccount(*authority),
            vec![
                AccountMeta::new(*nonce_pubkey, false),
                #[allow(deprecated)]
                AccountMeta::new_readonly(recent_blockhashes::id(), false),
                AccountMeta::new_readonly(rent::id(), false),
            ],
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_nonce_account() {
        let from_pubkey = Pubkey::new_unique();
        let nonce_pubkey = Pubkey::new_unique();
        let authorized = nonce_pubkey;
        let ixs = create_nonce_account(&from_pubkey, &nonce_pubkey, &authorized, 42);
        assert_eq!(ixs.len(), 2);
        let ix = &ixs[0];
        assert_eq!(ix.program_id, solana_native_programs::system_program::id());
        let pubkeys: Vec<_> = ix.accounts.iter().map(|am| am.pubkey).collect();
        assert!(pubkeys.contains(&from_pubkey));
        assert!(pubkeys.contains(&nonce_pubkey));
    }
}
