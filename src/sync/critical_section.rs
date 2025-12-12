//! Wrappers for the Python critical section API
//!
//! [Critical Sections](https://docs.python.org/3/c-api/init.html#python-critical-section-api) allow
//! access to the [`PyMutex`](https://docs.python.org/3/c-api/init.html#c.PyMutex) lock attached to
//! each Python object in the free-threaded build. They are no-ops on the GIL-enabled build.
//!
//! Provides weaker locking guarantees than traditional locks, but can in some cases be used to
//! provide guarantees similar to the GIL without the risk of deadlocks associated with traditional
//! locks.
//!
//! # Usage Notes
//!
//! The calling thread locks the per-object mutex when it enters the critical section and holds it
//! until exiting the critical section unless the critical section is suspended. Any call into the
//! CPython C API may cause the critical section to be suspended. Creating an inner critical
//! section, for example by accessing an item in a Python list or dict, will cause the outer
//! critical section to be relased while the inner critical section is active.
//!
//! As a consequence, it is only possible to lock one or two objects at a time. If you need two lock
//! two objects, you should use the variants that accept two arguments. The outer critical section
//! is suspended if you create an outer an inner critical section on two objects using the
//! single-argument variants.
//!
//! It is not currently possible to lock more than two objects simultaneously using this mechanism.
//! Taking a critical section on a container object does not lock the objects stored in the
//! container.
//!
//! Many CPython C API functions do not lock the per-object mutex on objects passed to Python. You
//! should not expect critical sections applied to built-in types to prevent concurrent
//! modification. This API is most useful for user-defined types with full control over how the
//! internal state for the type is managed.
//!
//! The caller must ensure the closure cannot implicitly release the critical section. If a
//! multithreaded program calls back into the Python interpreter in a manner that would cause the
//! critical section to be released, the per-object mutex will be unlocked and the state of the
//! object may be read from or modified by another thread. Concurrent modifications are impossible,
//! but races are possible and the state of an object may change "underneath" a suspended thread in
//! possibly surprising ways.

#[cfg(all(Py_3_14, not(Py_LIMITED_API)))]
use crate::types::PyMutex;

#[cfg(all(Py_3_14, not(Py_LIMITED_API)))]
use crate::Python;
use crate::{types::PyAny, Bound};
#[cfg(all(Py_3_14, not(Py_LIMITED_API)))]
use std::cell::UnsafeCell;

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

/// Allows access to data protected by a PyMutex in a critical section
///
/// Used with the `with_critical_section_mutex` and
/// `with_critical_section_mutex2` functions. See the documentation of those
/// functions for more details.
#[cfg(all(Py_3_14, not(Py_LIMITED_API)))]
pub struct EnteredCriticalSection<'a, T>(&'a UnsafeCell<T>);

#[cfg(all(Py_3_14, not(Py_LIMITED_API)))]
impl<T> EnteredCriticalSection<'_, T> {
    /// Get a mutable reference to the data wrapped by a PyMutex
    ///
    /// # Safety
    ///
    /// The caller must ensure the closure cannot implicitly release the critical section.
    ///
    /// If a multithreaded program calls back into the Python interpreter in a manner that would cause
    /// the critical section to be released, the `PyMutex` will be unlocked and the resource protected
    /// by the `PyMutex` may be read from or modified by another thread while the critical section is
    /// suspended. Concurrent modifications are impossible, but races are possible and the state of the
    /// protected resource may change in possibly surprising ways after calls into the interpreter.
    pub unsafe fn get_mut(&mut self) -> &mut T {
        unsafe { &mut *(self.0.get()) }
    }

    /// Get a immutable reference to the value wrapped by a PyMutex
    ///
    /// # Safety
    ///
    /// The caller must ensure the critical section is not released while the
    /// reference is alive. If a multithreaded program calls back into the
    /// Python interpreter in a manner that would cause the critical section to
    /// be released, the `PyMutex` will be unlocked and the resource protected
    /// by the `PyMutex` may be read from or modified by another thread while
    /// the critical section is suspended and the thread that owns the reference
    /// is blocked. Concurrent modifications are impossible, but races are
    /// possible and the state of an object may change "underneath" a suspended
    /// thread in possibly surprising ways. Note that many operations on Python
    /// objects may call back into the interpreter in a blocking manner because
    /// many C API calls can trigger the execution of arbitrary Python code.
    pub unsafe fn get(&self) -> &T {
        unsafe { &*(self.0.get()) }
    }
}

