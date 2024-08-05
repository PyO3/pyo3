use std::ffi::{c_int, c_void};
use std::ptr::{self, NonNull};

use crate::ffi;

thread_local! {
    /// De-facto statically allocated function pointers to be used as fields
    /// of [`ffi::PyModuleDef_Slot`]s.
    ///
    /// The [`SlotsClosureAllocs`] wrapper ensures that no [`c_void`]
    /// function pointer created at runtime is ever deallocated. Thus each pointer
    /// continues to exist throughout the remaining lifetime of the program and
    /// is therefore "de-facto static". These pointers may then be given to the
    /// [`ffi::PyModuleDef_Slot`]s used during multi-phase module initialization.
    static CLOSURE_ALLOCS: std::sync::Mutex<SlotsClosureAllocs> =
        std::sync::Mutex::new(SlotsClosureAllocs::new());

    /// De-facto statically allocated [`ffi::PyModuleDef_Slot`]s.
    ///
    /// The [`SlotAllocs`] wrapper ensures that no [`ffi::PyModuleDef_Slot`]s
    /// created at runtime are ever deallocated. Thus they continue to exist
    /// throughout the remaining program, making them "de-facto static". This allows
    /// these slots to be used for [`ffi::PyModuleDef`] (specifically as `m_slots`
    /// field).
    static SLOT_ALLOCS: std::sync::Mutex<SlotAllocs> =
        std::sync::Mutex::new(SlotAllocs::new());
}

/// `Send` and `Sync` wrapper `struct` for a closure that has been boxed by
/// and converted into a pointer ([`Box::into_raw`]) by [`alloc_closure`].
///
/// When dropped, boxes and calls the accompanying `dealloc_ptr`, which ought
/// to be *another* closure that converts the wrapped closure `closure_ptr` back
/// to its boxed type ([`Box::from_raw`]) and drops it.
struct SlotsClosure {
    #[allow(unused)]
    closure_ptr: *mut c_void,
    dealloc_ptr: *mut dyn FnOnce(),
}

unsafe impl Send for SlotsClosure {}
unsafe impl Sync for SlotsClosure {}

impl SlotsClosure {
    /// Creates a new [`SlotsClosure`].
    ///
    /// SAFETY: `closure_ptr` and `dealloc_ptr` must have been acquired via
    /// [`alloc_closure`] before.
    const unsafe fn new(closure_ptr: *mut c_void, dealloc_ptr: *mut dyn FnOnce()) -> Self {
        Self {
            closure_ptr,
            dealloc_ptr,
        }
    }
}

impl Drop for SlotsClosure {
    fn drop(&mut self) {
        // SAFETY: We obtained this pointer via Box::into_raw earlier
        let dealloc_boxed: Box<dyn FnOnce()> = unsafe { Box::from_raw(self.dealloc_ptr) };
        dealloc_boxed(); // closure_ptr is now invalid!
    }
}

/// Stores function pointers used during multi-phase module initialization.
/// See [`CLOSURE_ALLOCS`] and [`SlotsClosureAllocs::alloc_closure`] for more
/// information.
struct SlotsClosureAllocs {
    allocs: Vec<SlotsClosure>,
}

impl SlotsClosureAllocs {
    const fn new() -> Self {
        Self { allocs: Vec::new() }
    }

    /// Underlying implementation of [`alloc_closure`].
    ///
    /// This not only boxes and cases the given `closure` to a [`*mut c_void`],
    /// but also creates a second closure that acts as the drop handler of the
    /// passed one.
    ///
    /// The pointers of both closures are wrapped by [`SlotsClosure`] which
    /// implements a corresponding [`Drop`] handler. Each [`SlotsClosure`] is
    /// stored in [`Self::allocs`].
    ///
    /// This means that each returned [`*mut c_void`] function pointer lives as
    /// long as the [`SlotsClosureAllocs`] it is stored in.
    ///
    /// [`*mut c_void`]: c_void
    unsafe fn alloc_closure<F>(&mut self, closure: F) -> *mut c_void
    where
        F: FnMut(*mut ffi::PyObject) -> c_int,
    {
        let closure_ptr = Box::into_raw(Box::new(closure));

        let casted_ptr = closure_ptr as *const _ as *mut u8;

        let dealloc_closure: Box<dyn FnOnce()> = Box::new(move || {
            let closure_ptr: *mut F = casted_ptr.cast();
            let boxed = unsafe { Box::from_raw(closure_ptr) };
            drop(boxed);
        });

        let closure_ptr = closure_ptr as *mut c_void;

        let dealloc_ptr = Box::into_raw(dealloc_closure);

        self.allocs
            .push(unsafe { SlotsClosure::new(closure_ptr, dealloc_ptr) });

        return closure_ptr;
    }
}

