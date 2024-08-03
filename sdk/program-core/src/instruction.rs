//! Types for directing the execution of Solana programs.
//!
//! Every invocation of a Solana program executes a single instruction, as
//! defined by the [`Instruction`] type. An instruction is primarily a vector of
//! bytes, the contents of which are program-specific, and not interpreted by
//! the Solana runtime. This allows flexibility in how programs behave, how they
//! are controlled by client software, and what data encodings they use.
//!
//! Besides the instruction data, every account a program may read or write
//! while executing a given instruction is also included in `Instruction`, as
//! [`AccountMeta`] values. The runtime uses this information to efficiently
//! schedule execution of transactions.

#![allow(clippy::arithmetic_side_effects)]

#[cfg(feature = "bincode")]
use bincode::serialize;
#[cfg(feature = "borsh")]
use borsh::BorshSerialize;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;
use {crate::pubkey::Pubkey, core::fmt, solana_sanitize::Sanitize};
#[cfg(feature = "serde")]
use {
    serde_derive::{Deserialize, Serialize},
    solana_short_vec as short_vec,
};

/// Reasons the runtime might have rejected an instruction.
///
/// Members of this enum must not be removed, but new ones can be added.
/// Also, it is crucial that meta-information if any that comes along with
/// an error be consistent across software versions.  For example, it is
/// dangerous to include error strings from 3rd party crates because they could
/// change at any time and changes to them are difficult to detect.
#[cfg_attr(feature = "frozen-abi", derive(AbiExample, AbiEnumVisitor))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum InstructionError {
    /// Deprecated! Use CustomError instead!
    /// The program instruction returned an error
    GenericError,

    /// The arguments provided to a program were invalid
    InvalidArgument,

    /// An instruction's data contents were invalid
    InvalidInstructionData,

    /// An account's data contents was invalid
    InvalidAccountData,

    /// An account's data was too small
    AccountDataTooSmall,

    /// An account's balance was too small to complete the instruction
    InsufficientFunds,

    /// The account did not have the expected program id
    IncorrectProgramId,

    /// A signature was required but not found
    MissingRequiredSignature,

    /// An initialize instruction was sent to an account that has already been initialized.
    AccountAlreadyInitialized,

    /// An attempt to operate on an account that hasn't been initialized.
    UninitializedAccount,

    /// Program's instruction lamport balance does not equal the balance after the instruction
    UnbalancedInstruction,

    /// Program illegally modified an account's program id
    ModifiedProgramId,

    /// Program spent the lamports of an account that doesn't belong to it
    ExternalAccountLamportSpend,

    /// Program modified the data of an account that doesn't belong to it
    ExternalAccountDataModified,

    /// Read-only account's lamports modified
    ReadonlyLamportChange,

    /// Read-only account's data was modified
    ReadonlyDataModified,

    /// An account was referenced more than once in a single instruction
    // Deprecated, instructions can now contain duplicate accounts
    DuplicateAccountIndex,

    /// Executable bit on account changed, but shouldn't have
    ExecutableModified,

    /// Rent_epoch account changed, but shouldn't have
    RentEpochModified,

    /// The instruction expected additional account keys
    NotEnoughAccountKeys,

    /// Program other than the account's owner changed the size of the account data
    AccountDataSizeChanged,

    /// The instruction expected an executable account
    AccountNotExecutable,

    /// Failed to borrow a reference to account data, already borrowed
    AccountBorrowFailed,

    /// Account data has an outstanding reference after a program's execution
    AccountBorrowOutstanding,

    /// The same account was multiply passed to an on-chain program's entrypoint, but the program
    /// modified them differently.  A program can only modify one instance of the account because
    /// the runtime cannot determine which changes to pick or how to merge them if both are modified
    DuplicateAccountOutOfSync,

    /// Allows on-chain programs to implement program-specific error types and see them returned
    /// by the Solana runtime. A program-specific error may be any type that is represented as
    /// or serialized to a u32 integer.
    Custom(u32),

    /// The return value from the program was invalid.  Valid errors are either a defined builtin
    /// error value or a user-defined error in the lower 32 bits.
    InvalidError,

    /// Executable account's data was modified
    ExecutableDataModified,

    /// Executable account's lamports modified
    ExecutableLamportChange,

    /// Executable accounts must be rent exempt
    ExecutableAccountNotRentExempt,

    /// Unsupported program id
    UnsupportedProgramId,

    /// Cross-program invocation call depth too deep
    CallDepth,

    /// An account required by the instruction is missing
    MissingAccount,

    /// Cross-program invocation reentrancy not allowed for this instruction
    ReentrancyNotAllowed,

    /// Length of the seed is too long for address generation
    MaxSeedLengthExceeded,

    /// Provided seeds do not result in a valid address
    InvalidSeeds,

    /// Failed to reallocate account data of this length
    InvalidRealloc,

    /// Computational budget exceeded
    ComputationalBudgetExceeded,

    /// Cross-program invocation with unauthorized signer or writable account
    PrivilegeEscalation,

    /// Failed to create program execution environment
    ProgramEnvironmentSetupFailure,

    /// Program failed to complete
    ProgramFailedToComplete,

    /// Program failed to compile
    ProgramFailedToCompile,

    /// Account is immutable
    Immutable,

    /// Incorrect authority provided
    IncorrectAuthority,

    /// Failed to serialize or deserialize account data
    ///
    /// Warning: This error should never be emitted by the runtime.
    ///
    /// This error includes strings from the underlying 3rd party Borsh crate
    /// which can be dangerous because the error strings could change across
    /// Borsh versions. Only programs can use this error because they are
    /// consistent across Solana software versions.
    ///
    BorshIoError(String),

    /// An account does not have enough lamports to be rent-exempt
    AccountNotRentExempt,

    /// Invalid account owner
    InvalidAccountOwner,

    /// Program arithmetic overflowed
    ArithmeticOverflow,

    /// Unsupported sysvar
    UnsupportedSysvar,

    /// Illegal account owner
    IllegalOwner,

    /// Accounts data allocations exceeded the maximum allowed per transaction
    MaxAccountsDataAllocationsExceeded,

    /// Max accounts exceeded
    MaxAccountsExceeded,

    /// Max instruction trace length exceeded
    MaxInstructionTraceLengthExceeded,

    /// Builtin programs must consume compute units
    BuiltinProgramsMustConsumeComputeUnits,
    // Note: For any new error added here an equivalent ProgramError and its
    // conversions must also be added
}

