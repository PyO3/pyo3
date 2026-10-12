use crate::ffi;

/// Fallback constants for slots which `#[pymethods]` provides as inherent constants.
pub trait PyClassSlotProbes {
    const __PYO3_HAS_TP_REPR: bool = false;
    const __PYO3_HAS_NB_INT: bool = false;
}

impl<T> PyClassSlotProbes for T {}

/// Slots which `#[pyclass]` can provide default implementations for, these will be automatically
/// eliminated to `None` with help of [`PyClassSlotProbes`] if the user has provided an implementation.
#[derive(Copy, Clone, Default)]
pub struct DefaultPyO3Slots {
    pub(crate) tp_repr: Option<ffi::reprfunc>,
    pub(crate) nb_int: Option<ffi::unaryfunc>,
}

impl DefaultPyO3Slots {
    pub const EMPTY: Self = Self {
        tp_repr: None,
        nb_int: None,
    };

    pub const fn new() -> Self {
        Self {
            tp_repr: None,
            nb_int: None,
        }
    }

    pub const fn with_tp_repr_if(mut self, enabled: bool, repr: ffi::reprfunc) -> Self {
        if enabled {
            self.tp_repr = Some(repr);
        }
        self
    }

    pub const fn with_nb_int_if(mut self, enabled: bool, nb_int: ffi::unaryfunc) -> Self {
        if enabled {
            self.nb_int = Some(nb_int);
        }
        self
    }
}