/// Takes a closure intended to be used as a function pointer for a
/// [`ffi::PyModuleDef_Slot`], boxes it, and returns a [`*mut c_void`] function
/// pointer. This pointer ought to then given to a [`ffi::PyModuleDef_Slot`]
/// directly or to its wrapper [`ModuleDefSlot`].
///
/// The boxed closure remains allocated for the remaining lifetime of the
/// program.
///
/// SAFETY: The returned function pointer is solely to be used as [`Py_mod_exec`]
/// slot of the `m_slots` field of [`ffi::PyModuleDef`].
///
/// [`*mut c_void`]: c_void
/// [`Py_mod_exec`]: https://docs.python.org/3/c-api/module.html#c.Py_mod_exec
pub(crate) unsafe fn alloc_closure<F>(closure: F) -> *mut c_void
where
    F: FnMut(*mut ffi::PyObject) -> c_int,
{
    CLOSURE_ALLOCS.with(|allocs| {
        let mut lock = allocs.lock().unwrap();
        unsafe { lock.alloc_closure(closure) }
    })
}

/// [`Send`] and [`Sync`] wrapper of `ffi::PyModuleDef_Slot`.
#[repr(transparent)]
pub struct ModuleDefSlot(ffi::PyModuleDef_Slot);

unsafe impl Send for ModuleDefSlot {}
unsafe impl Sync for ModuleDefSlot {}

impl ModuleDefSlot {
    /// Creates a new [`ModuleDefSlot`] from the given `slot` and `value`.
    pub const fn new(slot: i32, value: *mut c_void) -> Self {
        Self(ffi::PyModuleDef_Slot { slot, value })
    }

    /// Creates a new [`ModuleDefSlot`] with `id` [`ffi::Py_mod_exec`] and
    /// [`module_state_init`] as function pointer.
    ///
    /// This slot should be used as the first element of the array of slots
    /// passed to [`ffi::PyModuleDef`] during multi-phase initialization,
    /// as it ensures that per-module state is initialized.
    pub const fn start() -> Self {
        Self::new(ffi::Py_mod_exec, module_state_init as *mut c_void)
    }

    /// Creates a new [`ModuleDefSlot`] with `id` `0`, which marks the end of
    /// the array of slots that is passed to [`ffi::PyModuleDef`] during
    /// multi-phase initialization.
    pub const fn end() -> Self {
        Self::new(0, ptr::null_mut())
    }

    /// Checks if `self` is [`end`].
    ///
    /// [`end`]: Self::end
    pub const fn is_end(&self) -> bool {
        self.0.slot == 0
    }
}

#[cfg(Py_3_12)]
impl ModuleDefSlot {
    /// Creates a new [`ModuleDefSlot`] that specifies that the module
    /// [does not support being imported in subinterpreters].
    ///
    /// [does not support being imported in subinterpreters]: https://docs.python.org/3/c-api/module.html#c.Py_MOD_MULTIPLE_INTERPRETERS_NOT_SUPPORTED
    pub const fn no_multiple_interpreters() -> Self {
        Self::new(
            ffi::Py_mod_multiple_interpreters,
            ffi::Py_MOD_MULTIPLE_INTERPRETERS_NOT_SUPPORTED,
        )
    }

