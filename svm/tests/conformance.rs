use {
    crate::{
        mock_bank::{MockBankCallback, MockForkGraph},
        transaction_builder::SanitizedTransactionBuilder,
    },
    lazy_static::lazy_static,
    prost::Message,
    solana_bpf_loader_program::syscalls::create_program_runtime_environment_v1,
    solana_compute_budget::compute_budget::ComputeBudget,
    solana_log_collector::LogCollector,
    solana_program_runtime::{
        invoke_context::{EnvironmentConfig, InvokeContext},
        loaded_programs::{ProgramCacheEntry, ProgramCacheForTxBatch, ProgramRuntimeEnvironments},
        solana_rbpf::{
            program::{BuiltinProgram, FunctionRegistry},
            vm::Config,
        },
    },
    solana_sdk::{
        account::{AccountSharedData, ReadableAccount, WritableAccount},
        bpf_loader_upgradeable,
        feature_set::{FeatureSet, FEATURE_NAMES},
        hash::Hash,
        instruction::AccountMeta,
        message::SanitizedMessage,
        pubkey::Pubkey,
        rent::Rent,
        signature::Signature,
        transaction::TransactionError,
        transaction_context::{
            ExecutionRecord, IndexOfAccount, InstructionAccount, TransactionAccount,
            TransactionContext,
        },
    },
    solana_svm::{
        account_loader::CheckedTransactionDetails,
        program_loader,
        transaction_processing_callback::TransactionProcessingCallback,
        transaction_processing_result::TransactionProcessingResultExtensions,
        transaction_processor::{
            ExecutionRecordingConfig, TransactionBatchProcessor, TransactionProcessingConfig,
            TransactionProcessingEnvironment,
        },
    },
    solana_svm_conformance::proto::{InstrEffects, InstrFixture},
    solana_timings::ExecuteTimings,
    std::{
        collections::{HashMap, HashSet},
        env,
        ffi::OsString,
        fs::{self, File},
        io::Read,
        path::PathBuf,
        process::Command,
        sync::{Arc, RwLock},
    },
};
// #[cfg(target_arch = "wasm32")]
// use wasm_bindgen_test::wasm_bindgen_test as test;

mod mock_bank;
mod transaction_builder;

const fn feature_u64(feature: &Pubkey) -> u64 {
    let feature_id = feature.to_bytes();
    feature_id[0] as u64
        | (feature_id[1] as u64) << 8
        | (feature_id[2] as u64) << 16
        | (feature_id[3] as u64) << 24
        | (feature_id[4] as u64) << 32
        | (feature_id[5] as u64) << 40
        | (feature_id[6] as u64) << 48
        | (feature_id[7] as u64) << 56
}

lazy_static! {
    static ref INDEXED_FEATURES: HashMap<u64, Pubkey> = {
        FEATURE_NAMES
            .iter()
            .map(|(pubkey, _)| (feature_u64(pubkey), *pubkey))
            .collect()
    };
}

fn setup() -> PathBuf {
    let mut dir = env::current_dir().unwrap();
    dir.push("test-vectors");
    if !dir.exists() {
        std::println!("Cloning test-vectors ...");
        Command::new("git")
            .args([
                "clone",
                "https://github.com/firedancer-io/test-vectors",
                dir.as_os_str().to_str().unwrap(),
            ])
            .status()
            .expect("Failed to download test-vectors");

        std::println!("Checking out commit 90a8ad069f8a07d2fdad3cf03b3fb486a00fe988");
        Command::new("git")
            .current_dir(&dir)
            .args(["checkout", "90a8ad069f8a07d2fdad3cf03b3fb486a00fe988"])
            .status()
            .expect("Failed to checkout to proper test-vector commit");

        std::println!("Setup done!");
    }

    dir
}

fn cleanup() {
    let mut dir = env::current_dir().unwrap();
    dir.push("test-vectors");

    if dir.exists() {
        fs::remove_dir_all(dir).expect("Failed to delete test-vectors repository");
    }
}

