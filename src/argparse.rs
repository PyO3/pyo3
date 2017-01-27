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
//! See also the macros `py_argparse!`, `py_fn!` and `py_method!`.

use std::ptr;
use python::{Python, PythonObject};
use objects::{PyObject, PyTuple, PyDict, PyString, exc};
use conversion::{RefFromPyObject, ToPyObject};
use ffi;
use err::{self, PyResult};

/// Description of a python parameter; used for `parse_args()`.
pub struct ParamDescription<'a> {
    /// The name of the parameter.
    pub name: &'a str,
    /// Whether the parameter is optional.
    pub is_optional: bool
}

/// Parse argument list
///
/// * fname:  Name of the current function
/// * params: Declared parameters of the function
/// * args:   Positional arguments
/// * kwargs: Keyword arguments
/// * output: Output array that receives the arguments.
///           Must have same length as `params` and must be initialized to `None`.
pub fn parse_args(
    py: Python,
    fname: Option<&str>, params: &[ParamDescription],
    args: &PyTuple, kwargs: Option<&PyDict>,
    output: &mut[Option<PyObject>]
) -> PyResult<()>
{
    assert!(params.len() == output.len());
    let nargs = args.len(py);
    let nkeywords = kwargs.map_or(0, |d| d.len(py));
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
        match kwargs.and_then(|d| d.get_item(py, p.name)) {
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
                    *out = Some(args.get_item(py, i));
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
        for (key, _value) in kwargs.unwrap().items(py) {
            let key = try!(try!(key.cast_as::<PyString>(py)).to_string(py));
            if !params.iter().any(|p| p.name == key) {
                return Err(err::PyErr::new::<exc::TypeError, _>(py,
                    format!("'{}' is an invalid keyword argument for this function",
                            key)));
            }
        }
    }
    Ok(())
}

