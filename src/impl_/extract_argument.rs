use crate::{
    exceptions::PyTypeError,
    ffi,
    type_object::PyTypeObject,
    types::{PyDict, PyString, PyTuple},
    FromPyObject, PyAny, PyErr, PyResult, Python,
};

#[doc(hidden)]
#[inline]
pub fn extract_argument<'py, T>(obj: &'py PyAny, arg_name: &str) -> PyResult<T>
where
    T: FromPyObject<'py>,
{
    match obj.extract() {
        Ok(e) => Ok(e),
        Err(e) => Err(argument_extraction_error(obj.py(), arg_name, e)),
    }
}

/// Adds the argument name to the error message of an error which occurred during argument extraction.
///
/// Only modifies TypeError. (Cannot guarantee all exceptions have constructors from
/// single string.)
#[doc(hidden)]
#[cold]
pub fn argument_extraction_error(py: Python, arg_name: &str, error: PyErr) -> PyErr {
    if error.get_type(py) == PyTypeError::type_object(py) {
        PyTypeError::new_err(format!("argument '{}': {}", arg_name, error.value(py)))
    } else {
        error
    }
}

pub struct KeywordOnlyParameterDescription {
    pub name: &'static str,
    pub required: bool,
}

/// Function argument specification for a `#[pyfunction]` or `#[pymethod]`.
pub struct FunctionDescription {
    pub cls_name: Option<&'static str>,
    pub func_name: &'static str,
    pub positional_parameter_names: &'static [&'static str],
    pub positional_only_parameters: usize,
    pub required_positional_parameters: usize,
    pub keyword_only_parameters: &'static [KeywordOnlyParameterDescription],
    pub accept_varargs: bool,
    pub accept_varkeywords: bool,
}

impl FunctionDescription {
    fn full_name(&self) -> String {
        if let Some(cls_name) = self.cls_name {
            format!("{}.{}()", cls_name, self.func_name)
        } else {
            format!("{}()", self.func_name)
        }
    }

    /// Wrapper around `extract_arguments` which uses the Python C-API "fastcall" convention.
    ///
    /// # Safety
    /// - `args` must be a pointer to a C-style array of valid `ffi::PyObject` pointers.
    /// - `kwnames` must be a pointer to a PyTuple, or NULL.
    /// - `nargs + kwnames.len()` is the total length of the `args` array.
    #[cfg(not(Py_LIMITED_API))]
    pub unsafe fn extract_arguments_fastcall<'py>(
        &self,
        py: Python<'py>,
        args: *const *mut ffi::PyObject,
        nargs: ffi::Py_ssize_t,
        kwnames: *mut ffi::PyObject,
        output: &mut [Option<&'py PyAny>],
    ) -> PyResult<(Option<&'py PyTuple>, Option<&'py PyDict>)> {
        let kwnames: Option<&PyTuple> = py.from_borrowed_ptr_or_opt(kwnames);
        // Safety: &PyAny has the same memory layout as `*mut ffi::PyObject`
        let args = args as *const &PyAny;
        let kwargs = if let Option::Some(kwnames) = kwnames {
            ::std::slice::from_raw_parts(args.offset(nargs), kwnames.len())
        } else {
            &[]
        };
        let args = std::slice::from_raw_parts(args, nargs as usize);
        self.extract_arguments(
            py,
            args.iter().copied(),
            kwnames.map(|kwnames| {
                kwnames
                    .as_slice()
                    .iter()
                    .copied()
                    .zip(kwargs.iter().copied())
            }),
            output,
        )
    }

    /// Wrapper around `extract_arguments` which uses the
    /// tuple-and-dict Python call convention.
    ///
    /// # Safety
    /// - `args` must be a pointer to a PyTuple.
    /// - `kwargs` must be a pointer to a PyDict, or NULL.
    pub unsafe fn extract_arguments_tuple_dict<'py>(
        &self,
        py: Python<'py>,
        args: *mut ffi::PyObject,
        kwargs: *mut ffi::PyObject,
        output: &mut [Option<&'py PyAny>],
    ) -> PyResult<(Option<&'py PyTuple>, Option<&'py PyDict>)> {
        let args = py.from_borrowed_ptr::<PyTuple>(args);
        let kwargs: ::std::option::Option<&PyDict> = py.from_borrowed_ptr_or_opt(kwargs);
        self.extract_arguments(py, args.iter(), kwargs.map(|dict| dict.iter()), output)
    }