#[test]
fn execute_fixtures() {
    // bpf-loader tests
    let base_dir = [
        include_bytes!("../../../test-vectors/instr/fixtures/bpf-loader/0732b78f6e65637a.bin.fix").as_slice(),
        include_bytes!("../../../test-vectors/instr/fixtures/bpf-loader/17cc2a4cb4e52b6b.bin.fix").as_slice(),
        include_bytes!("../../../test-vectors/instr/fixtures/bpf-loader/57313dcd67df8102.bin.fix").as_slice(),
        include_bytes!("../../../test-vectors/instr/fixtures/bpf-loader/997ec069a1e03f7b.bin.fix").as_slice(),
        include_bytes!("../../../test-vectors/instr/fixtures/bpf-loader/bf05a479df709e92.bin.fix").as_slice(),
        include_bytes!("../../../test-vectors/instr/fixtures/bpf-loader/11a9c773abf6946e.bin.fix").as_slice(),
        include_bytes!("../../../test-vectors/instr/fixtures/bpf-loader/4dc5943160201c33.bin.fix").as_slice(),
        include_bytes!("../../../test-vectors/instr/fixtures/bpf-loader/856b6d181608a874.bin.fix").as_slice(),
        include_bytes!("../../../test-vectors/instr/fixtures/bpf-loader/9cbe226ebd08f876.bin.fix").as_slice(),
        include_bytes!("../../../test-vectors/instr/fixtures/bpf-loader/c7c38bb8a66a7e05.bin.fix").as_slice(),
    ];
    run_from_folder(&base_dir);
}

fn run_from_folder(base_dir: &[&[u8]]) {
    for path in base_dir {
        let fixture = InstrFixture::decode(*path).unwrap();
        run_fixture(fixture, false);
    }
}

fn run_fixture(fixture: InstrFixture, execute_as_instr: bool) {
    let input = fixture.input.unwrap();
    let output = fixture.output.as_ref().unwrap();

    let mut transaction_builder = SanitizedTransactionBuilder::default();
    let program_id = Pubkey::new_from_array(input.program_id.try_into().unwrap());
    let mut accounts: Vec<AccountMeta> = Vec::with_capacity(input.instr_accounts.len());
    let mut signatures: HashMap<Pubkey, Signature> =
        HashMap::with_capacity(input.instr_accounts.len());

    for item in input.instr_accounts {
        let pubkey = Pubkey::new_from_array(
            input.accounts[item.index as usize]
                .address
                .clone()
                .try_into()
                .unwrap(),
        );
        accounts.push(AccountMeta {
            pubkey,
            is_signer: item.is_signer,
            is_writable: item.is_writable,
        });

        if item.is_signer {
            signatures.insert(pubkey, Signature::new_unique());
        }
    }

    transaction_builder.create_instruction(program_id, accounts, signatures, input.data);

    let mut feature_set = FeatureSet::default();
    if let Some(features) = &input.epoch_context.as_ref().unwrap().features {
        for id in &features.features {
            if let Some(pubkey) = INDEXED_FEATURES.get(id) {
                feature_set.activate(pubkey, 0);
            }
        }
    }

    let mut fee_payer = Pubkey::new_unique();
    let mut mock_bank = MockBankCallback::default();
    {
        let mut account_data_map = mock_bank.account_shared_data.write().unwrap();
        for item in input.accounts {
            let pubkey = Pubkey::new_from_array(item.address.try_into().unwrap());
            let mut account_data = AccountSharedData::default();
            account_data.set_lamports(item.lamports);
            account_data.set_data(item.data);
            account_data.set_owner(Pubkey::new_from_array(item.owner.try_into().unwrap()));
            account_data.set_executable(item.executable);
            account_data.set_rent_epoch(item.rent_epoch);

            account_data_map.insert(pubkey, account_data);
        }
        let mut account_data = AccountSharedData::default();
        account_data.set_lamports(800000);

        while account_data_map.contains_key(&fee_payer) {
            // The fee payer must not coincide with any of the previous accounts
            fee_payer = Pubkey::new_unique();
        }
        account_data_map.insert(fee_payer, account_data);
    }

    let Ok(transaction) =
        transaction_builder.build(Hash::default(), (fee_payer, Signature::new_unique()), false)
    else {
        // If we can't build a sanitized transaction,
        // the output must be a failed instruction as well
        assert_ne!(output.result, 0);
        return;
    };

    let transactions = vec![transaction];
    let transaction_check = vec![Ok(CheckedTransactionDetails {
        nonce: None,
        lamports_per_signature: 30,
    })];

    let compute_budget = ComputeBudget {
        compute_unit_limit: input.cu_avail,
        ..ComputeBudget::default()
    };

    let v1_environment =
        create_program_runtime_environment_v1(&feature_set, &compute_budget, false, false).unwrap();

    mock_bank.override_feature_set(feature_set);
    let batch_processor = TransactionBatchProcessor::<MockForkGraph>::new(42, 2, HashSet::new());

    let fork_graph = Arc::new(RwLock::new(MockForkGraph {}));
    {
        let mut program_cache = batch_processor.program_cache.write().unwrap();
        program_cache.environments = ProgramRuntimeEnvironments {
            program_runtime_v1: Arc::new(v1_environment),
            program_runtime_v2: Arc::new(BuiltinProgram::new_loader(
                Config::default(),
                FunctionRegistry::default(),
            )),
        };
        program_cache.fork_graph = Some(Arc::downgrade(&fork_graph.clone()));
    }

    batch_processor.fill_missing_sysvar_cache_entries(&mock_bank);
    register_builtins(&batch_processor, &mock_bank);

    #[allow(deprecated)]
    let (blockhash, lamports_per_signature) = batch_processor
        .sysvar_cache()
        .get_recent_blockhashes()
        .ok()
        .and_then(|x| (*x).last().cloned())
        .map(|x| (x.blockhash, x.fee_calculator.lamports_per_signature))
        .unwrap_or_default();

    let recording_config = ExecutionRecordingConfig {
        enable_log_recording: true,
        enable_return_data_recording: true,
        enable_cpi_recording: false,
    };
    let processor_config = TransactionProcessingConfig {
        account_overrides: None,
        check_program_modification_slot: false,
        compute_budget: None,
        log_messages_bytes_limit: None,
        limit_to_load_programs: true,
        recording_config,
        transaction_account_lock_limit: None,
    };

    if execute_as_instr {
        execute_fixture_as_instr(
            &mock_bank,
            &batch_processor,
            transactions[0].message(),
            compute_budget,
            output,
            input.cu_avail,
        );
        return;
    }

    let result = batch_processor.load_and_execute_sanitized_transactions(
        &mock_bank,
        &transactions,
        transaction_check,
        &TransactionProcessingEnvironment {
            blockhash,
            lamports_per_signature,
            ..Default::default()
        },
        &processor_config,
    );

    // Assert that the transaction has worked without errors.
    if let Err(err) = result.processing_results[0].flattened_result() {
        if matches!(err, TransactionError::InsufficientFundsForRent { .. }) {
            // This is a transaction error not an instruction error, so execute the instruction
            // instead.
            execute_fixture_as_instr(
                &mock_bank,
                &batch_processor,
                transactions[0].message(),
                compute_budget,
                output,
                input.cu_avail,
            );
            return;
        }

        assert_ne!(
            output.result, 0,
        );
        return;
    }

    let executed_tx = result.processing_results[0]
        .processed_transaction()
        .unwrap();
    let execution_details = &executed_tx.execution_details;
    let loaded_accounts = &executed_tx.loaded_transaction.accounts;
    verify_accounts_and_data(
        loaded_accounts,
        output,
        execution_details.executed_units,
        input.cu_avail,
        execution_details
            .return_data
            .as_ref()
            .map(|x| &x.data)
            .unwrap_or(&Vec::new()),
    );
}