impl std::error::Error for InstructionError {}

impl fmt::Display for InstructionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InstructionError::GenericError => f.write_str("generic instruction error"),
            InstructionError::InvalidArgument => f.write_str("invalid program argument"),
            InstructionError::InvalidInstructionData => f.write_str("invalid instruction data"),
            InstructionError::InvalidAccountData => {
                f.write_str("invalid account data for instruction")
            }
            InstructionError::AccountDataTooSmall => {
                f.write_str("account data too small for instruction")
            }
            InstructionError::InsufficientFunds => {
                f.write_str("insufficient funds for instruction")
            }
            InstructionError::IncorrectProgramId => {
                f.write_str("incorrect program id for instruction")
            }
            InstructionError::MissingRequiredSignature => {
                f.write_str("missing required signature for instruction")
            }
            InstructionError::AccountAlreadyInitialized => {
                f.write_str("instruction requires an uninitialized account")
            }
            InstructionError::UninitializedAccount => {
                f.write_str("instruction requires an initialized account")
            }
            InstructionError::UnbalancedInstruction => {
                f.write_str("sum of account balances before and after instruction do not match")
            }
            InstructionError::ModifiedProgramId => {
                f.write_str("instruction illegally modified the program id of an account")
            }
            InstructionError::ExternalAccountLamportSpend => {
                f.write_str("instruction spent from the balance of an account it does not own")
            }
            InstructionError::ExternalAccountDataModified => {
                f.write_str("instruction modified data of an account it does not own")
            }
            InstructionError::ReadonlyLamportChange => {
                f.write_str("instruction changed the balance of a read-only account")
            }
            InstructionError::ReadonlyDataModified => {
                f.write_str("instruction modified data of a read-only account")
            }
            InstructionError::DuplicateAccountIndex => {
                f.write_str("instruction contains duplicate accounts")
            }
            InstructionError::ExecutableModified => {
                f.write_str("instruction changed executable bit of an account")
            }
            InstructionError::RentEpochModified => {
                f.write_str("instruction modified rent epoch of an account")
            }
            InstructionError::NotEnoughAccountKeys => {
                f.write_str("insufficient account keys for instruction")
            }
            InstructionError::AccountDataSizeChanged => f.write_str(
                "program other than the account's owner changed the size of the account data",
            ),
            InstructionError::AccountNotExecutable => {
                f.write_str("instruction expected an executable account")
            }
            InstructionError::AccountBorrowFailed => f.write_str(
                "instruction tries to borrow reference for an account which is already borrowed",
            ),
            InstructionError::AccountBorrowOutstanding => {
                f.write_str("instruction left account with an outstanding borrowed reference")
            }
            InstructionError::DuplicateAccountOutOfSync => {
                f.write_str("instruction modifications of multiply-passed account differ")
            }
            InstructionError::Custom(num) => {
                write!(f, "custom program error: {num:#x}")
            }
            InstructionError::InvalidError => f.write_str("program returned invalid error code"),
            InstructionError::ExecutableDataModified => {
                f.write_str("instruction changed executable accounts data")
            }
            InstructionError::ExecutableLamportChange => {
                f.write_str("instruction changed the balance of an executable account")
            }
            InstructionError::ExecutableAccountNotRentExempt => {
                f.write_str("executable accounts must be rent exempt")
            }
            InstructionError::UnsupportedProgramId => f.write_str("Unsupported program id"),
            InstructionError::CallDepth => {
                f.write_str("Cross-program invocation call depth too deep")
            }
            InstructionError::MissingAccount => {
                f.write_str("An account required by the instruction is missing")
            }
            InstructionError::ReentrancyNotAllowed => {
                f.write_str("Cross-program invocation reentrancy not allowed for this instruction")
            }
            InstructionError::MaxSeedLengthExceeded => {
                f.write_str("Length of the seed is too long for address generation")
            }
            InstructionError::InvalidSeeds => {
                f.write_str("Provided seeds do not result in a valid address")
            }
            InstructionError::InvalidRealloc => f.write_str("Failed to reallocate account data"),
            InstructionError::ComputationalBudgetExceeded => {
                f.write_str("Computational budget exceeded")
            }
            InstructionError::PrivilegeEscalation => {
                f.write_str("Cross-program invocation with unauthorized signer or writable account")
            }
            InstructionError::ProgramEnvironmentSetupFailure => {
                f.write_str("Failed to create program execution environment")
            }
            InstructionError::ProgramFailedToComplete => f.write_str("Program failed to complete"),
            InstructionError::ProgramFailedToCompile => f.write_str("Program failed to compile"),
            InstructionError::Immutable => f.write_str("Account is immutable"),
            InstructionError::IncorrectAuthority => f.write_str("Incorrect authority provided"),
            InstructionError::BorshIoError(s) => {
                write!(f, "Failed to serialize or deserialize account data: {s}",)
            }
            InstructionError::AccountNotRentExempt => {
                f.write_str("An account does not have enough lamports to be rent-exempt")
            }
            InstructionError::InvalidAccountOwner => f.write_str("Invalid account owner"),
            InstructionError::ArithmeticOverflow => f.write_str("Program arithmetic overflowed"),
            InstructionError::UnsupportedSysvar => f.write_str("Unsupported sysvar"),
            InstructionError::IllegalOwner => f.write_str("Provided owner is not allowed"),
            InstructionError::MaxAccountsDataAllocationsExceeded => f.write_str(
                "Accounts data allocations exceeded the maximum allowed per transaction",
            ),
            InstructionError::MaxAccountsExceeded => f.write_str("Max accounts exceeded"),
            InstructionError::MaxInstructionTraceLengthExceeded => {
                f.write_str("Max instruction trace length exceeded")
            }
            InstructionError::BuiltinProgramsMustConsumeComputeUnits => {
                f.write_str("Builtin programs must consume compute units")
            }
        }
    }
}