/// Executes a closure with a Python critical section held on an object.
///
/// Locks the per-object mutex for the object `op` that is held while the closure `f` is
/// executing. The critical section may be temporarily released and re-acquired if the closure calls
/// back into the interpreter. See the notes in the
/// [`pyo3::sync::critical_section`][crate::sync::critical_section] module documentation for more
/// details.
///
/// This is structurally equivalent to the use of the paired Py_BEGIN_CRITICAL_SECTION and
/// Py_END_CRITICAL_SECTION C-API macros.
#[cfg_attr(not(Py_GIL_DISABLED), allow(unused_variables))]
pub fn with_critical_section<F, R>(object: &Bound<'_, PyAny>, f: F) -> R
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
        f()
    }
}

/// Executes a closure with a Python critical section held on two objects.
///
/// Locks the per-object mutex for the objects `a` and `b` that are held while the closure `f` is
/// executing. The critical section may be temporarily released and re-acquired if the closure calls
/// back into the interpreter. See the notes in the
/// [`pyo3::sync::critical_section`][crate::sync::critical_section] module documentation for more
/// details.
///
/// This is structurally equivalent to the use of the paired
/// Py_BEGIN_CRITICAL_SECTION2 and Py_END_CRITICAL_SECTION2 C-API macros.
#[cfg_attr(not(Py_GIL_DISABLED), allow(unused_variables))]
pub fn with_critical_section2<F, R>(a: &Bound<'_, PyAny>, b: &Bound<'_, PyAny>, f: F) -> R
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
        f()
    }
}