    /// Extracts the `args` and `kwargs` provided into `output`, according to this function
    /// definition.
    ///
    /// `output` must have the same length as this function has positional and keyword-only
    /// parameters (as per the `positional_parameter_names` and `keyword_only_parameters`
    /// respectively).
    ///
    /// If `accept_varargs` or `accept_varkeywords`, then the returned `&PyTuple` and `&PyDict` may
    /// be `Some` if there are extra arguments.
    ///
    /// Unexpected, duplicate or invalid arguments will cause this function to return `TypeError`.
    #[inline]
    fn extract_arguments<'py>(
        &self,
        py: Python<'py>,
        mut args: impl ExactSizeIterator<Item = &'py PyAny>,
        kwargs: Option<impl Iterator<Item = (&'py PyAny, &'py PyAny)>>,
        output: &mut [Option<&'py PyAny>],
    ) -> PyResult<(Option<&'py PyTuple>, Option<&'py PyDict>)> {
        let num_positional_parameters = self.positional_parameter_names.len();

        debug_assert!(self.positional_only_parameters <= num_positional_parameters);
        debug_assert!(self.required_positional_parameters <= num_positional_parameters);
        debug_assert_eq!(
            output.len(),
            num_positional_parameters + self.keyword_only_parameters.len()
        );

        // Handle positional arguments
        let args_provided = {
            let args_provided = args.len();
            if self.accept_varargs {
                std::cmp::min(num_positional_parameters, args_provided)
            } else if args_provided > num_positional_parameters {
                return Err(self.too_many_positional_arguments(args_provided));
            } else {
                args_provided
            }
        };

        // Copy positional arguments into output
        for (out, arg) in output[..args_provided].iter_mut().zip(args.by_ref()) {
            *out = Some(arg);
        }

        // Collect varargs into tuple
        let varargs = if self.accept_varargs {
            Some(PyTuple::new(py, args))
        } else {
            None
        };

        // Handle keyword arguments
        let varkeywords = match (kwargs, self.accept_varkeywords) {
            (Some(kwargs), true) => {
                let mut varkeywords = None;
                self.extract_keyword_arguments(kwargs, output, |name, value| {
                    varkeywords
                        .get_or_insert_with(|| PyDict::new(py))
                        .set_item(name, value)
                })?;
                varkeywords
            }
            (Some(kwargs), false) => {
                self.extract_keyword_arguments(
                    kwargs,
                    output,
                    #[cold]
                    |name, _| Err(self.unexpected_keyword_argument(name)),
                )?;
                None
            }
            (None, _) => None,
        };

        // Check that there's sufficient positional arguments once keyword arguments are specified
        if args_provided < self.required_positional_parameters {
            for out in &output[..self.required_positional_parameters] {
                if out.is_none() {
                    return Err(self.missing_required_positional_arguments(output));
                }
            }
        }

        // Check no missing required keyword arguments
        let keyword_output = &output[num_positional_parameters..];
        for (param, out) in self.keyword_only_parameters.iter().zip(keyword_output) {
            if param.required && out.is_none() {
                return Err(self.missing_required_keyword_arguments(keyword_output));
            }
        }

        Ok((varargs, varkeywords))
    }

    fn extract_keyword_arguments<'py>(
        &self,
        kwargs: impl Iterator<Item = (&'py PyAny, &'py PyAny)>,
        output: &mut [Option<&'py PyAny>],
        mut unexpected_keyword_handler: impl FnMut(&'py PyAny, &'py PyAny) -> PyResult<()>,
    ) -> PyResult<()> {
        let positional_args_count = self.positional_parameter_names.len();
        let mut positional_only_keyword_arguments = Vec::new();
        'for_each_kwarg: for (kwarg_name_py, value) in kwargs {
            let kwarg_name = match kwarg_name_py.downcast::<PyString>()?.to_str() {
                Ok(kwarg_name) => kwarg_name,
                // This keyword is not a UTF8 string: all PyO3 argument names are guaranteed to be
                // UTF8 by construction.
                Err(_) => {
                    unexpected_keyword_handler(kwarg_name_py, value)?;
                    continue;
                }
            };

            // Compare the keyword name against each parameter in turn. This is exactly the same method
            // which CPython uses to map keyword names. Although it's O(num_parameters), the number of
            // parameters is expected to be small so it's not worth constructing a mapping.
            for (i, param) in self.keyword_only_parameters.iter().enumerate() {
                if param.name == kwarg_name {
                    output[positional_args_count + i] = Some(value);
                    continue 'for_each_kwarg;
                }
            }

            // Repeat for positional parameters
            if let Some(i) = self.find_keyword_parameter_in_positionals(kwarg_name) {
                if i < self.positional_only_parameters {
                    positional_only_keyword_arguments.push(kwarg_name);
                } else if output[i].replace(value).is_some() {
                    return Err(self.multiple_values_for_argument(kwarg_name));
                }
                continue;
            }

            unexpected_keyword_handler(kwarg_name_py, value)?;
        }

