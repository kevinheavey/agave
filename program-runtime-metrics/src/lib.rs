use {
    solana_measure::measure::Measure,
    solana_program_runtime::{
        invoke_context::InvokeContext,
        loaded_programs::{
            ProgramCacheEntry, ProgramCacheEntryOwner, ProgramCacheEntryType,
            ProgramRuntimeEnvironment,
        },
    },
    solana_pubkey::Pubkey,
    solana_rbpf::{elf::Executable, program::BuiltinProgram, verifier::RequisiteVerifier},
    solana_sdk::saturating_add_assign,
    solana_timings::ExecuteDetailsTimings,
    solana_type_overrides::sync::{atomic::AtomicU64, Arc},
};

/// Time measurements for loading a single [ProgramCacheEntry].
#[derive(Debug, Default)]
pub struct LoadProgramMetrics {
    /// Program address, but as text
    pub program_id: String,
    /// Microseconds it took to `create_program_runtime_environment`
    pub register_syscalls_us: u64,
    /// Microseconds it took to `Executable::<InvokeContext>::load`
    pub load_elf_us: u64,
    /// Microseconds it took to `executable.verify::<RequisiteVerifier>`
    pub verify_code_us: u64,
    /// Microseconds it took to `executable.jit_compile`
    pub jit_compile_us: u64,
}

impl LoadProgramMetrics {
    pub fn submit_datapoint(&self, timings: &mut ExecuteDetailsTimings) {
        saturating_add_assign!(
            timings.create_executor_register_syscalls_us,
            self.register_syscalls_us
        );
        saturating_add_assign!(timings.create_executor_load_elf_us, self.load_elf_us);
        saturating_add_assign!(timings.create_executor_verify_code_us, self.verify_code_us);
        saturating_add_assign!(timings.create_executor_jit_compile_us, self.jit_compile_us);
        solana_metrics::datapoint_trace!(
            "create_executor_trace",
            ("program_id", self.program_id, String),
            ("register_syscalls_us", self.register_syscalls_us, i64),
            ("load_elf_us", self.load_elf_us, i64),
            ("verify_code_us", self.verify_code_us, i64),
            ("jit_compile_us", self.jit_compile_us, i64),
        );
    }
}

/// Creates a new user program
pub fn new_program_cache_entry(
    loader_key: &Pubkey,
    program_runtime_environment: ProgramRuntimeEnvironment,
    deployment_slot: u64,
    effective_slot: u64,
    elf_bytes: &[u8],
    account_size: usize,
    metrics: &mut LoadProgramMetrics,
) -> Result<ProgramCacheEntry, Box<dyn std::error::Error>> {
    new_internal(
        loader_key,
        program_runtime_environment,
        deployment_slot,
        effective_slot,
        elf_bytes,
        account_size,
        metrics,
        false, /* reloading */
    )
}

/// Reloads a user program, *without* running the verifier.
///
/// # Safety
///
/// This method is unsafe since it assumes that the program has already been verified. Should
/// only be called when the program was previously verified and loaded in the cache, but was
/// unloaded due to inactivity. It should also be checked that the `program_runtime_environment`
/// hasn't changed since it was unloaded.
pub unsafe fn reload_program_cache_entry(
    loader_key: &Pubkey,
    program_runtime_environment: Arc<BuiltinProgram<InvokeContext<'static>>>,
    deployment_slot: u64,
    effective_slot: u64,
    elf_bytes: &[u8],
    account_size: usize,
    metrics: &mut LoadProgramMetrics,
) -> Result<ProgramCacheEntry, Box<dyn std::error::Error>> {
    new_internal(
        loader_key,
        program_runtime_environment,
        deployment_slot,
        effective_slot,
        elf_bytes,
        account_size,
        metrics,
        true, /* reloading */
    )
}

fn new_internal(
    loader_key: &Pubkey,
    program_runtime_environment: Arc<BuiltinProgram<InvokeContext<'static>>>,
    deployment_slot: u64,
    effective_slot: u64,
    elf_bytes: &[u8],
    account_size: usize,
    metrics: &mut LoadProgramMetrics,
    reloading: bool,
) -> Result<ProgramCacheEntry, Box<dyn std::error::Error>> {
    let load_elf_time = Measure::start("load_elf_time");
    // The following unused_mut exception is needed for architectures that do not
    // support JIT compilation.
    #[allow(unused_mut)]
    let mut executable = Executable::load(elf_bytes, program_runtime_environment.clone())?;
    metrics.load_elf_us = load_elf_time.end_as_us();

    if !reloading {
        let verify_code_time = Measure::start("verify_code_time");
        executable.verify::<RequisiteVerifier>()?;
        metrics.verify_code_us = verify_code_time.end_as_us();
    }

    #[cfg(all(not(target_os = "windows"), target_arch = "x86_64"))]
    {
        let jit_compile_time = Measure::start("jit_compile_time");
        executable.jit_compile()?;
        metrics.jit_compile_us = jit_compile_time.end_as_us();
    }

    Ok(ProgramCacheEntry {
        deployment_slot,
        account_owner: ProgramCacheEntryOwner::try_from(loader_key).unwrap(),
        account_size,
        effective_slot,
        tx_usage_counter: AtomicU64::new(0),
        program: ProgramCacheEntryType::Loaded(executable),
        ix_usage_counter: AtomicU64::new(0),
        latest_access_slot: AtomicU64::new(0),
    })
}
