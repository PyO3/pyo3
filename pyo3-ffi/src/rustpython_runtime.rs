use rustpython::InterpreterBuilderExt;
use rustpython_vm::{
    AsObject, InterpreterBuilder, Settings, VirtualMachine,
    builtins::PyUtf8StrRef,
    TryFromObject,
};
use std::any::Any;
use std::cell::{Cell, UnsafeCell};
use std::mem::MaybeUninit;
use std::panic::{self, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, OnceLock};

thread_local! {
    static ATTACH_COUNT: Cell<u32> = const { Cell::new(0) };
    static CURRENT_VM: Cell<*const VirtualMachine> = const { Cell::new(std::ptr::null()) };
    static ON_RUNTIME_THREAD: Cell<bool> = const { Cell::new(false) };
}

struct RuntimeHandle {
    tx: mpsc::Sender<RuntimeRequest>,
    thread_id: std::thread::ThreadId,
}

enum RuntimeRequest {
    Call {
        thunk: unsafe fn(usize, &VirtualMachine),
        payload: usize,
        done_tx: mpsc::SyncSender<()>,
    },
}

static RUNTIME: OnceLock<RuntimeHandle> = OnceLock::new();
static INITIALIZED: AtomicBool = AtomicBool::new(false);

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum AttachState {
    Assumed,
    Ensured,
}

fn current_vm() -> Option<&'static VirtualMachine> {
    let ptr = CURRENT_VM.with(|current| current.get());
    (!ptr.is_null()).then(|| unsafe { &*ptr })
}

fn runtime() -> &'static RuntimeHandle {
    RUNTIME.get_or_init(|| {
        let (tx, rx) = mpsc::channel::<RuntimeRequest>();
        let (ready_tx, ready_rx) = mpsc::sync_channel(1);

        std::thread::spawn(move || {
            let thread_id = std::thread::current().id();
            let interpreter = InterpreterBuilder::new()
                .settings(runtime_settings())
                .init_stdlib()
                .interpreter();

            interpreter.enter(|vm| {
                struct RuntimeThreadGuard {
                    previous_vm: *const VirtualMachine,
                }

                impl Drop for RuntimeThreadGuard {
                    fn drop(&mut self) {
                        CURRENT_VM.with(|current| current.set(self.previous_vm));
                        ON_RUNTIME_THREAD.with(|flag| flag.set(false));
                    }
                }

                ON_RUNTIME_THREAD.with(|flag| flag.set(true));
                let previous_vm = CURRENT_VM.with(|current| {
                    let previous = current.get();
                    current.set(vm as *const VirtualMachine);
                    previous
                });
                let _guard = RuntimeThreadGuard { previous_vm };

                crate::pyerrors::init_exception_symbols(vm);
                crate::methodobject::init_builtin_function_descriptors(vm);
                let _ = vm.new_scope_with_main();
                let _ = vm.import("warnings", 0);
                let _ = vm.import("site", 0);
                let _ = import_optional_module(vm, "sitecustomize");
                let _ = import_optional_module(vm, "usercustomize");
                crate::import::install_registered_inittab_modules(vm);
                ready_tx
                    .send(thread_id)
                    .expect("RustPython runtime initialization channel closed");

                while let Ok(request) = rx.recv() {
                    match request {
                        RuntimeRequest::Call {
                            thunk,
                            payload,
                            done_tx,
                        } => {
                            unsafe { thunk(payload, vm) };
                            let _ = done_tx.send(());
                        }
                    }
                }
            });
        });

        let thread_id = ready_rx
            .recv()
            .expect("RustPython runtime thread terminated before initialization");
        RuntimeHandle { tx, thread_id }
    })
}

fn runtime_settings() -> Settings {
    let mut settings = Settings::default();

    for key in ["RUSTPYTHONPATH", "PYTHONPATH"] {
        if let Some(paths) = std::env::var_os(key) {
            settings.path_list.extend(
                std::env::split_paths(&paths).map(|path| path.to_string_lossy().into_owned()),
            );
        }
    }

    settings
}