    /// Creates a new [`ModuleDefSlot`] that specifies that the module
    /// [supports being imported in subinterpreters].
    ///
    /// [supports being imported in subinterpreters]: https://docs.python.org/3/c-api/module.html#c.Py_MOD_MULTIPLE_INTERPRETERS_SUPPORTED
    pub const fn multiple_interpreters() -> Self {
        Self::new(
            ffi::Py_mod_multiple_interpreters,
            ffi::Py_MOD_MULTIPLE_INTERPRETERS_SUPPORTED,
        )
    }

    /// Creates a new [`ModuleDefSlot`] that specifies that the module
    /// [supports a separate GIL per subinterpreter].
    ///
    /// [supports a separate GIL per subinterpreter]: https://docs.python.org/3/c-api/module.html#c.Py_MOD_PER_INTERPRETER_GIL_SUPPORTED
    pub const fn per_interpreter_gil() -> Self {
        Self::new(
            ffi::Py_mod_multiple_interpreters,
            ffi::Py_MOD_PER_INTERPRETER_GIL_SUPPORTED,
        )
    }
}

/// [`Sync`] wrapper for a (de-facto) `static` slice of [`ffi::PyModuleDef_Slot`]s.
///
/// This struct can be acquired by either converting it from an existing
/// [`&'static \[ModuleDefSlot\]`] or via [`alloc_slots`].
///
/// The inner [`*mut ffi::PyModuleDef_Slot`] points to the start of a C array
/// of [`ffi::PyModuleDef_Slot`] that is terminated by a slot with `id` `0`
/// (see [`ModuleDefSlot::end`]).
///
/// [`*mut ffi::PyModuleDef_Slot`]: ffi::PyModuleDef_Slot
/// [`&'static \[ModuleDefSlot\]`]: ModuleDefSlot
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct ModuleDefSlots(*mut ffi::PyModuleDef_Slot);

unsafe impl Sync for ModuleDefSlots {}

impl ModuleDefSlots {
    /// SAFETY: Requires that `ptr` points to an array of [`ffi::PyModuleDef_Slot`]
    /// that is terminated by a slot with `id` `0`.
    const unsafe fn new(ptr: *mut ffi::PyModuleDef_Slot) -> Self {
        Self(ptr)
    }

    /// Creates a new [`ModuleDefSlots`] struct from the given `slots`.
    ///
    /// NOTE: Will `panic!` if `slots` is empty or [not terminated correctly].
    ///
    /// [not terminated correctly]: ModuleDefSlot::end
    pub fn new_from_static(slots: &'static [ModuleDefSlot]) -> Self {
        match slots.last() {
            Some(last) if !last.is_end() => panic!("slot array is not terminated correctly"),
            Some(_) => {}
            None => panic!("slot array is empty"),
        };

        Self(slots as *const _ as *mut ffi::PyModuleDef_Slot)
    }

    /// Returns the inner [`*mut ffi::PyModuleDef_Slot`].
    ///
    /// [`*mut ffi::PyModuleDef_Slot`]: ffi::PyModuleDef_Slot
    pub(crate) const fn into_inner(self) -> *mut ffi::PyModuleDef_Slot {
        self.0
    }
}

/// Stores vectors of slots used during multi-phase module initialization.
/// See [`SLOT_ALLOCS`] and [`SlotAllocs::alloc_slots`] for more information.
struct SlotAllocs {
    slots_list: Vec<Vec<ModuleDefSlot>>,
}

impl SlotAllocs {
    const fn new() -> Self {
        Self {
            slots_list: Vec::new(),
        }
    }

    fn alloc_slots(&mut self, slots: impl IntoIterator<Item = ModuleDefSlot>) -> ModuleDefSlots {
        let slots: Vec<ModuleDefSlot> = slots.into_iter().collect();

        self.slots_list.push(slots);
        let slots = self
            .slots_list
            .last()
            .expect("slots_list.last() should never be None")
            .as_slice();

        unsafe { ModuleDefSlots::new(slots as *const _ as *mut ffi::PyModuleDef_Slot) }
    }
}

