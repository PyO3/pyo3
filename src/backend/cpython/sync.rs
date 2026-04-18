#[cfg(all(Py_3_14, not(Py_LIMITED_API)))]
use crate::sync::critical_section::EnteredCriticalSection;
#[cfg(all(Py_3_14, not(Py_LIMITED_API)))]
use crate::types::PyMutex;
#[cfg(all(Py_3_14, not(Py_LIMITED_API)))]
use crate::Python;
use crate::{types::PyAny, Bound};

#[cfg(Py_GIL_DISABLED)]
struct CSGuard(crate::ffi::PyCriticalSection);

#[cfg(Py_GIL_DISABLED)]
impl Drop for CSGuard {
    fn drop(&mut self) {
        unsafe {
            crate::ffi::PyCriticalSection_End(&mut self.0);
        }
    }
}

#[cfg(Py_GIL_DISABLED)]
struct CS2Guard(crate::ffi::PyCriticalSection2);

#[cfg(Py_GIL_DISABLED)]
impl Drop for CS2Guard {
    fn drop(&mut self) {
        unsafe {
            crate::ffi::PyCriticalSection2_End(&mut self.0);
        }
    }
}

pub(crate) fn with_critical_section<F, R>(object: &Bound<'_, PyAny>, f: F) -> R
where
    F: FnOnce() -> R,
{
    #[cfg(Py_GIL_DISABLED)]
    {
        let mut guard = CSGuard(unsafe { std::mem::zeroed() });
        unsafe { crate::ffi::PyCriticalSection_Begin(&mut guard.0, object.as_ptr()) };
        f()
    }
    #[cfg(not(Py_GIL_DISABLED))]
    {
        let _ = object;
        f()
    }
}

pub(crate) fn with_critical_section2<F, R>(a: &Bound<'_, PyAny>, b: &Bound<'_, PyAny>, f: F) -> R
where
    F: FnOnce() -> R,
{
    #[cfg(Py_GIL_DISABLED)]
    {
        let mut guard = CS2Guard(unsafe { std::mem::zeroed() });
        unsafe { crate::ffi::PyCriticalSection2_Begin(&mut guard.0, a.as_ptr(), b.as_ptr()) };
        f()
    }
    #[cfg(not(Py_GIL_DISABLED))]
    {
        let _ = (a, b);
        f()
    }
}

#[cfg(all(Py_3_14, not(Py_LIMITED_API)))]
pub(crate) fn with_critical_section_mutex<F, R, T>(_py: Python<'_>, mutex: &PyMutex<T>, f: F) -> R
where
    F: for<'s> FnOnce(EnteredCriticalSection<'s, T>) -> R,
{
    #[cfg(Py_GIL_DISABLED)]
    {
        let mut guard = CSGuard(unsafe { std::mem::zeroed() });
        unsafe { crate::ffi::PyCriticalSection_BeginMutex(&mut guard.0, &mut *mutex.mutex.get()) };
        f(EnteredCriticalSection(&mutex.data))
    }
    #[cfg(not(Py_GIL_DISABLED))]
    {
        f(EnteredCriticalSection(&mutex.data))
    }
}

#[cfg(all(Py_3_14, not(Py_LIMITED_API)))]
pub(crate) fn with_critical_section_mutex2<F, R, T1, T2>(
    _py: Python<'_>,
    m1: &PyMutex<T1>,
    m2: &PyMutex<T2>,
    f: F,
) -> R
where
    F: for<'s> FnOnce(EnteredCriticalSection<'s, T1>, EnteredCriticalSection<'s, T2>) -> R,
{
    #[cfg(Py_GIL_DISABLED)]
    {
        let mut guard = CS2Guard(unsafe { std::mem::zeroed() });
        unsafe {
            crate::ffi::PyCriticalSection2_BeginMutex(
                &mut guard.0,
                &mut *m1.mutex.get(),
                &mut *m2.mutex.get(),
            )
        };
        f(
            EnteredCriticalSection(&m1.data),
            EnteredCriticalSection(&m2.data),
        )
    }
    #[cfg(not(Py_GIL_DISABLED))]
    {
        f(
            EnteredCriticalSection(&m1.data),
            EnteredCriticalSection(&m2.data),
        )
    }
}

pub(crate) fn once_lock_get_or_init<'a, F, T>(
    cell: &'a once_cell::sync::OnceCell<T>,
    _py: crate::Python<'_>,
    f: F,
) -> &'a T
where
    F: FnOnce() -> T,
{
    use crate::internal::state::SuspendAttach;

    let ts_guard = unsafe { SuspendAttach::new() };
    cell.get_or_init(move || {
        drop(ts_guard);
        f()
    })
}

pub(crate) fn once_lock_get_or_try_init<'a, F, T, E>(
    cell: &'a once_cell::sync::OnceCell<T>,
    _py: crate::Python<'_>,
    f: F,
) -> Result<&'a T, E>
where
    F: FnOnce() -> Result<T, E>,
{
    use crate::internal::state::SuspendAttach;

    let ts_guard = unsafe { SuspendAttach::new() };
    cell.get_or_try_init(move || {
        drop(ts_guard);
        f()
    })
}
