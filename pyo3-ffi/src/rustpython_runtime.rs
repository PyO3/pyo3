use rustpython::InterpreterBuilderExt;
use rustpython_vm::{Interpreter, InterpreterBuilder, VirtualMachine};
use std::cell::Cell;
use std::sync::OnceLock;

static INTERPRETER: OnceLock<usize> = OnceLock::new();

thread_local! {
    static CURRENT_VM: Cell<Option<*const VirtualMachine>> = const { Cell::new(None) };
    static ATTACH_COUNT: Cell<u32> = const { Cell::new(0) };
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum AttachState {
    Assumed,
    Ensured,
}

pub(crate) fn initialize() {
    let _ = interpreter();
}

pub(crate) fn interpreter() -> &'static Interpreter {
    let ptr = INTERPRETER.get_or_init(|| {
        let interpreter = InterpreterBuilder::new().init_stdlib().interpreter();
        Box::into_raw(Box::new(interpreter)) as usize
    });
    unsafe { &*(*ptr as *const Interpreter) }
}

pub(crate) fn is_initialized() -> bool {
    INTERPRETER.get().is_some()
}

pub(crate) fn finalize() {
    // RustPython does not currently expose a CPython-style global finalize API.
    // Keep the process-global interpreter alive for the duration of the process.
}

pub(crate) fn ensure_attached() -> AttachState {
    let already_attached = ATTACH_COUNT.with(|count| {
        let current = count.get();
        count.set(current + 1);
        current > 0
    });

    if already_attached {
        AttachState::Assumed
    } else {
        let vm_ptr = interpreter().enter(|vm| vm as *const VirtualMachine);
        CURRENT_VM.with(|cell| cell.set(Some(vm_ptr)));
        if let Some(vm) = current_vm() {
            crate::pyerrors::init_exception_symbols(vm);
            crate::methodobject::init_builtin_function_descriptors(vm);
        }
        AttachState::Ensured
    }
}

pub(crate) fn release_attached() {
    ATTACH_COUNT.with(|count| {
        let current = count.get();
        if current <= 1 {
            count.set(0);
            CURRENT_VM.with(|cell| cell.set(None));
        } else {
            count.set(current - 1);
        }
    });
}

pub(crate) fn is_attached() -> bool {
    ATTACH_COUNT.with(|count| count.get() > 0)
}

pub(crate) fn current_vm() -> Option<&'static VirtualMachine> {
    CURRENT_VM.with(|cell| cell.get()).map(|ptr| unsafe { &*ptr })
}

pub(crate) fn with_vm<R>(f: impl FnOnce(&VirtualMachine) -> R) -> R {
    let vm = current_vm().expect("RustPython FFI used outside an attached interpreter context");
    f(vm)
}