/// This macro is used to parse a parameter list into a set of variables.
///
/// Syntax: `py_argparse!(py, fname, args, kwargs, (parameter-list) { body })`
///
/// * `py`: the `Python` token
/// * `fname`: expression of type `Option<&str>`: Name of the function used in error messages.
/// * `args`: expression of type `&PyTuple`: The position arguments
/// * `kwargs`: expression of type `Option<&PyDict>`: The named arguments
/// * `parameter-list`: a comma-separated list of parameter declarations.
///   Parameter declarations have one of these formats:
///    1. `name`
///    2. `name: ty`
///    3. `name: ty = default_value`
///    4. `*name`
///    5. `*name : ty`
///    6. `**name`
///    7. `**name : ty`
///
///   The types used must implement the `FromPyObject` trait.
///   If no type is specified, the parameter implicitly uses
///   `&PyObject` (format 1), `&PyTuple` (format 4) or `&PyDict` (format 6).
///   If a default value is specified, it must be a compile-time constant
//    of type `ty`.
/// * `body`: expression of type `PyResult<_>`.
///     The extracted argument values are available in this scope.
///
/// `py_argparse!()` expands to code that extracts values from `args` and `kwargs` and assigns
/// them to the parameters. If the extraction is successful, `py_argparse!()` evaluates
/// the body expression and returns of that evaluation.
/// If extraction fails, `py_argparse!()` returns a failed `PyResult` without evaluating `body`.
///
/// The `py_argparse!()` macro special-cases reference types (when `ty` starts with a `&` token):
/// In this case, the macro uses the `RefFromPyObject` trait instead of the `FromPyObject` trait.
/// When using at least one reference parameter, the `body` block is placed within a closure,
/// so `return` statements might behave unexpectedly in this case. (this only affects direct use
/// of `py_argparse!`; `py_fn!` is unaffected as the body there is always in a separate function
/// from the generated argument-parsing code).
#[macro_export]
macro_rules! py_argparse {
    ($py:expr, $fname:expr, $args:expr, $kwargs:expr, $plist:tt $body:block) => {
        py_argparse_parse_plist! { py_argparse_impl { $py, $fname, $args, $kwargs, $body, } $plist }
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_argparse_parse_plist {
    // Parses a parameter-list into a format more suitable for consumption by Rust macros.
    // py_argparse_parse_plist! { callback { initial_args } (plist) }
    //  = callback! { initial_args [{ pname:ptype = [ {**} {default-value} ] } ...] }
    // The braces around the *s and the default-value are used even if they are empty.

    // Special-case entry-point for empty parameter list:
    { $callback:ident { $($initial_arg:tt)* } ( ) } => {
        $callback! { $($initial_arg)* [] }
    };
    // Regular entry point for non-empty parameter list:
    { $callback:ident $initial_args:tt ( $( $p:tt )+ ) } => {
        // add trailing comma to plist so that the parsing step can assume every
        // parameter ends with a comma.
        py_argparse_parse_plist_impl! { $callback $initial_args [] ( $($p)*, ) }
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_argparse_parse_plist_impl {
    // TT muncher macro that does the main work for py_argparse_parse_plist!.

    // Base case: all parameters handled
    { $callback:ident { $($initial_arg:tt)* } $output:tt ( ) } => {
        $callback! { $($initial_arg)* $output }
    };
    // Kwargs parameter with reference extraction
    { $callback:ident $initial_args:tt [ $($output:tt)* ]
        ( ** $name:ident : &$t:ty , $($tail:tt)* )
    } => {
        py_argparse_parse_plist_impl! {
            $callback $initial_args
            [ $($output)* { $name:&$t = [ {**} {} {$t} ] } ]
            ($($tail)*)
        }
    };
    // Kwargs parameter
    { $callback:ident $initial_args:tt [ $($output:tt)* ]
        ( ** $name:ident : $t:ty , $($tail:tt)* )
    } => {
        py_argparse_parse_plist_impl! {
            $callback $initial_args
            [ $($output)* { $name:$t = [ {**} {} {} ] } ]
            ($($tail)*)
        }
    };
    // Kwargs parameter with implicit type
    { $callback:ident $initial_args:tt [ $($output:tt)* ]
        ( ** $name:ident , $($tail:tt)* )
    } => {
        py_argparse_parse_plist_impl! {
            $callback $initial_args
            [ $($output)* { $name:Option<&$crate::PyDict> = [ {**} {} {} ] } ]
            ($($tail)*)
        }
    };
    // Varargs parameter with reference extraction
    { $callback:ident $initial_args:tt [ $($output:tt)* ]
        ( * $name:ident : &$t:ty , $($tail:tt)* )
    } => {
        py_argparse_parse_plist_impl! {
            $callback $initial_args
            [ $($output)* { $name:&$t = [ {*} {} {$t} ] } ]
            ($($tail)*)
        }
    };
    // Varargs parameter
    { $callback:ident $initial_args:tt [ $($output:tt)* ]
        ( * $name:ident : $t:ty , $($tail:tt)* )
    } => {
        py_argparse_parse_plist_impl! {
            $callback $initial_args
            [ $($output)* { $name:$t = [ {*} {} {} ] } ]
            ($($tail)*)
        }
    };
    // Varargs parameter with implicit type
    { $callback:ident $initial_args:tt [ $($output:tt)* ]
        ( * $name:ident , $($tail:tt)* )
    } => {
        py_argparse_parse_plist_impl! {
            $callback $initial_args
            [ $($output)* { $name:&$crate::PyTuple = [ {*} {} {} ] } ]
            ($($tail)*)
        }
    };
    // Simple parameter with reference extraction
    { $callback:ident $initial_args:tt [ $($output:tt)* ]
        ( $name:ident : &$t:ty , $($tail:tt)* )
    } => {
        py_argparse_parse_plist_impl! {
            $callback $initial_args
            [ $($output)* { $name:&$t = [ {} {} {$t} ] } ]
            ($($tail)*)
        }
    };
    // Simple parameter
    { $callback:ident $initial_args:tt [ $($output:tt)* ]
        ( $name:ident : $t:ty , $($tail:tt)* )
    } => {
        py_argparse_parse_plist_impl! {
            $callback $initial_args
            [ $($output)* { $name:$t = [ {} {} {} ] } ]
            ($($tail)*)
        }
    };
    // Simple parameter with implicit type
    { $callback:ident $initial_args:tt [ $($output:tt)* ]
        ( $name:ident , $($tail:tt)* )
    } => {
        py_argparse_parse_plist_impl! {
            $callback $initial_args
            [ $($output)* { $name:&$crate::PyObject = [ {} {} {} ] } ]
            ($($tail)*)
        }
    };
    // Optional parameter with reference extraction
    { $callback:ident $initial_args:tt [ $($output:tt)* ]
        ( $name:ident : &$t:ty = $default:expr, $($tail:tt)* )
    } => {
        py_argparse_parse_plist_impl! {
            $callback $initial_args
            [ $($output)* { $name:&$t = [ {} {$default} {$t} ] } ]
            ($($tail)*)
        }
    };
    // Optional parameter
    { $callback:ident $initial_args:tt [ $($output:tt)* ]
        ( $name:ident : $t:ty = $default:expr , $($tail:tt)* )
    } => {
        py_argparse_parse_plist_impl! {
            $callback $initial_args
            [ $($output)* { $name:$t = [ {} {$default} {} ] } ]
            ($($tail)*)
        }
    };
}

// The main py_argparse!() macro, except that it expects the parameter-list
// in the output format of py_argparse_parse_plist!().
#[macro_export]
#[doc(hidden)]
macro_rules! py_argparse_impl {
    // special case: function signature is (*args, **kwargs),
    // so we can directly pass along our inputs without calling parse_args().
    ($py:expr, $fname:expr, $args:expr, $kwargs:expr, $body:block,
        [
            { $pargs:ident   : $pargs_type:ty   = [ {*}  {} {} ] }
            { $pkwargs:ident : $pkwargs_type:ty = [ {**} {} {} ] }
        ]
    ) => {{
        let _py: $crate::Python = $py;
        // TODO: use extract() to be more flexible in which type is expected
        let $pargs: $pargs_type = $args;
        let $pkwargs: $pkwargs_type = $kwargs;
        $body
    }};

    // normal argparse logic
    ($py:expr, $fname:expr, $args:expr, $kwargs:expr, $body:block,
        [ $( { $pname:ident : $ptype:ty = $detail:tt } )* ]
    ) => {{
        const PARAMS: &'static [$crate::argparse::ParamDescription<'static>] = &[
            $(
                py_argparse_param_description! { $pname : $ptype = $detail }
            ),*
        ];
        let py: $crate::Python = $py;
        let mut output = [$( py_replace_expr!($pname None) ),*];
        match $crate::argparse::parse_args(py, $fname, PARAMS, $args, $kwargs, &mut output) {
            Ok(()) => {
                // Experimental slice pattern syntax would be really nice here (#23121)
                //let [$(ref $pname),*] = output;
                // We'll use an iterator instead.
                let mut _iter = output.iter();
                // We'll have to generate a bunch of nested `match` statements
                // (at least until we can use ? + catch, assuming that will be hygienic wrt. macros),
                // so use a recursive helper macro for that:
                py_argparse_extract!( py, _iter, $body,
                    [ $( { $pname : $ptype = $detail } )* ])
            },
            Err(e) => Err(e)
        }
    }};
}

// Like py_argparse_impl!(), but accepts `*mut ffi::PyObject` for $args and $kwargs.
#[macro_export]
#[doc(hidden)]
macro_rules! py_argparse_raw {
    ($py:ident, $fname:expr, $args:expr, $kwargs:expr, $plist:tt $body:block) => {{
        let args: $crate::PyTuple = $crate::PyObject::from_borrowed_ptr($py, $args).unchecked_cast_into();
        let kwargs: Option<$crate::PyDict> = $crate::argparse::get_kwargs($py, $kwargs);
        let ret = py_argparse_impl!($py, $fname, &args, kwargs.as_ref(), $body, $plist);
        $crate::PyDrop::release_ref(args, $py);
        $crate::PyDrop::release_ref(kwargs, $py);
        ret
    }};
}

#[inline]
#[doc(hidden)]
pub unsafe fn get_kwargs(py: Python, ptr: *mut ffi::PyObject) -> Option<PyDict> {
    if ptr.is_null() {
        None
    } else {
        Some(PyObject::from_borrowed_ptr(py, ptr).unchecked_cast_into())
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_argparse_param_description {
    // normal parameter
    { $pname:ident : $ptype:ty = [ {} {} $rtype:tt ] } => (
        $crate::argparse::ParamDescription {
            name: stringify!($pname),
            is_optional: false
        }
    );
    // optional parameters
    { $pname:ident : $ptype:ty = [ {} {$default:expr} {$($rtype:tt)*} ] } => (
        $crate::argparse::ParamDescription {
            name: stringify!($pname),
            is_optional: true
        }
    );
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_argparse_extract {
    // base case
    ( $py:expr, $iter:expr, $body:block, [] ) => { $body };
    // normal parameter
    ( $py:expr, $iter:expr, $body:block,
        [ { $pname:ident : $ptype:ty = [ {} {} {} ] } $($tail:tt)* ]
    ) => {
        // First unwrap() asserts the iterated sequence is long enough (which should be guaranteed);
        // second unwrap() asserts the parameter was not missing (which fn parse_args already checked for).
        match <$ptype as $crate::FromPyObject>::extract($py, $iter.next().unwrap().as_ref().unwrap()) {
            Ok($pname) => py_argparse_extract!($py, $iter, $body, [$($tail)*]),
            Err(e) => Err(e)
        }
    };
    // normal parameter with reference extraction
    ( $py:expr, $iter:expr, $body:block,
        [ { $pname:ident : $ptype:ty = [ {} {} {$rtype:ty} ] } $($tail:tt)* ]
    ) => {
        // First unwrap() asserts the iterated sequence is long enough (which should be guaranteed);
        // second unwrap() asserts the parameter was not missing (which fn parse_args already checked for).
        match <$rtype as $crate::RefFromPyObject>::with_extracted($py,
            $iter.next().unwrap().as_ref().unwrap(),
            |$pname: $ptype| py_argparse_extract!($py, $iter, $body, [$($tail)*])
        ) {
            Ok(v) => v,
            Err(e) => Err(e)
        }
    };
    // optional parameter
    ( $py:expr, $iter:expr, $body:block,
        [ { $pname:ident : $ptype:ty = [ {} {$default:expr} {} ] } $($tail:tt)* ]
    ) => {
        match $iter.next().unwrap().as_ref().map(|obj| obj.extract::<_>($py)).unwrap_or(Ok($default)) {
            Ok($pname) => py_argparse_extract!($py, $iter, $body, [$($tail)*]),
            Err(e) => Err(e)
        }
    };
    // optional parameter with reference extraction
    ( $py:expr, $iter:expr, $body:block,
        [ { $pname:ident : $ptype:ty = [ {} {$default:expr} {$rtype:ty} ] } $($tail:tt)* ]
    ) => {
        //unwrap() asserts the iterated sequence is long enough (which should be guaranteed);
        $crate::argparse::with_extracted_or_default($py,
            $iter.next().unwrap().as_ref(),
            |$pname: $ptype| py_argparse_extract!($py, $iter, $body, [$($tail)*]),
            $default)
    };
}

#[doc(hidden)] // used in py_argparse_extract!() macro
pub fn with_extracted_or_default<P: ?Sized, R, F>(py: Python, obj: Option<&PyObject>, f: F, default: &'static P) -> PyResult<R>
    where F: FnOnce(&P) -> PyResult<R>,
          P: RefFromPyObject
{
    match obj {
        Some(obj) => match P::with_extracted(py, obj, f) {
            Ok(result) => result,
            Err(e) => Err(e)
        },
        None => f(default)
    }
}

#[cfg(test)]
mod test {
    use python::{Python, PythonObject};
    use objects::PyTuple;
    use conversion::ToPyObject;

    #[test]
    pub fn test_parse() {
        let gil_guard = Python::acquire_gil();
        let py = gil_guard.python();
        let mut called = false;
        let tuple = ("abc", 42).to_py_object(py);
        py_argparse!(py, None, &tuple, None, (x: &str, y: i32) {
            assert_eq!(x, "abc");
            assert_eq!(y, 42);
            called = true;
            Ok(())
        }).unwrap();
        assert!(called);
    }

    #[test]
    pub fn test_default_param_type() {
        let gil_guard = Python::acquire_gil();
        let py = gil_guard.python();
        let mut called = false;
        let tuple = ("abc",).to_py_object(py);
        py_argparse!(py, None, &tuple, None, (x) {
            assert_eq!(*x, tuple.get_item(py, 0));
            called = true;
            Ok(())
        }).unwrap();
        assert!(called);
    }

    #[test]
    pub fn test_default_value() {
        let gil_guard = Python::acquire_gil();
        let py = gil_guard.python();
        let mut called = false;
        let tuple = (0, "foo").to_py_object(py);
        py_argparse!(py, None, &tuple, None, (x: usize = 42, y: &str = "abc") {
            assert_eq!(x, 0);
            assert_eq!(y, "foo");
            called = true;
            Ok(())
        }).unwrap();
        assert!(called);

        let mut called = false;
        let tuple = PyTuple::new(py, &[]);
        py_argparse!(py, None, &tuple, None, (x: usize = 42, y: &str = "abc") {
            assert_eq!(x, 42);
            assert_eq!(y, "abc");
            called = true;
            Ok(())
        }).unwrap();
        assert!(called);
    }
}