/// Takes the given `slots` to be used for multi-phase module initialization,
/// allocates them on the heap and returns a corresponding [`ModuleDefSlots`].
///
/// The `slots` remain allocated for the remaining lifetime of the program.
pub fn alloc_slots(slots: impl IntoIterator<Item = ModuleDefSlot>) -> ModuleDefSlots {
    SLOT_ALLOCS.with(|allocs| {
        let mut lock = allocs.lock().unwrap();
        lock.alloc_slots(slots)
    })
}

/// Represents a Python module's state.
///
/// More precisely, this `struct` resides on the per-module memory area
/// allocated during the module's creation.
#[repr(C)]
#[derive(Debug)]
pub struct ModuleState {
    inner: Option<NonNull<ModuleStateImpl>>,
}

impl ModuleState {
    pub fn new() -> Self {
        let boxed = Box::new(ModuleStateImpl::new());

        Self {
            inner: NonNull::new(Box::into_raw(boxed)),
        }
    }
}

impl Default for ModuleState {
    fn default() -> Self {
        Self::new()
    }
}

/// Inner layout of [`ModuleState`].
///
/// In order to guarantee that all resources acquired during the initialization
/// of per-module state are correctly released, this `struct` exists as the sole
/// field of [`ModuleState`] in the form of a pointer. This allows
/// [`module_state_free`] to safely [`drop`] this `struct` when [`ModuleState`]
/// is being deallocated by the Python interpreter.
#[repr(C)]
#[derive(Debug)]
struct ModuleStateImpl {}

impl ModuleStateImpl {
    fn new() -> Self {
        Self {}
    }
}

/// Called during multi-phase initialization in order to create an instance of
/// [`ModuleState`] on the memory area specific to modules.
///
/// Slot: [`Py_mod_exec`]
///
/// [`Py_mod_exec`]: https://docs.python.org/3/c-api/module.html#c.Py_mod_exec
pub unsafe extern "C" fn module_state_init(module: *mut ffi::PyObject) -> c_int {
    let state: *mut ModuleState = ffi::PyModule_GetState(module.cast()).cast();

    if state.is_null() {
        *state = ModuleState::new();
        return 0;
    }

    0
}

/// Called during GC traversal of the module object.
///
/// Used for the [`m_traverse`] field of [`PyModuleDef`].
///
/// [`m_traverse`]: https://docs.python.org/3/c-api/module.html#c.PyModuleDef.m_traverse
/// [`PyModuleDef`]: https://docs.python.org/3/c-api/module.html#c.PyModuleDef
pub unsafe extern "C" fn module_state_traverse(
    _module: *mut ffi::PyObject,
    _visit: ffi::visitproc,
    _arg: *mut c_void,
) -> c_int {
    0
}

/// Called during GC clearing of the module object.
///
/// Used for the [`m_clear`] field of [`PyModuleDef`].
///
/// [`m_clear`]: https://docs.python.org/3/c-api/module.html#c.PyModuleDef.m_clear
/// [`PyModuleDef`]: https://docs.python.org/3/c-api/module.html#c.PyModuleDef
pub unsafe extern "C" fn module_state_clear(_module: *mut ffi::PyObject) -> c_int {
    // Should any PyObjects be made part of ModuleState or ModuleStateInner,
    // these have to be Py_CLEARed here.
    // See: examples/sequential/src/module.rs
    0
}

/// Called during deallocation of the module object.
///
/// Used for the [`m_free`] field of [`PyModuleDef`].
///
/// [`m_free`]: https://docs.python.org/3/c-api/module.html#c.PyModuleDef.m_free
/// [`PyModuleDef`]: https://docs.python.org/3/c-api/module.html#c.PyModuleDef
pub unsafe extern "C" fn module_state_free(module: *mut c_void) {
    let state: *mut ModuleState = ffi::PyModule_GetState(module.cast()).cast();
    if let Some(inner) = (*state).inner {
        let ptr = inner.as_ptr();
        // SAFETY: We obtained this pointer via Box::into_raw beforehand.
        drop(unsafe { Box::from_raw(ptr) });
    }
}
