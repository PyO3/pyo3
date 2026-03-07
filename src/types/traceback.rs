use crate::err::{error_on_minusone, PyResult};
use crate::types::{any::PyAnyMethods, string::PyStringMethods, PyString};
use crate::{ffi, Bound, PyAny};
#[cfg(all(not(Py_LIMITED_API), not(PyPy), not(GraalPy)))]
use crate::{
    types::{frame::PyFrameMethods, PyFrame},
    BoundObject, IntoPyObject, PyTypeCheck, Python,
};

/// Represents a Python traceback.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyTraceback>`][crate::Py] or [`Bound<'py, PyTraceback>`][Bound].
///
/// For APIs available on traceback objects, see the [`PyTracebackMethods`] trait which is implemented for
/// [`Bound<'py, PyTraceback>`][Bound].
#[repr(transparent)]
pub struct PyTraceback(PyAny);

pyobject_native_type_core!(
    PyTraceback,
    pyobject_native_static_type_object!(ffi::PyTraceBack_Type),
    "builtins",
    "traceback",
    #checkfunction=ffi::PyTraceBack_Check
);

impl PyTraceback {
    #[cfg(all(not(Py_LIMITED_API), not(PyPy), not(GraalPy)))]
    pub(crate) fn new<'py>(
        py: Python<'py>,
        next: Option<Bound<'py, PyTraceback>>,
        frame: Bound<'py, PyFrame>,
        instruction_index: i32,
        line_number: i32,
    ) -> PyResult<Bound<'py, PyTraceback>> {
        unsafe {
            Ok(PyTraceback::classinfo_object(py)
                .call1((next, frame, instruction_index, line_number))?
                .cast_into_unchecked())
        }
    }

    /// Creates a new traceback object from an iterator of frames.
    ///
    /// The frames should be ordered from newest to oldest, i.e. the first frame in the iterator
    /// will be the innermost frame in the traceback.
    #[cfg(all(not(Py_LIMITED_API), not(PyPy), not(GraalPy)))]
    pub fn from_frames<'py, I>(
        py: Python<'py>,
        frames: I,
    ) -> PyResult<Option<Bound<'py, PyTraceback>>>
    where
        I: IntoIterator,
        I::Item: IntoPyObject<'py, Target = PyFrame>,
    {
        frames.into_iter().try_fold(None, |prev, frame| {
            let frame = frame.into_pyobject(py).map_err(Into::into)?.into_bound();
            let line_number = frame.line_number();
            PyTraceback::new(py, prev, frame, 0, line_number).map(Some)
        })
    }
}

/// Implementation of functionality for [`PyTraceback`].
///
/// These methods are defined for the `Bound<'py, PyTraceback>` smart pointer, so to use method call
/// syntax these methods are separated into a trait, because stable Rust does not yet support
/// `arbitrary_self_types`.
#[doc(alias = "PyTraceback")]
pub trait PyTracebackMethods<'py>: crate::sealed::Sealed {
    /// Formats the traceback as a string.
    ///
    /// This does not include the exception type and value. The exception type and value can be
    /// formatted using the `Display` implementation for `PyErr`.
    ///
    /// # Example
    ///
    /// The following code formats a Python traceback and exception pair from Rust:
    ///
    /// ```rust
    /// # use pyo3::{Python, PyResult, prelude::PyTracebackMethods, ffi::c_str};
    /// # let result: PyResult<()> =
    /// Python::attach(|py| {
    ///     let err = py
    ///         .run(c"raise Exception('banana')", None, None)
    ///         .expect_err("raise will create a Python error");
    ///
    ///     let traceback = err.traceback(py).expect("raised exception will have a traceback");
    ///     assert_eq!(
    ///         format!("{}{}", traceback.format()?, err),
    ///         "\
    /// Traceback (most recent call last):
    ///   File \"<string>\", line 1, in <module>
    /// Exception: banana\
    /// "
    ///     );
    ///     Ok(())
    /// })
    /// # ;
    /// # result.expect("example failed");
    /// ```
    fn format(&self) -> PyResult<String>;

    /// Get the next traceback towards the frame where the exception was raised.
    fn next_traceback(&self) -> PyResult<Option<Bound<'py, PyTraceback>>>;

    /// Get the innermost traceback, i.e. the traceback corresponding to the frame where the exception was raised.
    fn innermost_traceback(&self) -> PyResult<Bound<'py, PyTraceback>>;

    /// Append a traceback to the end of this traceback, i.e. the frames of the appended traceback
    /// will be newer than the frames of this traceback.
    fn append(&self, traceback: Bound<'py, PyTraceback>) -> PyResult<()>;
}

