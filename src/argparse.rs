// Copyright (c) 2017-present PyO3 Project and Contributors
//
// based on Daniel Grunwald's https://github.com/dgrunwald/rust-cpython

//! Python argument parsing
use conversion::PyTryFrom;
use err::PyResult;
use objects::{exc, PyDict, PyObjectRef, PyString, PyTuple};

#[derive(Debug)]
/// Description of a python parameter; used for `parse_args()`.
pub struct ParamDescription<'a> {
    /// The name of the parameter.
    pub name: &'a str,
    /// Whether the parameter is optional.
    pub is_optional: bool,
    /// Whether the parameter is optional.
    pub kw_only: bool,
}

/// Parse argument list
///
/// * fname:  Name of the current function
/// * params: Declared parameters of the function
/// * args:   Positional arguments
/// * kwargs: Keyword arguments
/// * output: Output array that receives the arguments.
///           Must have same length as `params` and must be initialized to `None`.
pub fn parse_args<'p>(
    fname: Option<&str>,
    params: &[ParamDescription],
    args: &'p PyTuple,
    kwargs: Option<&'p PyDict>,
    accept_args: bool,
    accept_kwargs: bool,
    output: &mut [Option<&'p PyObjectRef>],
) -> PyResult<()> {
    let nargs = args.len();
    let nkeywords = kwargs.map_or(0, |d| d.len());
    if !accept_args && (nargs + nkeywords > params.len()) {
        return Err(exc::TypeError::py_err(format!(
            "{}{} takes at most {} argument{} ({} given)",
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
                    return Err(exc::TypeError::py_err(format!(
                        "Argument given by name ('{}') and position ({})",
                        p.name,
                        i + 1
                    )));
                }
            }
            None => {
                if p.kw_only {
                    if !p.is_optional {
                        return Err(exc::TypeError::py_err(format!(
                            "Required argument ('{}') is keyword only argument",
                            p.name
                        )));
                    }
                    *out = None;
                } else if i < nargs {
                    *out = Some(args.get_item(i));
                } else {
                    *out = None;
                    if !p.is_optional {
                        return Err(exc::TypeError::py_err(format!(
                            "Required argument ('{}') (pos {}) not found",
                            p.name,
                            i + 1
                        )));
                    }
                }
            }
        }
    }
    if !accept_kwargs && used_keywords != nkeywords {
        // check for extraneous keyword arguments
        for item in kwargs.unwrap().items().iter() {
            let item = <PyTuple as PyTryFrom>::try_from(item)?;
            let key = <PyString as PyTryFrom>::try_from(item.get_item(0))?.to_string()?;
            if !params.iter().any(|p| p.name == key) {
                return Err(exc::TypeError::py_err(format!(
                    "'{}' is an invalid keyword argument for this function",
                    key
                )));
            }
        }
    }
    Ok(())
}