/// Executes a closure with a Python critical section held on a `PyMutex`.
///
/// Locks the mutex `mutex` until the closure `f` finishes. The mutex may be temporarily unlocked
/// and re-acquired if the closure calls back into the interpreter. See the notes in the
/// [`pyo3::sync::critical_section`][crate::sync::critical_section] module documentation for more
/// details.
///
/// This variant is particularly useful when paired with a global `PyMutex` to create a "local GIL"
/// to protect global state in an extension in an analogous manner to the GIL without introducing
/// any deadlock risks or affecting runtime behavior on the GIL-enabled build.
///
/// This is structurally equivalent to the use of the paired Py_BEGIN_CRITICAL_SECTION_MUTEX and
/// Py_END_CRITICAL_SECTION C-API macros.
///
/// # Safety
///
/// The caller must ensure the closure cannot implicitly release the critical section. See the
/// safety notes in the documentation for
/// [`pyo3::sync::critical_section::EnteredCriticalSection`](crate::sync::critical_section::EnteredCriticalSection)
/// for more details.
#[cfg(all(Py_3_14, not(Py_LIMITED_API)))]
#[cfg_attr(not(Py_GIL_DISABLED), allow(unused_variables))]
pub fn with_critical_section_mutex<F, R, T>(_py: Python<'_>, mutex: &PyMutex<T>, f: F) -> R
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

/// Executes a closure with a Python critical section held on two `PyMutex` instances.
///
/// Simultaneously locks the mutexes `m1` and `m2` and holds them until the closure `f` is
/// finished. The mutexes may be temporarily unlock and re-acquired if the closure calls back into
/// the interpreter. See the notes in the
/// [`pyo3::sync::critical_section`][crate::sync::critical_section] module documentation for more
/// details.
///
/// This is structurally equivalent to the use of the paired
/// Py_BEGIN_CRITICAL_SECTION2_MUTEX and Py_END_CRITICAL_SECTION2 C-API macros.
///
/// A no-op on GIL-enabled builds, where the critical section API is exposed as
/// a no-op by the Python C API.
///
/// # Safety
///
/// The caller must ensure the closure cannot implicitly release the critical section. See the
/// safety notes in the documentation for
/// [`pyo3::sync::critical_section::EnteredCriticalSection`](crate::sync::critical_section::EnteredCriticalSection)
/// for more details.
#[cfg(all(Py_3_14, not(Py_LIMITED_API)))]
#[cfg_attr(not(Py_GIL_DISABLED), allow(unused_variables))]
pub fn with_critical_section_mutex2<F, R, T1, T2>(
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

// We are building wasm Python with pthreads disabled and all these
// tests use threads
#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    #[cfg(feature = "macros")]
    use super::{with_critical_section, with_critical_section2};
    #[cfg(all(not(Py_LIMITED_API), Py_3_14))]
    use super::{with_critical_section_mutex, with_critical_section_mutex2};
    #[cfg(all(not(Py_LIMITED_API), Py_3_14))]
    use crate::types::PyMutex;
    #[cfg(feature = "macros")]
    use std::sync::atomic::{AtomicBool, Ordering};
    #[cfg(any(feature = "macros", all(not(Py_LIMITED_API), Py_3_14)))]
    use std::sync::Barrier;

    #[cfg(feature = "macros")]
    use crate::Py;
    #[cfg(any(feature = "macros", all(not(Py_LIMITED_API), Py_3_14)))]
    use crate::Python;

    #[cfg(feature = "macros")]
    #[crate::pyclass(crate = "crate")]
    struct VecWrapper(Vec<isize>);

    #[cfg(feature = "macros")]
    #[crate::pyclass(crate = "crate")]
    struct BoolWrapper(AtomicBool);

    #[cfg(feature = "macros")]
    #[test]
    fn test_critical_section() {
        let barrier = Barrier::new(2);

        let bool_wrapper = Python::attach(|py| -> Py<BoolWrapper> {
            Py::new(py, BoolWrapper(AtomicBool::new(false))).unwrap()
        });

        std::thread::scope(|s| {
            s.spawn(|| {
                Python::attach(|py| {
                    let b = bool_wrapper.bind(py);
                    with_critical_section(b, || {
                        barrier.wait();
                        std::thread::sleep(std::time::Duration::from_millis(10));
                        b.borrow().0.store(true, Ordering::Release);
                    })
                });
            });
            s.spawn(|| {
                barrier.wait();
                Python::attach(|py| {
                    let b = bool_wrapper.bind(py);
                    // this blocks until the other thread's critical section finishes
                    with_critical_section(b, || {
                        assert!(b.borrow().0.load(Ordering::Acquire));
                    });
                });
            });
        });
    }

    #[cfg(all(not(Py_LIMITED_API), Py_3_14))]
    #[test]
    fn test_critical_section_mutex() {
        let barrier = Barrier::new(2);

        let mutex = PyMutex::new(false);

        std::thread::scope(|s| {
            s.spawn(|| {
                Python::attach(|py| {
                    with_critical_section_mutex(py, &mutex, |mut b| {
                        barrier.wait();
                        std::thread::sleep(std::time::Duration::from_millis(10));
                        // SAFETY: we never call back into the python interpreter inside this critical section
                        *(unsafe { b.get_mut() }) = true;
                    });
                });
            });
            s.spawn(|| {
                barrier.wait();
                Python::attach(|py| {
                    // blocks until the other thread enters a critical section
                    with_critical_section_mutex(py, &mutex, |b| {
                        // SAFETY: we never call back into the python interpreter inside this critical section
                        assert!(unsafe { *b.get() });
                    });
                });
            });
        });
    }

    #[cfg(feature = "macros")]
    #[test]
    fn test_critical_section2() {
        let barrier = Barrier::new(3);

        let (bool_wrapper1, bool_wrapper2) = Python::attach(|py| {
            (
                Py::new(py, BoolWrapper(AtomicBool::new(false))).unwrap(),
                Py::new(py, BoolWrapper(AtomicBool::new(false))).unwrap(),
            )
        });

        std::thread::scope(|s| {
            s.spawn(|| {
                Python::attach(|py| {
                    let b1 = bool_wrapper1.bind(py);
                    let b2 = bool_wrapper2.bind(py);
                    with_critical_section2(b1, b2, || {
                        barrier.wait();
                        std::thread::sleep(std::time::Duration::from_millis(10));
                        b1.borrow().0.store(true, Ordering::Release);
                        b2.borrow().0.store(true, Ordering::Release);
                    })
                });
            });
            s.spawn(|| {
                barrier.wait();
                Python::attach(|py| {
                    let b1 = bool_wrapper1.bind(py);
                    // this blocks until the other thread's critical section finishes
                    with_critical_section(b1, || {
                        assert!(b1.borrow().0.load(Ordering::Acquire));
                    });
                });
            });
            s.spawn(|| {
                barrier.wait();
                Python::attach(|py| {
                    let b2 = bool_wrapper2.bind(py);
                    // this blocks until the other thread's critical section finishes
                    with_critical_section(b2, || {
                        assert!(b2.borrow().0.load(Ordering::Acquire));
                    });
                });
            });
        });
    }

    #[cfg(all(Py_3_14, not(Py_LIMITED_API)))]
    #[test]
    fn test_critical_section_mutex2() {
        let barrier = Barrier::new(2);

        let m1 = PyMutex::new(false);
        let m2 = PyMutex::new(false);

        std::thread::scope(|s| {
            s.spawn(|| {
                Python::attach(|py| {
                    with_critical_section_mutex2(py, &m1, &m2, |mut b1, mut b2| {
                        barrier.wait();
                        std::thread::sleep(std::time::Duration::from_millis(10));
                        // SAFETY: we never call back into the python interpreter inside this critical section
                        unsafe { (*b1.get_mut()) = true };
                        unsafe { (*b2.get_mut()) = true };
                    });
                });
            });
            s.spawn(|| {
                barrier.wait();
                Python::attach(|py| {
                    // blocks until the other thread enters a critical section
                    with_critical_section_mutex2(py, &m1, &m2, |b1, b2| {
                        // SAFETY: we never call back into the python interpreter inside this critical section
                        assert!(unsafe { *b1.get() });
                        assert!(unsafe { *b2.get() });
                    });
                });
            });
        });
    }

    #[cfg(feature = "macros")]
    #[test]
    fn test_critical_section2_same_object_no_deadlock() {
        let barrier = Barrier::new(2);

        let bool_wrapper = Python::attach(|py| -> Py<BoolWrapper> {
            Py::new(py, BoolWrapper(AtomicBool::new(false))).unwrap()
        });

        std::thread::scope(|s| {
            s.spawn(|| {
                Python::attach(|py| {
                    let b = bool_wrapper.bind(py);
                    with_critical_section2(b, b, || {
                        barrier.wait();
                        std::thread::sleep(std::time::Duration::from_millis(10));
                        b.borrow().0.store(true, Ordering::Release);
                    })
                });
            });
            s.spawn(|| {
                barrier.wait();
                Python::attach(|py| {
                    let b = bool_wrapper.bind(py);
                    // this blocks until the other thread's critical section finishes
                    with_critical_section(b, || {
                        assert!(b.borrow().0.load(Ordering::Acquire));
                    });
                });
            });
        });
    }

    #[cfg(all(Py_3_14, not(Py_LIMITED_API)))]
    #[test]
    fn test_critical_section_mutex2_same_object_no_deadlock() {
        let barrier = Barrier::new(2);

        let m = PyMutex::new(false);

        std::thread::scope(|s| {
            s.spawn(|| {
                Python::attach(|py| {
                    with_critical_section_mutex2(py, &m, &m, |mut b1, b2| {
                        barrier.wait();
                        std::thread::sleep(std::time::Duration::from_millis(10));
                        // SAFETY: we never call back into the python interpreter inside this critical section
                        unsafe { (*b1.get_mut()) = true };
                        assert!(unsafe { *b2.get() });
                    });
                });
            });
            s.spawn(|| {
                barrier.wait();
                Python::attach(|py| {
                    // this blocks until the other thread's critical section finishes
                    with_critical_section_mutex(py, &m, |b| {
                        // SAFETY: we never call back into the python interpreter inside this critical section
                        assert!(unsafe { *b.get() });
                    });
                });
            });
        });
    }

    #[cfg(feature = "macros")]
    #[test]
    fn test_critical_section2_two_containers() {
        let (vec1, vec2) = Python::attach(|py| {
            (
                Py::new(py, VecWrapper(vec![1, 2, 3])).unwrap(),
                Py::new(py, VecWrapper(vec![4, 5])).unwrap(),
            )
        });

        std::thread::scope(|s| {
            s.spawn(|| {
                Python::attach(|py| {
                    let v1 = vec1.bind(py);
                    let v2 = vec2.bind(py);
                    with_critical_section2(v1, v2, || {
                        // v2.extend(v1)
                        v2.borrow_mut().0.extend(v1.borrow().0.iter());
                    })
                });
            });
            s.spawn(|| {
                Python::attach(|py| {
                    let v1 = vec1.bind(py);
                    let v2 = vec2.bind(py);
                    with_critical_section2(v1, v2, || {
                        // v1.extend(v2)
                        v1.borrow_mut().0.extend(v2.borrow().0.iter());
                    })
                });
            });
        });

        Python::attach(|py| {
            let v1 = vec1.bind(py);
            let v2 = vec2.bind(py);
            // execution order is not guaranteed, so we need to check both
            // NB: extend should be atomic, items must not be interleaved
            // v1.extend(v2)
            // v2.extend(v1)
            let expected1_vec1 = vec![1, 2, 3, 4, 5];
            let expected1_vec2 = vec![4, 5, 1, 2, 3, 4, 5];
            // v2.extend(v1)
            // v1.extend(v2)
            let expected2_vec1 = vec![1, 2, 3, 4, 5, 1, 2, 3];
            let expected2_vec2 = vec![4, 5, 1, 2, 3];

            assert!(
                (v1.borrow().0.eq(&expected1_vec1) && v2.borrow().0.eq(&expected1_vec2))
                    || (v1.borrow().0.eq(&expected2_vec1) && v2.borrow().0.eq(&expected2_vec2))
            );
        });
    }

    #[cfg(all(Py_3_14, not(Py_LIMITED_API)))]
    #[test]
    fn test_critical_section_mutex2_two_containers() {
        let (m1, m2) = (PyMutex::new(vec![1, 2, 3]), PyMutex::new(vec![4, 5]));

        let (m1_guard, m2_guard) = (m1.lock().unwrap(), m2.lock().unwrap());

        std::thread::scope(|s| {
            s.spawn(|| {
                Python::attach(|py| {
                    with_critical_section_mutex2(py, &m1, &m2, |mut v1, v2| {
                        // v1.extend(v1)
                        // SAFETY: we never call back into the python interpreter inside this critical section
                        let vec1 = unsafe { v1.get_mut() };
                        let vec2 = unsafe { v2.get() };
                        vec1.extend(vec2.iter());
                    })
                });
            });
            s.spawn(|| {
                Python::attach(|py| {
                    with_critical_section_mutex2(py, &m1, &m2, |v1, mut v2| {
                        // v2.extend(v1)
                        // SAFETY: we never call back into the python interpreter inside this critical section
                        let vec1 = unsafe { v1.get() };
                        let vec2 = unsafe { v2.get_mut() };
                        vec2.extend(vec1.iter());
                    })
                });
            });
            // the other threads waiting for locks should not block this attach
            Python::attach(|_| {
                // On the free-threaded build, the critical sections should have blocked
                // the other threads from modification.
                #[cfg(Py_GIL_DISABLED)]
                {
                    assert_eq!(&*m1_guard, &[1, 2, 3]);
                    assert_eq!(&*m2_guard, &[4, 5]);
                }
            });
            drop(m1_guard);
            drop(m2_guard);
        });

        // execution order is not guaranteed, so we need to check both
        // NB: extend should be atomic, items must not be interleaved
        // v1.extend(v2)
        // v2.extend(v1)
        let expected1_vec1 = vec![1, 2, 3, 4, 5];
        let expected1_vec2 = vec![4, 5, 1, 2, 3, 4, 5];
        // v2.extend(v1)
        // v1.extend(v2)
        let expected2_vec1 = vec![1, 2, 3, 4, 5, 1, 2, 3];
        let expected2_vec2 = vec![4, 5, 1, 2, 3];

        let v1 = m1.lock().unwrap();
        let v2 = m2.lock().unwrap();
        assert!(
            (&*v1, &*v2) == (&expected1_vec1, &expected1_vec2)
                || (&*v1, &*v2) == (&expected2_vec1, &expected2_vec2)
        );
    }
}
