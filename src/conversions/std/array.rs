use crate::conversion::IntoPyObject;
use crate::instance::Bound;
use crate::types::any::PyAnyMethods;
use crate::types::PySequence;
use crate::{err::DowncastError, ffi, FromPyObject, PyAny, PyResult, Python};
use crate::{exceptions, PyErr};

impl<'py, T, const N: usize> IntoPyObject<'py> for [T; N]
where
    T: IntoPyObject<'py>,
{
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    /// Turns [`[u8; N]`](std::array) into [`PyBytes`], all other `T`s will be turned into a [`PyList`]
    ///
    /// [`PyBytes`]: crate::types::PyBytes
    /// [`PyList`]: crate::types::PyList
    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        T::owned_sequence_into_pyobject(self, py, crate::conversion::private::Token)
    }
}

impl<'a, 'py, T, const N: usize> IntoPyObject<'py> for &'a [T; N]
where
    &'a T: IntoPyObject<'py>,
{
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        self.as_slice().into_pyobject(py)
    }
}

impl<'py, T, const N: usize> FromPyObject<'py> for [T; N]
where
    T: FromPyObject<'py>,
{
    fn extract_bound(obj: &Bound<'py, PyAny>) -> PyResult<Self> {
        create_array_from_obj(obj)
    }
}

fn create_array_from_obj<'py, T, const N: usize>(obj: &Bound<'py, PyAny>) -> PyResult<[T; N]>
where
    T: FromPyObject<'py>,
{
    // Types that pass `PySequence_Check` usually implement enough of the sequence protocol
    // to support this function and if not, we will only fail extraction safely.
    let seq = unsafe {
        if ffi::PySequence_Check(obj.as_ptr()) != 0 {
            obj.cast_unchecked::<PySequence>()
        } else {
            return Err(DowncastError::new(obj, "Sequence").into());
        }
    };
    let seq_len = seq.len()?;
    if seq_len != N {
        return Err(invalid_sequence_length(N, seq_len));
    }
    array_try_from_fn(|idx| seq.get_item(idx).and_then(|any| any.extract()))
}

// TODO use std::array::try_from_fn, if that stabilises:
// (https://github.com/rust-lang/rust/issues/89379)
fn array_try_from_fn<E, F, T, const N: usize>(mut cb: F) -> Result<[T; N], E>
where
    F: FnMut(usize) -> Result<T, E>,
{
    // Helper to safely create arrays since the standard library doesn't
    // provide one yet. Shouldn't be necessary in the future.
    struct ArrayGuard<T, const N: usize> {
        dst: *mut T,
        initialized: usize,
    }

    impl<T, const N: usize> Drop for ArrayGuard<T, N> {
        fn drop(&mut self) {
            debug_assert!(self.initialized <= N);
            let initialized_part = core::ptr::slice_from_raw_parts_mut(self.dst, self.initialized);
            unsafe {
                core::ptr::drop_in_place(initialized_part);
            }
        }
    }

    // [MaybeUninit<T>; N] would be "nicer" but is actually difficult to create - there are nightly
    // APIs which would make this easier.
    let mut array: core::mem::MaybeUninit<[T; N]> = core::mem::MaybeUninit::uninit();
    let mut guard: ArrayGuard<T, N> = ArrayGuard {
        dst: array.as_mut_ptr() as _,
        initialized: 0,
    };
    unsafe {
        let mut value_ptr = array.as_mut_ptr() as *mut T;
        for i in 0..N {
            core::ptr::write(value_ptr, cb(i)?);
            value_ptr = value_ptr.offset(1);
            guard.initialized += 1;
        }
        core::mem::forget(guard);
        Ok(array.assume_init())
    }
}

fn invalid_sequence_length(expected: usize, actual: usize) -> PyErr {
    exceptions::PyValueError::new_err(format!(
        "expected a sequence of length {expected} (got {actual})"
    ))
}

#[cfg(test)]
mod tests {
    use std::{
        panic,
        sync::atomic::{AtomicUsize, Ordering},
    };

    use crate::{
        conversion::IntoPyObject,
        ffi,
        types::{any::PyAnyMethods, PyBytes, PyBytesMethods},
    };
    use crate::{types::PyList, PyResult, Python};