fn import_optional_module(vm: &VirtualMachine, name: &'static str) -> rustpython_vm::PyResult<()> {
    match vm.import(name, 0) {
        Ok(_) => Ok(()),
        Err(err)
            if err.fast_isinstance(vm.ctx.exceptions.import_error)
                || err.fast_isinstance(vm.ctx.exceptions.module_not_found_error) =>
        {
            let missing_name = err
                .as_object()
                .get_attr("name", vm)
                .ok()
                .and_then(|value| PyUtf8StrRef::try_from_object(vm, value).ok())
                .map(|value: PyUtf8StrRef| value.as_str().to_owned());

            if missing_name.as_deref() == Some(name) {
                Ok(())
            } else {
                Err(err)
            }
        }
        Err(err) => Err(err),
    }
}

struct DispatchState<F, R> {
    closure: UnsafeCell<Option<F>>,
    result: UnsafeCell<MaybeUninit<R>>,
    panic: UnsafeCell<Option<Box<dyn Any + Send + 'static>>>,
}

unsafe fn dispatch_call<F, R>(payload: usize, vm: &VirtualMachine)
where
    F: FnOnce(&VirtualMachine) -> R,
{
    let state = unsafe { &*(payload as *const DispatchState<F, R>) };
    let closure = unsafe {
        (&mut *state.closure.get())
            .take()
            .expect("RustPython runtime dispatch closure missing")
    };
    let outcome = panic::catch_unwind(AssertUnwindSafe(|| closure(vm)));

    unsafe {
        match outcome {
            Ok(result) => {
                (*state.result.get()).write(result);
            }
            Err(err) => {
                *state.panic.get() = Some(err);
            }
        }
    }
}

fn dispatch<F, R>(f: F) -> R
where
    F: FnOnce(&VirtualMachine) -> R,
{
    if ON_RUNTIME_THREAD.with(|flag| flag.get()) {
        return f(current_vm().expect("RustPython runtime thread missing current VM"));
    }

    let runtime = runtime();
    debug_assert_ne!(runtime.thread_id, std::thread::current().id());

    let state = DispatchState {
        closure: UnsafeCell::new(Some(f)),
        result: UnsafeCell::new(MaybeUninit::uninit()),
        panic: UnsafeCell::new(None),
    };
    let payload = (&state as *const DispatchState<_, _>) as usize;
    let (done_tx, done_rx) = mpsc::sync_channel(1);

    runtime
        .tx
        .send(RuntimeRequest::Call {
            thunk: dispatch_call::<F, R>,
            payload,
            done_tx,
        })
        .expect("RustPython runtime thread terminated during dispatch");
    done_rx
        .recv()
        .expect("RustPython runtime thread terminated before dispatch completed");

    if let Some(err) = unsafe { (&mut *state.panic.get()).take() } {
        panic::resume_unwind(err);
    }

    unsafe { (*state.result.get()).assume_init_read() }
}

pub(crate) fn initialize() {
    let _ = runtime();
    INITIALIZED.store(true, Ordering::SeqCst);
}

pub(crate) fn runtime_thread_id() -> Option<std::thread::ThreadId> {
    RUNTIME.get().map(|runtime| runtime.thread_id)
}

pub(crate) fn is_initialized() -> bool {
    INITIALIZED.load(Ordering::SeqCst)
}

pub(crate) fn finalize() {
    // RustPython does not currently expose a CPython-style global finalize API.
    // Keep the process-global runtime thread alive for the duration of the process,
    // but update the public lifecycle bit so PyO3 can model embedded init/finalize.
    INITIALIZED.store(false, Ordering::SeqCst);
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
        initialize();
        AttachState::Ensured
    }
}

pub(crate) fn release_attached() {
    ATTACH_COUNT.with(|count| {
        let current = count.get();
        if current <= 1 {
            count.set(0);
        } else {
            count.set(current - 1);
        }
    });
}

pub(crate) fn is_attached() -> bool {
    ATTACH_COUNT.with(|count| count.get() > 0)
}

pub(crate) fn with_vm<R>(f: impl FnOnce(&VirtualMachine) -> R) -> R {
    if ON_RUNTIME_THREAD.with(|flag| flag.get()) {
        return f(current_vm().expect("RustPython runtime thread missing current VM"));
    }

    assert!(
        is_attached(),
        "RustPython FFI used outside an attached interpreter context"
    );

    dispatch(f)
}
