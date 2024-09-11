use crate::Instruction;
#[cfg(feature = "set-syscall-stubs")]
use std::sync::{Arc, RwLock};

#[cfg(feature = "set-syscall-stubs")]
struct DefaultInstructionSyscallStubs {}
#[cfg(feature = "set-syscall-stubs")]
impl InstructionSyscallStubs for DefaultInstructionSyscallStubs {}

#[cfg(feature = "set-syscall-stubs")]
lazy_static::lazy_static! {
    pub static ref INSTRUCTION_SYSCALL_STUBS: Arc<RwLock<Box<dyn InstructionSyscallStubs>>> = Arc::new(
        RwLock::new(Box::new(DefaultInstructionSyscallStubs {}))
    );
}

#[cfg(feature = "set-syscall-stubs")]
pub trait InstructionSyscallStubs: Sync + Send {
    fn sol_get_processed_sibling_instruction(&self, _index: usize) -> Option<Instruction> {
        None
    }
    fn sol_get_stack_height(&self) -> u64 {
        0
    }
}

#[cfg(feature = "set-syscall-stubs")]
pub(crate) fn sol_get_processed_sibling_instruction(index: usize) -> Option<Instruction> {
    INSTRUCTION_SYSCALL_STUBS
        .read()
        .unwrap()
        .sol_get_processed_sibling_instruction(index)
}
#[cfg(not(feature = "set-syscall-stubs"))]
pub(crate) fn sol_get_processed_sibling_instruction(_index: usize) -> Option<Instruction> {
    None
}

#[cfg(feature = "set-syscall-stubs")]
pub(crate) fn sol_get_stack_height() -> u64 {
    INSTRUCTION_SYSCALL_STUBS
        .read()
        .unwrap()
        .sol_get_stack_height()
}

#[cfg(not(feature = "set-syscall-stubs"))]
pub(crate) fn sol_get_stack_height() -> u64 {
    0
}

#[cfg(feature = "set-syscall-stubs")]
pub fn set_instruction_syscall_stubs(
    syscall_stubs: Box<dyn InstructionSyscallStubs>,
) -> Box<dyn InstructionSyscallStubs> {
    std::mem::replace(
        &mut INSTRUCTION_SYSCALL_STUBS.write().unwrap(),
        syscall_stubs,
    )
}