/// A directive for a single invocation of a Solana program.
///
/// An instruction specifies which program it is calling, which accounts it may
/// read or modify, and additional data that serves as input to the program. One
/// or more instructions are included in transactions submitted by Solana
/// clients. Instructions are also used to describe [cross-program
/// invocations][cpi].
///
/// [cpi]: https://solana.com/docs/core/cpi
///
/// During execution, a program will receive a list of account data as one of
/// its arguments, in the same order as specified during `Instruction`
/// construction.
///
/// While Solana is agnostic to the format of the instruction data, it has
/// built-in support for serialization via [`borsh`] and [`bincode`].
///
/// [`borsh`]: https://docs.rs/borsh/latest/borsh/
/// [`bincode`]: https://docs.rs/bincode/latest/bincode/
///
/// # Specifying account metadata
///
/// When constructing an [`Instruction`], a list of all accounts that may be
/// read or written during the execution of that instruction must be supplied as
/// [`AccountMeta`] values.
///
/// Any account whose data may be mutated by the program during execution must
/// be specified as writable. During execution, writing to an account that was
/// not specified as writable will cause the transaction to fail. Writing to an
/// account that is not owned by the program will cause the transaction to fail.
///
/// Any account whose lamport balance may be mutated by the program during
/// execution must be specified as writable. During execution, mutating the
/// lamports of an account that was not specified as writable will cause the
/// transaction to fail. While _subtracting_ lamports from an account not owned
/// by the program will cause the transaction to fail, _adding_ lamports to any
/// account is allowed, as long is it is mutable.
///
/// Accounts that are not read or written by the program may still be specified
/// in an `Instruction`'s account list. These will affect scheduling of program
/// execution by the runtime, but will otherwise be ignored.
///
/// When building a transaction, the Solana runtime coalesces all accounts used
/// by all instructions in that transaction, along with accounts and permissions
/// required by the runtime, into a single account list. Some accounts and
/// account permissions required by the runtime to process a transaction are
/// _not_ required to be included in an `Instruction`s account list. These
/// include:
///
/// - The program ID &mdash; it is a separate field of `Instruction`
/// - The transaction's fee-paying account &mdash; it is added during [`Message`]
///   construction. A program may still require the fee payer as part of the
///   account list if it directly references it.
///
/// [`Message`]: crate::message::Message
///
/// Programs may require signatures from some accounts, in which case they
/// should be specified as signers during `Instruction` construction. The
/// program must still validate during execution that the account is a signer.
#[cfg(not(target_arch = "wasm32"))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Instruction {
    /// Pubkey of the program that executes this instruction.
    pub program_id: Pubkey,
    /// Metadata describing accounts that should be passed to the program.
    pub accounts: Vec<AccountMeta>,
    /// Opaque data passed to the program for its own interpretation.
    pub data: Vec<u8>,
}