impl<'py> PyTracebackMethods<'py> for Bound<'py, PyTraceback> {
    fn format(&self) -> PyResult<String> {
        let py = self.py();
        let string_io = py
            .import(intern!(py, "io"))?
            .getattr(intern!(py, "StringIO"))?
            .call0()?;
        let result = unsafe { ffi::PyTraceBack_Print(self.as_ptr(), string_io.as_ptr()) };
        error_on_minusone(py, result)?;
        let formatted = string_io
            .getattr(intern!(py, "getvalue"))?
            .call0()?
            .cast::<PyString>()?
            .to_cow()?
            .into_owned();
        Ok(formatted)
    }

    fn next_traceback(&self) -> PyResult<Option<Bound<'py, PyTraceback>>> {
        Ok(self.getattr(intern!(self.py(), "tb_next"))?.extract()?)
    }

    fn innermost_traceback(&self) -> PyResult<Bound<'py, PyTraceback>> {
        let mut current = self.clone();
        while let Some(next) = current.next_traceback()? {
            current = next;
        }
        Ok(current)
    }

    fn append(&self, next: Bound<'py, PyTraceback>) -> PyResult<()> {
        self.innermost_traceback()?
            .setattr(intern!(self.py(), "tb_next"), next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::IntoPyObject;
    use crate::{
        types::{dict::PyDictMethods, PyDict},
        PyErr, Python,
    };

    #[test]
    fn format_traceback() {
        Python::attach(|py| {
            let err = py
                .run(c"raise Exception('banana')", None, None)
                .expect_err("raising should have given us an error");

            assert_eq!(
                err.traceback(py).unwrap().format().unwrap(),
                "Traceback (most recent call last):\n  File \"<string>\", line 1, in <module>\n"
            );
        })
    }

    #[test]
    fn test_err_from_value() {
        Python::attach(|py| {
            let locals = PyDict::new(py);
            // Produce an error from python so that it has a traceback
            py.run(
                cr"
try:
    raise ValueError('raised exception')
except Exception as e:
    err = e
",
                None,
                Some(&locals),
            )
            .unwrap();
            let err = PyErr::from_value(locals.get_item("err").unwrap().unwrap());
            let traceback = err.value(py).getattr("__traceback__").unwrap();
            assert!(err.traceback(py).unwrap().is(&traceback));
        })
    }

    #[test]
    fn test_err_into_py() {
        Python::attach(|py| {
            let locals = PyDict::new(py);
            // Produce an error from python so that it has a traceback
            py.run(
                cr"
def f():
    raise ValueError('raised exception')
",
                None,
                Some(&locals),
            )
            .unwrap();
            let f = locals.get_item("f").unwrap().unwrap();
            let err = f.call0().unwrap_err();
            let traceback = err.traceback(py).unwrap();
            let err_object = err.clone_ref(py).into_pyobject(py).unwrap();

            assert!(err_object.getattr("__traceback__").unwrap().is(&traceback));
        })
    }

    #[test]
    #[cfg(all(not(Py_LIMITED_API), not(PyPy), not(GraalPy)))]
    fn test_create_traceback() {
        Python::attach(|py| {
            // most recent frame first, oldest frame last
            let frames = [
                PyFrame::new(py, c"file3.py", c"func3", 30).unwrap(),
                PyFrame::new(py, c"file2.py", c"func2", 20).unwrap(),
                PyFrame::new(py, c"file1.py", c"func1", 10).unwrap(),
            ];

            let traceback = PyTraceback::from_frames(py, frames).unwrap().unwrap();
            assert_eq!(
                traceback.format().unwrap(), "Traceback (most recent call last):\n  File \"file1.py\", line 10, in func1\n  File \"file2.py\", line 20, in func2\n  File \"file3.py\", line 30, in func3\n"
            );
        })
    }

    #[test]
    #[cfg(all(not(Py_LIMITED_API), not(PyPy), not(GraalPy)))]
    fn test_insert_traceback() {
        Python::attach(|py| {
            let traceback = PyTraceback::from_frames(
                py,
                [
                    PyFrame::new(py, c"file2.py", c"func2", 20).unwrap(),
                    PyFrame::new(py, c"file1.py", c"func1", 10).unwrap(),
                ],
            )
            .unwrap()
            .unwrap();

            let rust_traceback = PyTraceback::from_frames(
                py,
                [
                    PyFrame::new(py, c"rust2.rs", c"func2", 22).unwrap(),
                    PyFrame::new(py, c"rust1.rs", c"func1", 11).unwrap(),
                ],
            )
            .unwrap()
            .unwrap();

            // Stacktrace where python calls into rust
            traceback.append(rust_traceback).unwrap();

            assert_eq!(
                traceback.format().unwrap(),
                r#"Traceback (most recent call last):
  File "file1.py", line 10, in func1
  File "file2.py", line 20, in func2
  File "rust1.rs", line 11, in func1
  File "rust2.rs", line 22, in func2
"#
            );
        })
    }
}