    #[test]
    fn array_try_from_fn() {
        static DROP_COUNTER: AtomicUsize = AtomicUsize::new(0);
        struct CountDrop;
        impl Drop for CountDrop {
            fn drop(&mut self) {
                DROP_COUNTER.fetch_add(1, Ordering::SeqCst);
            }
        }
        let _ = catch_unwind_silent(move || {
            let _: Result<[CountDrop; 4], ()> = super::array_try_from_fn(|idx| {
                #[allow(clippy::manual_assert)]
                if idx == 2 {
                    panic!("peek a boo");
                }
                Ok(CountDrop)
            });
        });
        assert_eq!(DROP_COUNTER.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_extract_bytearray_to_array() {
        Python::attach(|py| {
            let v: [u8; 33] = py
                .eval(
                    ffi::c_str!("bytearray(b'abcabcabcabcabcabcabcabcabcabcabc')"),
                    None,
                    None,
                )
                .unwrap()
                .extract()
                .unwrap();
            assert!(&v == b"abcabcabcabcabcabcabcabcabcabcabc");
        })
    }

    #[test]
    fn test_extract_small_bytearray_to_array() {
        Python::attach(|py| {
            let v: [u8; 3] = py
                .eval(ffi::c_str!("bytearray(b'abc')"), None, None)
                .unwrap()
                .extract()
                .unwrap();
            assert!(&v == b"abc");
        });
    }
    #[test]
    fn test_into_pyobject_array_conversion() {
        Python::attach(|py| {
            let array: [f32; 4] = [0.0, -16.0, 16.0, 42.0];
            let pyobject = array.into_pyobject(py).unwrap();
            let pylist = pyobject.cast::<PyList>().unwrap();
            assert_eq!(pylist.get_item(0).unwrap().extract::<f32>().unwrap(), 0.0);
            assert_eq!(pylist.get_item(1).unwrap().extract::<f32>().unwrap(), -16.0);
            assert_eq!(pylist.get_item(2).unwrap().extract::<f32>().unwrap(), 16.0);
            assert_eq!(pylist.get_item(3).unwrap().extract::<f32>().unwrap(), 42.0);
        });
    }

    #[test]
    fn test_extract_invalid_sequence_length() {
        Python::attach(|py| {
            let v: PyResult<[u8; 3]> = py
                .eval(ffi::c_str!("bytearray(b'abcdefg')"), None, None)
                .unwrap()
                .extract();
            assert_eq!(
                v.unwrap_err().to_string(),
                "ValueError: expected a sequence of length 3 (got 7)"
            );
        })
    }

    #[test]
    fn test_intopyobject_array_conversion() {
        Python::attach(|py| {
            let array: [f32; 4] = [0.0, -16.0, 16.0, 42.0];
            let pylist = array
                .into_pyobject(py)
                .unwrap()
                .cast_into::<PyList>()
                .unwrap();

            assert_eq!(pylist.get_item(0).unwrap().extract::<f32>().unwrap(), 0.0);
            assert_eq!(pylist.get_item(1).unwrap().extract::<f32>().unwrap(), -16.0);
            assert_eq!(pylist.get_item(2).unwrap().extract::<f32>().unwrap(), 16.0);
            assert_eq!(pylist.get_item(3).unwrap().extract::<f32>().unwrap(), 42.0);
        });
    }

    #[test]
    fn test_array_intopyobject_impl() {
        Python::attach(|py| {
            let bytes: [u8; 6] = *b"foobar";
            let obj = bytes.into_pyobject(py).unwrap();
            assert!(obj.is_instance_of::<PyBytes>());
            let obj = obj.cast_into::<PyBytes>().unwrap();
            assert_eq!(obj.as_bytes(), &bytes);

            let nums: [u16; 4] = [0, 1, 2, 3];
            let obj = nums.into_pyobject(py).unwrap();
            assert!(obj.is_instance_of::<PyList>());
        });
    }

    #[test]
    fn test_extract_non_iterable_to_array() {
        Python::attach(|py| {
            let v = py.eval(ffi::c_str!("42"), None, None).unwrap();
            v.extract::<i32>().unwrap();
            v.extract::<[i32; 1]>().unwrap_err();
        });
    }

    #[cfg(feature = "macros")]
    #[test]
    fn test_pyclass_intopy_array_conversion() {
        #[crate::pyclass(crate = "crate")]
        struct Foo;

        Python::attach(|py| {
            let array: [Foo; 8] = [Foo, Foo, Foo, Foo, Foo, Foo, Foo, Foo];
            let list = array
                .into_pyobject(py)
                .unwrap()
                .cast_into::<PyList>()
                .unwrap();
            let _bound = list.get_item(4).unwrap().cast::<Foo>().unwrap();
        });
    }

    // https://stackoverflow.com/a/59211505
    fn catch_unwind_silent<F, R>(f: F) -> std::thread::Result<R>
    where
        F: FnOnce() -> R + panic::UnwindSafe,
    {
        let prev_hook = panic::take_hook();
        panic::set_hook(Box::new(|_| {}));
        let result = panic::catch_unwind(f);
        panic::set_hook(prev_hook);
        result
    }
}