/// wasm-bindgen version of the Instruction struct.
/// This duplication is required until https://github.com/rustwasm/wasm-bindgen/issues/3671
/// is fixed. This must not diverge from the regular non-wasm Instruction struct.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct Instruction {
    #[wasm_bindgen(skip)]
    pub program_id: Pubkey,
    #[wasm_bindgen(skip)]
    pub accounts: Vec<AccountMeta>,
    #[wasm_bindgen(skip)]
    pub data: Vec<u8>,
}

impl Instruction {
    #[cfg(feature = "borsh")]
    /// Create a new instruction from a value, encoded with [`borsh`].
    ///
    /// [`borsh`]: https://docs.rs/borsh/latest/borsh/
    ///
    /// `program_id` is the address of the program that will execute the instruction.
    /// `accounts` contains a description of all accounts that may be accessed by the program.
    ///
    /// Borsh serialization is often preferred over bincode as it has a stable
    /// [specification] and an [implementation in JavaScript][jsb], neither of
    /// which are true of bincode.
    ///
    /// [specification]: https://borsh.io/
    /// [jsb]: https://github.com/near/borsh-js
    ///
    /// # Examples
    ///
    /// ```
    /// # use solana_program_core::{
    /// #     pubkey::Pubkey,
    /// #     instruction::{AccountMeta, Instruction},
    /// # };
    /// # use borsh::{BorshSerialize, BorshDeserialize};
    /// #
    /// #[derive(BorshSerialize, BorshDeserialize)]
    /// # #[borsh(crate = "borsh")]
    /// pub struct MyInstruction {
    ///     pub lamports: u64,
    /// }
    ///
    /// pub fn create_instruction(
    ///     program_id: &Pubkey,
    ///     from: &Pubkey,
    ///     to: &Pubkey,
    ///     lamports: u64,
    /// ) -> Instruction {
    ///     let instr = MyInstruction { lamports };
    ///
    ///     Instruction::new_with_borsh(
    ///         *program_id,
    ///         &instr,
    ///         vec![
    ///             AccountMeta::new(*from, true),
    ///             AccountMeta::new(*to, false),
    ///         ],
    ///    )
    /// }
    /// ```
    pub fn new_with_borsh<T: BorshSerialize>(
        program_id: Pubkey,
        data: &T,
        accounts: Vec<AccountMeta>,
    ) -> Self {
        let data = borsh::to_vec(data).unwrap();
        Self {
            program_id,
            accounts,
            data,
        }
    }

