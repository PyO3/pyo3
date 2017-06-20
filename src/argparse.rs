// Copyright (c) 2017-present PyO3 Project and Contributors
//
// based on Daniel Grunwald's https://github.com/dgrunwald/rust-cpython


use ffi;
use python::Python;
use objects::{PyObject, PyTuple, PyDict, PyString, exc};
use conversion::RefFromPyObject;
use err::{self, PyResult};

/// Description of a python parameter; used for `parse_args()`.
pub struct ParamDescription<'a> {
    /// The name of the parameter.
    pub name: &'a str,
    /// Whether the parameter is optional.
    pub is_optional: bool,
    /// Whether the parameter is optional.
    pub kw_only: bool
}

/// Parse argument list
///
/// * fname:  Name of the current function
/// * params: Declared parameters of the function
/// * args:   Positional arguments
/// * kwargs: Keyword arguments
/// * output: Output array that receives the arguments.
///           Must have same length as `params` and must be initialized to `None`.
pub fn parse_args<'p>(py: Python<'p>,
                      fname: Option<&str>, params: &[ParamDescription],
                      args: &'p PyTuple, kwargs: Option<&'p PyDict>,
                      accept_args: bool, accept_kwargs: bool,
                      output: &mut[Option<PyObject>]) -> PyResult<()>
{
    assert!(params.len() == output.len());

    let nargs = args.len(py);
    let nkeywords = kwargs.map_or(0, |d| d.len(py));
    if !accept_args && (nargs + nkeywords > params.len()) {
        return Err(err::PyErr::new::<exc::TypeError, _>(
            py,
            format!("{}{} takes at most {} argument{} ({} given)",
                    fname.unwrap_or("function"),
                    if fname.is_some() { "()" } else { "" },
                    params.len(),
                    if params.len() == 1 { "s" } else { "" },
                    nargs + nkeywords
            )));
    }
    let mut used_keywords = 0;
    // Iterate through the parameters and assign values to output:
    for (i, (p, out)) in params.iter().zip(output).enumerate() {
        match kwargs.and_then(|d| d.get_item(py, p.name)) {
            Some(kwarg) => {
                *out = Some(kwarg);
                used_keywords += 1;
                if i < nargs {
                    return Err(err::PyErr::new::<exc::TypeError, _>(
                        py,
                        format!("Argument given by name ('{}') and position ({})", p.name, i+1)));
                }
            },
            None => {
                if p.kw_only {
                    *out = None;
                }
                else if i < nargs {
                    *out = Some(args.get_item(py, i));
                } else {
                    *out = None;
                    if !p.is_optional {
                        return Err(err::PyErr::new::<exc::TypeError, _>(
                            py,
                            format!("Required argument ('{}') (pos {}) not found",
                                    p.name, i+1)));
                    }
                }
            }
        }
    }
    if !accept_kwargs && used_keywords != nkeywords {
        // check for extraneous keyword arguments
        for (key, _value) in kwargs.unwrap().items(py) {
            let key = try!(try!(key.cast_as::<PyString>(py)).to_string(py));
            if !params.iter().any(|p| p.name == key) {
                return Err(err::PyErr::new::<exc::TypeError, _>(
                    py,
                    format!("'{}' is an invalid keyword argument for this function",
                            key)));
            }
        }
    }
    Ok(())
}

#[inline]
#[doc(hidden)]
pub unsafe fn get_kwargs(py: Python, ptr: *mut ffi::PyObject) -> Option<PyDict> {
    if ptr.is_null() {
        None
    } else {
        Some(PyDict::from_borrowed_ptr(py, ptr))
    }
}

#[doc(hidden)] // used in py_argparse_extract!() macro
pub fn with_extracted_or_default<'p, P: ?Sized, R, F>(
    py: Python, obj: Option<&'p PyObject>, f: F, default: &'static P) -> PyResult<R>
    where F: FnOnce(&P) -> PyResult<R>,
          P: RefFromPyObject<'p>
{
    match obj {
        Some(obj) => match P::with_extracted(py, obj, f) {
            Ok(result) => result,
            Err(e) => Err(e)
        },
        None => f(default)
    }
}