fn register_builtins(
    batch_processor: &TransactionBatchProcessor<MockForkGraph>,
    mock_bank: &MockBankCallback,
) {
    let bpf_loader = "solana_bpf_loader_upgradeable_program";
    batch_processor.add_builtin(
        mock_bank,
        bpf_loader_upgradeable::id(),
        bpf_loader,
        ProgramCacheEntry::new_builtin(
            0,
            bpf_loader.len(),
            solana_bpf_loader_program::Entrypoint::vm,
        ),
    );

    let system_program = "system_program";
    batch_processor.add_builtin(
        mock_bank,
        solana_system_program::id(),
        system_program,
        ProgramCacheEntry::new_builtin(
            0,
            system_program.len(),
            solana_system_program::system_processor::Entrypoint::vm,
        ),
    );
}

fn execute_fixture_as_instr(
    mock_bank: &MockBankCallback,
    batch_processor: &TransactionBatchProcessor<MockForkGraph>,
    sanitized_message: &SanitizedMessage,
    compute_budget: ComputeBudget,
    output: &InstrEffects,
    cu_avail: u64,
) {
    let rent = if let Ok(rent) = batch_processor.sysvar_cache().get_rent() {
        (*rent).clone()
    } else {
        Rent::default()
    };

    let transaction_accounts: Vec<TransactionAccount> = sanitized_message
        .account_keys()
        .iter()
        .map(|key| (*key, mock_bank.get_account_shared_data(key).unwrap()))
        .collect();

    let mut transaction_context = TransactionContext::new(
        transaction_accounts,
        rent,
        compute_budget.max_instruction_stack_depth,
        compute_budget.max_instruction_trace_length,
    );

    let mut loaded_programs = ProgramCacheForTxBatch::new(
        42,
        batch_processor
            .program_cache
            .read()
            .unwrap()
            .environments
            .clone(),
        None,
        2,
    );

    let program_idx = sanitized_message.instructions()[0].program_id_index as usize;
    let program_id = *sanitized_message.account_keys().get(program_idx).unwrap();

    let loaded_program = program_loader::load_program_with_pubkey(
        mock_bank,
        &batch_processor.get_environments_for_epoch(2).unwrap(),
        &program_id,
        42,
        false,
    )
    .unwrap();

    loaded_programs.replenish(program_id, loaded_program);
    loaded_programs.replenish(
        solana_system_program::id(),
        Arc::new(ProgramCacheEntry::new_builtin(
            0u64,
            0usize,
            solana_system_program::system_processor::Entrypoint::vm,
        )),
    );

    let log_collector = LogCollector::new_ref();

    let sysvar_cache = &batch_processor.sysvar_cache();
    let env_config = EnvironmentConfig::new(
        Hash::default(),
        None,
        None,
        mock_bank.feature_set.clone(),
        0,
        sysvar_cache,
    );

    let mut invoke_context = InvokeContext::new(
        &mut transaction_context,
        &mut loaded_programs,
        env_config,
        Some(log_collector.clone()),
        compute_budget,
    );

    let mut instruction_accounts: Vec<InstructionAccount> =
        Vec::with_capacity(sanitized_message.instructions()[0].accounts.len());

    for (instruction_acct_idx, index_txn) in sanitized_message.instructions()[0]
        .accounts
        .iter()
        .enumerate()
    {
        let index_in_callee = sanitized_message.instructions()[0]
            .accounts
            .get(0..instruction_acct_idx)
            .unwrap()
            .iter()
            .position(|idx| *idx == *index_txn)
            .unwrap_or(instruction_acct_idx);

        instruction_accounts.push(InstructionAccount {
            index_in_transaction: *index_txn as IndexOfAccount,
            index_in_caller: *index_txn as IndexOfAccount,
            index_in_callee: index_in_callee as IndexOfAccount,
            is_signer: sanitized_message.is_signer(*index_txn as usize),
            is_writable: sanitized_message.is_writable(*index_txn as usize),
        });
    }

    let mut compute_units_consumed = 0u64;
    let mut timings = ExecuteTimings::default();
    let result = invoke_context.process_instruction(
        &sanitized_message.instructions()[0].data,
        &instruction_accounts,
        &[program_idx as IndexOfAccount],
        &mut compute_units_consumed,
        &mut timings,
    );

    if output.result == 0 {
        assert!(
            result.is_ok(),
        );
    } else {
        assert!(
            result.is_err(),
        );
        return;
    }

    let ExecutionRecord {
        accounts,
        return_data,
        ..
    } = transaction_context.into();

    verify_accounts_and_data(
        &accounts,
        output,
        compute_units_consumed,
        cu_avail,
        &return_data.data,
    );
}