    /// Create a new instruction from a value, encoded with [`bincode`].
    ///
    /// [`bincode`]: https://docs.rs/bincode/latest/bincode/
    ///
    /// `program_id` is the address of the program that will execute the instruction.
    /// `accounts` contains a description of all accounts that may be accessed by the program.
    ///
    /// # Examples
    ///
    /// ```
    /// # use solana_program_core::{
    /// #     pubkey::Pubkey,
    /// #     instruction::{AccountMeta, Instruction},
    /// # };
    /// # use serde::{Serialize, Deserialize};
    /// #
    /// #[derive(Serialize, Deserialize)]
    /// pub struct MyInstruction {
    ///     pub lamports: u64,
    /// }
    ///
    /// pub fn create_instruction(
    ///     program_id: &Pubkey,
    ///     from: &Pubkey,
    ///     to: &Pubkey,
    ///     lamports: u64,
    /// ) -> Instruction {
    ///     let instr = MyInstruction { lamports };
    ///
    ///     Instruction::new_with_bincode(
    ///         *program_id,
    ///         &instr,
    ///         vec![
    ///             AccountMeta::new(*from, true),
    ///             AccountMeta::new(*to, false),
    ///         ],
    ///    )
    /// }
    /// ```
    #[cfg(feature = "bincode")]
    pub fn new_with_bincode<T: serde::Serialize>(
        program_id: Pubkey,
        data: &T,
        accounts: Vec<AccountMeta>,
    ) -> Self {
        let data = serialize(data).unwrap();
        Self {
            program_id,
            accounts,
            data,
        }
    }

    /// Create a new instruction from a byte slice.
    ///
    /// `program_id` is the address of the program that will execute the instruction.
    /// `accounts` contains a description of all accounts that may be accessed by the program.
    ///
    /// The caller is responsible for ensuring the correct encoding of `data` as expected
    /// by the callee program.
    ///
    /// # Examples
    ///
    /// ```
    /// # use solana_program_core::{
    /// #     pubkey::Pubkey,
    /// #     instruction::{AccountMeta, Instruction},
    /// # };
    /// # use borsh::{io::Error, BorshSerialize, BorshDeserialize};
    /// #
    /// #[derive(BorshSerialize, BorshDeserialize)]
    /// # #[borsh(crate = "borsh")]
    /// pub struct MyInstruction {
    ///     pub lamports: u64,
    /// }
    ///
    /// pub fn create_instruction(
    ///     program_id: &Pubkey,
    ///     from: &Pubkey,
    ///     to: &Pubkey,
    ///     lamports: u64,
    /// ) -> Result<Instruction, Error> {
    ///     let instr = MyInstruction { lamports };
    ///
    ///     let mut instr_in_bytes: Vec<u8> = Vec::new();
    ///     instr.serialize(&mut instr_in_bytes)?;
    ///
    ///     Ok(Instruction::new_with_bytes(
    ///         *program_id,
    ///         &instr_in_bytes,
    ///         vec![
    ///             AccountMeta::new(*from, true),
    ///             AccountMeta::new(*to, false),
    ///         ],
    ///    ))
    /// }
    /// ```
    pub fn new_with_bytes(program_id: Pubkey, data: &[u8], accounts: Vec<AccountMeta>) -> Self {
        Self {
            program_id,
            accounts,
            data: data.to_vec(),
        }
    }
}

