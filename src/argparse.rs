// Copyright (c) 2015 Daniel Grunwald
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of this
// software and associated documentation files (the "Software"), to deal in the Software
// without restriction, including without limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons
// to whom the Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all copies or
// substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED,
// INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR
// PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE
// FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR
// OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

//! This module contains logic for parsing a python argument list.

use std::ptr;
use python::{Python, PythonObject};
use objects::{PyObject, PyTuple, PyDict, PyString, exc};
use conversion::ToPyObject;
use ffi;
use err::{self, PyResult};

pub struct ParamDescription<'a> {
    name: &'a str,
    is_optional: bool
}

/// Parse argument list
///
/// fname:  Name of the current function
/// params: Declared parameters of the function
/// args:   Positional arguments
/// kwargs: Keyword arguments
/// output: Output array that receives the arguments.
///         Must have same length as `params` and must be initialized to `None`.
pub fn parse_args<'p>(
    fname: Option<&str>, params: &[ParamDescription],
    args: &PyTuple<'p>, kwargs: Option<&PyDict<'p>>,
    output: &mut[Option<PyObject<'p>>]
) -> PyResult<'p, ()>
{
    assert!(params.len() == output.len());
    let py = args.python();
    let nargs = args.len();
    let nkeywords = kwargs.map_or(0, |d| d.len());
    if nargs + nkeywords > params.len() {
        return Err(err::PyErr::new::<exc::TypeError, _>(py,
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
        match kwargs.and_then(|d| d.get_item(p.name)) {
            Some(kwarg) => {
                *out = Some(kwarg);
                used_keywords += 1;
                if i < nargs {
                    return Err(err::PyErr::new::<exc::TypeError, _>(py,
                        format!("Argument given by name ('{}') and position ({})",
                                p.name, i+1)));
                }
            },
            None => {
                if i < nargs {
                    *out = Some(args.get_item(i));
                } else {
                    *out = None;
                    if !p.is_optional {
                        return Err(err::PyErr::new::<exc::TypeError, _>(py,
                            format!("Required argument ('{}') (pos {}) not found",
                                    p.name, i+1)));
                    }
                }
            }
        }
    }
    if used_keywords != nkeywords {
        // check for extraneous keyword arguments
        for (key, _value) in kwargs.unwrap().items() {
            let key = try!(PyString::extract(&key));
            if !params.iter().any(|p| p.name == key) {
                return Err(err::PyErr::new::<exc::TypeError, _>(py,
                    format!("'{}' is an invalid keyword argument for this function",
                            key)));
            }
        }
    }
    Ok(())
}

macro_rules! argparse_extract {
    ( ( ) $body:block ) => { $body };
    ( ( $pname:ident : $ptype:ty ) $body:block) => {
        match <$ptype as $crate::ExtractPyObject>::prepare_extract($pname.as_ref().unwrap()) {
            Ok(prepared) => {
                match <$ptype as $crate::ExtractPyObject>::extract(&prepared) {
                    Ok($pname) => $body,
                    Err(e) => Err(e)
                }
            },
            Err(e) => Err(e)
        }
    };
    ( ( $pname:ident : $ptype:ty , $($r:tt)+ ) $body:block) => {
        argparse_extract!(($pname: $ptype) {
            argparse_extract!( ( $($r)* ) $body)
        })
    }
}

macro_rules! argparse_snd {
    ( $fst:expr, $snd:expr) => { $snd }
}

macro_rules! argparse {
    ($fname:expr, $args:expr, $kwargs:expr, ($( $pname:ident : $ptype:ty ),*) $body:block) => {{
        const PARAMS: &'static [$crate::argparse::ParamDescription<'static>] = &[
            $(
                $crate::argparse::ParamDescription {
                    name: stringify!($pname),
                    is_optional: false
                }
            ),*
        ];
        let mut output = [$( argparse_snd!($pname, None) ),*];
        match $crate::argparse::parse_args($fname, PARAMS, $args, $kwargs, &mut output) {
            Ok(()) => {
                let &[$(ref $pname),*] = &output;
                argparse_extract!( ( $( $pname : $ptype ),* ) $body )
            },
            Err(e) => Err(e)
        }
    }}
}

#[cfg(test)]
mod test {
    use python::{Python, PythonObject};
    use conversion::ToPyObject;

    #[test]
    pub fn test_parse() {
        let gil_guard = Python::acquire_gil();
        let py = gil_guard.python();
        let mut called = false;
        let tuple = ("abc", 42).to_py_object(py);
        argparse!(None, &tuple, None, (x: &str, y: i32) {
            assert_eq!(x, "abc");
            assert_eq!(y, 42);
            called = true;
            Ok(())
        }).unwrap();
        assert!(called);
    }
}