        if positional_only_keyword_arguments.is_empty() {
            Ok(())
        } else {
            Err(self.positional_only_keyword_arguments(&positional_only_keyword_arguments))
        }
    }

    fn find_keyword_parameter_in_positionals(&self, kwarg_name: &str) -> Option<usize> {
        for (i, param_name) in self.positional_parameter_names.iter().enumerate() {
            if *param_name == kwarg_name {
                return Some(i);
            }
        }
        None
    }

    #[cold]
    fn too_many_positional_arguments(&self, args_provided: usize) -> PyErr {
        let was = if args_provided == 1 { "was" } else { "were" };
        let msg = if self.required_positional_parameters != self.positional_parameter_names.len() {
            format!(
                "{} takes from {} to {} positional arguments but {} {} given",
                self.full_name(),
                self.required_positional_parameters,
                self.positional_parameter_names.len(),
                args_provided,
                was
            )
        } else {
            format!(
                "{} takes {} positional arguments but {} {} given",
                self.full_name(),
                self.positional_parameter_names.len(),
                args_provided,
                was
            )
        };
        PyTypeError::new_err(msg)
    }

    #[cold]
    fn multiple_values_for_argument(&self, argument: &str) -> PyErr {
        PyTypeError::new_err(format!(
            "{} got multiple values for argument '{}'",
            self.full_name(),
            argument
        ))
    }

    #[cold]
    fn unexpected_keyword_argument(&self, argument: &PyAny) -> PyErr {
        PyTypeError::new_err(format!(
            "{} got an unexpected keyword argument '{}'",
            self.full_name(),
            argument
        ))
    }

    #[cold]
    fn positional_only_keyword_arguments(&self, parameter_names: &[&str]) -> PyErr {
        let mut msg = format!(
            "{} got some positional-only arguments passed as keyword arguments: ",
            self.full_name()
        );
        push_parameter_list(&mut msg, parameter_names);
        PyTypeError::new_err(msg)
    }

    #[cold]
    fn missing_required_arguments(&self, argument_type: &str, parameter_names: &[&str]) -> PyErr {
        let arguments = if parameter_names.len() == 1 {
            "argument"
        } else {
            "arguments"
        };
        let mut msg = format!(
            "{} missing {} required {} {}: ",
            self.full_name(),
            parameter_names.len(),
            argument_type,
            arguments,
        );
        push_parameter_list(&mut msg, parameter_names);
        PyTypeError::new_err(msg)
    }

    #[cold]
    fn missing_required_keyword_arguments(&self, keyword_outputs: &[Option<&PyAny>]) -> PyErr {
        debug_assert_eq!(self.keyword_only_parameters.len(), keyword_outputs.len());

        let missing_keyword_only_arguments: Vec<_> = self
            .keyword_only_parameters
            .iter()
            .zip(keyword_outputs)
            .filter_map(|(keyword_desc, out)| {
                if keyword_desc.required && out.is_none() {
                    Some(keyword_desc.name)
                } else {
                    None
                }
            })
            .collect();

        debug_assert!(!missing_keyword_only_arguments.is_empty());
        self.missing_required_arguments("keyword", &missing_keyword_only_arguments)
    }

    #[cold]
    fn missing_required_positional_arguments(&self, output: &[Option<&PyAny>]) -> PyErr {
        let missing_positional_arguments: Vec<_> = self
            .positional_parameter_names
            .iter()
            .take(self.required_positional_parameters)
            .zip(output)
            .filter_map(|(param, out)| if out.is_none() { Some(*param) } else { None })
            .collect();

        debug_assert!(!missing_positional_arguments.is_empty());
        self.missing_required_arguments("positional", &missing_positional_arguments)
    }
}