/// Addition that returns [`InstructionError::InsufficientFunds`] on overflow.
///
/// This is an internal utility function.
#[doc(hidden)]
pub fn checked_add(a: u64, b: u64) -> Result<u64, InstructionError> {
    a.checked_add(b).ok_or(InstructionError::InsufficientFunds)
}

/// Describes a single account read or written by a program during instruction
/// execution.
///
/// When constructing an [`Instruction`], a list of all accounts that may be
/// read or written during the execution of that instruction must be supplied.
/// Any account that may be mutated by the program during execution, either its
/// data or metadata such as held lamports, must be writable.
///
/// Note that because the Solana runtime schedules parallel transaction
/// execution around which accounts are writable, care should be taken that only
/// accounts which actually may be mutated are specified as writable. As the
/// default [`AccountMeta::new`] constructor creates writable accounts, this is
/// a minor hazard: use [`AccountMeta::new_readonly`] to specify that an account
/// is not writable.
#[repr(C)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct AccountMeta {
    /// An account's public key.
    pub pubkey: Pubkey,
    /// True if an `Instruction` requires a `Transaction` signature matching `pubkey`.
    pub is_signer: bool,
    /// True if the account data or metadata may be mutated during program execution.
    pub is_writable: bool,
}

impl AccountMeta {
    /// Construct metadata for a writable account.
    ///
    /// # Examples
    ///
    /// ```
    /// # use solana_program_core::{
    /// #     pubkey::Pubkey,
    /// #     instruction::{AccountMeta, Instruction},
    /// # };
    /// # use borsh::{BorshSerialize, BorshDeserialize};
    /// #
    /// # #[derive(BorshSerialize, BorshDeserialize)]
    /// # #[borsh(crate = "borsh")]
    /// # pub struct MyInstruction;
    /// #
    /// # let instruction = MyInstruction;
    /// # let from = Pubkey::new_unique();
    /// # let to = Pubkey::new_unique();
    /// # let program_id = Pubkey::new_unique();
    /// let instr = Instruction::new_with_borsh(
    ///     program_id,
    ///     &instruction,
    ///     vec![
    ///         AccountMeta::new(from, true),
    ///         AccountMeta::new(to, false),
    ///     ],
    /// );
    /// ```
    pub fn new(pubkey: Pubkey, is_signer: bool) -> Self {
        Self {
            pubkey,
            is_signer,
            is_writable: true,
        }
    }

    /// Construct metadata for a read-only account.
    ///
    /// # Examples
    ///
    /// ```
    /// # use solana_program_core::{
    /// #     pubkey::Pubkey,
    /// #     instruction::{AccountMeta, Instruction},
    /// # };
    /// # use borsh::{BorshSerialize, BorshDeserialize};
    /// #
    /// # #[derive(BorshSerialize, BorshDeserialize)]
    /// # #[borsh(crate = "borsh")]
    /// # pub struct MyInstruction;
    /// #
    /// # let instruction = MyInstruction;
    /// # let from = Pubkey::new_unique();
    /// # let to = Pubkey::new_unique();
    /// # let from_account_storage = Pubkey::new_unique();
    /// # let program_id = Pubkey::new_unique();
    /// let instr = Instruction::new_with_borsh(
    ///     program_id,
    ///     &instruction,
    ///     vec![
    ///         AccountMeta::new(from, true),
    ///         AccountMeta::new(to, false),
    ///         AccountMeta::new_readonly(from_account_storage, false),
    ///     ],
    /// );
    /// ```
    pub fn new_readonly(pubkey: Pubkey, is_signer: bool) -> Self {
        Self {
            pubkey,
            is_signer,
            is_writable: false,
        }
    }
}