fn verify_accounts_and_data(
    accounts: &[TransactionAccount],
    output: &InstrEffects,
    consumed_units: u64,
    cu_avail: u64,
    return_data: &Vec<u8>,
) {
    let idx_map: HashMap<Pubkey, usize> = accounts
        .iter()
        .enumerate()
        .map(|(idx, item)| (item.0, idx))
        .collect();

    for item in &output.modified_accounts {
        let pubkey = Pubkey::new_from_array(item.address.clone().try_into().unwrap());
        let index = *idx_map
            .get(&pubkey)
            .expect("Account not in expected results");

        let received_data = &accounts[index].1;

        assert_eq!(
            received_data.lamports(),
            item.lamports,
        );
        assert_eq!(
            received_data.data(),
            item.data.as_slice(),
        );
        assert_eq!(
            received_data.owner(),
            &Pubkey::new_from_array(item.owner.clone().try_into().unwrap()),
        );
        assert_eq!(
            received_data.executable(),
            item.executable,
        );

        // u64::MAX means we are not considering the epoch
        if item.rent_epoch != u64::MAX && received_data.rent_epoch() != u64::MAX {
            assert_eq!(
                received_data.rent_epoch(),
                item.rent_epoch,
            );
        }
    }

    assert_eq!(
        consumed_units,
        cu_avail.saturating_sub(output.cu_avail),
    );

    if return_data.is_empty() {
        assert!(output.return_data.is_empty());
    } else {
        assert_eq!(&output.return_data, return_data);
    }
}