fn push_parameter_list(msg: &mut String, parameter_names: &[&str]) {
    for (i, parameter) in parameter_names.iter().enumerate() {
        if i != 0 {
            if parameter_names.len() > 2 {
                msg.push(',');
            }

            if i == parameter_names.len() - 1 {
                msg.push_str(" and ")
            } else {
                msg.push(' ')
            }
        }

        msg.push('\'');
        msg.push_str(parameter);
        msg.push('\'');
    }
}

#[cfg(test)]
mod tests {
    use crate::{types::PyTuple, AsPyPointer, PyAny, Python, ToPyObject};

    use super::{push_parameter_list, FunctionDescription};

    #[test]
    fn unexpected_keyword_argument() {
        let function_description = FunctionDescription {
            cls_name: None,
            func_name: "example",
            positional_parameter_names: &[],
            positional_only_parameters: 0,
            required_positional_parameters: 0,
            keyword_only_parameters: &[],
            accept_varargs: false,
            accept_varkeywords: false,
        };

        Python::with_gil(|py| {
            let err = function_description
                .extract_arguments(
                    py,
                    [].iter().copied(),
                    Some(
                        [(
                            "foo".to_object(py).into_ref(py),
                            1u8.to_object(py).into_ref(py),
                        )]
                        .iter()
                        .copied(),
                    ),
                    &mut [],
                )
                .unwrap_err();
            assert_eq!(
                err.to_string(),
                "TypeError: example() got an unexpected keyword argument 'foo'"
            );
        })
    }

    #[test]
    fn keyword_not_string() {
        let function_description = FunctionDescription {
            cls_name: None,
            func_name: "example",
            positional_parameter_names: &[],
            positional_only_parameters: 0,
            required_positional_parameters: 0,
            keyword_only_parameters: &[],
            accept_varargs: false,
            accept_varkeywords: false,
        };

        Python::with_gil(|py| {
            let err = function_description
                .extract_arguments(
                    py,
                    [].iter().copied(),
                    Some(
                        [(
                            1u8.to_object(py).into_ref(py),
                            1u8.to_object(py).into_ref(py),
                        )]
                        .iter()
                        .copied(),
                    ),
                    &mut [],
                )
                .unwrap_err();
            assert_eq!(
                err.to_string(),
                "TypeError: 'int' object cannot be converted to 'PyString'"
            );
        })
    }

    #[test]
    fn missing_required_arguments() {
        let function_description = FunctionDescription {
            cls_name: None,
            func_name: "example",
            positional_parameter_names: &["foo", "bar"],
            positional_only_parameters: 0,
            required_positional_parameters: 2,
            keyword_only_parameters: &[],
            accept_varargs: false,
            accept_varkeywords: false,
        };

        Python::with_gil(|py| {
            let mut output = [None, None];
            let err = unsafe {
                function_description.extract_arguments_tuple_dict(
                    py,
                    PyTuple::new(py, Vec::<&PyAny>::new()).as_ptr(),
                    std::ptr::null_mut(),
                    &mut output,
                )
            }
            .unwrap_err();
            assert_eq!(
                err.to_string(),
                "TypeError: example() missing 2 required positional arguments: 'foo' and 'bar'"
            );
        })
    }

    #[test]
    fn push_parameter_list_empty() {
        let mut s = String::new();
        push_parameter_list(&mut s, &[]);
        assert_eq!(&s, "");
    }

    #[test]
    fn push_parameter_list_one() {
        let mut s = String::new();
        push_parameter_list(&mut s, &["a"]);
        assert_eq!(&s, "'a'");
    }

    #[test]
    fn push_parameter_list_two() {
        let mut s = String::new();
        push_parameter_list(&mut s, &["a", "b"]);
        assert_eq!(&s, "'a' and 'b'");
    }

    #[test]
    fn push_parameter_list_three() {
        let mut s = String::new();
        push_parameter_list(&mut s, &["a", "b", "c"]);
        assert_eq!(&s, "'a', 'b', and 'c'");
    }

    #[test]
    fn push_parameter_list_four() {
        let mut s = String::new();
        push_parameter_list(&mut s, &["a", "b", "c", "d"]);
        assert_eq!(&s, "'a', 'b', 'c', and 'd'");
    }
}