/// A compact encoding of an instruction.
///
/// A `CompiledInstruction` is a component of a multi-instruction [`Message`],
/// which is the core of a Solana transaction. It is created during the
/// construction of `Message`. Most users will not interact with it directly.
///
/// [`Message`]: crate::message::Message
#[cfg_attr(feature = "frozen-abi", derive(AbiExample))]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "camelCase")
)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CompiledInstruction {
    /// Index into the transaction keys array indicating the program account that executes this instruction.
    pub program_id_index: u8,
    /// Ordered indices into the transaction keys array indicating which accounts to pass to the program.
    #[cfg_attr(feature = "serde", serde(with = "short_vec"))]
    pub accounts: Vec<u8>,
    /// The program input data.
    #[cfg_attr(feature = "serde", serde(with = "short_vec"))]
    pub data: Vec<u8>,
}

impl Sanitize for CompiledInstruction {}

impl CompiledInstruction {
    #[cfg(feature = "bincode")]
    pub fn new<T: serde::Serialize>(program_ids_index: u8, data: &T, accounts: Vec<u8>) -> Self {
        let data = serialize(data).unwrap();
        Self {
            program_id_index: program_ids_index,
            accounts,
            data,
        }
    }

    pub fn new_from_raw_parts(program_id_index: u8, data: Vec<u8>, accounts: Vec<u8>) -> Self {
        Self {
            program_id_index,
            accounts,
            data,
        }
    }

    pub fn program_id<'a>(&self, program_ids: &'a [Pubkey]) -> &'a Pubkey {
        &program_ids[self.program_id_index as usize]
    }
}

/// Use to query and convey information about the sibling instruction components
/// when calling the `sol_get_processed_sibling_instruction` syscall.
#[repr(C)]
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct ProcessedSiblingInstruction {
    /// Length of the instruction data
    pub data_len: u64,
    /// Number of AccountMeta structures
    pub accounts_len: u64,
}

/// Returns a sibling instruction from the processed sibling instruction list.
///
/// The processed sibling instruction list is a reverse-ordered list of
/// successfully processed sibling instructions. For example, given the call flow:
///
/// A
/// B -> C -> D
/// B -> E
/// B -> F
///
/// Then B's processed sibling instruction list is: `[A]`
/// Then F's processed sibling instruction list is: `[E, C]`
pub fn get_processed_sibling_instruction(index: usize) -> Option<Instruction> {
    #[cfg(target_os = "solana")]
    {
        let mut meta = ProcessedSiblingInstruction::default();
        let mut program_id = Pubkey::default();

        if 1 == unsafe {
            crate::syscalls::sol_get_processed_sibling_instruction(
                index as u64,
                &mut meta,
                &mut program_id,
                &mut u8::default(),
                &mut AccountMeta::default(),
            )
        } {
            let mut data = Vec::new();
            let mut accounts = Vec::new();
            data.resize_with(meta.data_len as usize, u8::default);
            accounts.resize_with(meta.accounts_len as usize, AccountMeta::default);

            let _ = unsafe {
                crate::syscalls::sol_get_processed_sibling_instruction(
                    index as u64,
                    &mut meta,
                    &mut program_id,
                    data.as_mut_ptr(),
                    accounts.as_mut_ptr(),
                )
            };

            Some(Instruction::new_with_bytes(program_id, &data, accounts))
        } else {
            None
        }
    }

    #[cfg(not(target_os = "solana"))]
    crate::program_stubs::sol_get_processed_sibling_instruction(index)
}

// Stack height when processing transaction-level instructions
pub const TRANSACTION_LEVEL_STACK_HEIGHT: usize = 1;

/// Get the current stack height, transaction-level instructions are height
/// TRANSACTION_LEVEL_STACK_HEIGHT, fist invoked inner instruction is height
/// TRANSACTION_LEVEL_STACK_HEIGHT + 1, etc...
pub fn get_stack_height() -> usize {
    #[cfg(target_os = "solana")]
    unsafe {
        crate::syscalls::sol_get_stack_height() as usize
    }

    #[cfg(not(target_os = "solana"))]
    {
        crate::program_stubs::sol_get_stack_height() as usize
    }
}
