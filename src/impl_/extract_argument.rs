use crate::{
    exceptions::PyTypeError,
    ffi,
    type_object::PyTypeObject,
    types::{PyDict, PyString, PyTuple},
    FromPyObject, PyAny, PyErr, PyResult, Python,
};

/// The standard implementation of how PyO3 extracts a `#[pyfunction]` or `#[pymethod]` function argument.
#[doc(hidden)]
#[inline]
pub fn extract_argument<'py, T>(obj: &'py PyAny, arg_name: &str) -> PyResult<T>
where
    T: FromPyObject<'py>,
{
    match obj.extract() {
        Ok(value) => Ok(value),
        Err(e) => Err(argument_extraction_error(obj.py(), arg_name, e)),
    }
}

/// Alternative to [`extract_argument`] used for `Option<T>` arguments (because they are implicitly treated
/// as optional if at the end of the positional parameters).
#[doc(hidden)]
#[inline]
pub fn extract_optional_argument<'py, T>(
    obj: Option<&'py PyAny>,
    arg_name: &str,
) -> PyResult<Option<T>>
where
    T: FromPyObject<'py>,
{
    match obj {
        Some(obj) => match obj.extract() {
            Ok(value) => Ok(value),
            Err(e) => Err(argument_extraction_error(obj.py(), arg_name, e)),
        },
        None => Ok(None),
    }
}

/// Alternative to [`extract_argument`] used when the argument has a default value provided by an annotation.
#[doc(hidden)]
#[inline]
pub fn extract_argument_with_default<'py, T>(
    obj: Option<&'py PyAny>,
    arg_name: &str,
    default: impl FnOnce() -> T,
) -> PyResult<T>
where
    T: FromPyObject<'py>,
{
    match obj {
        Some(obj) => match obj.extract() {
            Ok(value) => Ok(value),
            Err(e) => Err(argument_extraction_error(obj.py(), arg_name, e)),
        },
        None => Ok(default()),
    }
}

/// Alternative to [`extract_argument`] used when the argument has a `#[pyo3(from_py_with)]` annotation.
///
/// # Safety
/// - `obj` must not be None (this helper is only used for required function arguments).
#[doc(hidden)]
#[inline]
pub fn from_py_with<'py, T>(
    obj: &'py PyAny,
    arg_name: &str,
    extractor: impl FnOnce(&'py PyAny) -> PyResult<T>,
) -> PyResult<T> {
    // Safety: obj is not None (see safety
    match extractor(obj) {
        Ok(value) => Ok(value),
        Err(e) => Err(argument_extraction_error(obj.py(), arg_name, e)),
    }
}

/// Alternative to [`extract_argument`] used when the argument has a `#[pyo3(from_py_with)]` annotation and also a default value.
#[doc(hidden)]
#[inline]
pub fn from_py_with_with_default<'py, T>(
    obj: Option<&'py PyAny>,
    arg_name: &str,
    extractor: impl FnOnce(&'py PyAny) -> PyResult<T>,
    default: impl FnOnce() -> T,
) -> PyResult<T> {
    match obj {
        Some(obj) => match extractor(obj) {
            Ok(value) => Ok(value),
            Err(e) => Err(argument_extraction_error(obj.py(), arg_name, e)),
        },
        None => Ok(default()),
    }
}

/// Adds the argument name to the error message of an error which occurred during argument extraction.
///
/// Only modifies TypeError. (Cannot guarantee all exceptions have constructors from
/// single string.)
#[doc(hidden)]
#[cold]
pub fn argument_extraction_error(py: Python<'_>, arg_name: &str, error: PyErr) -> PyErr {
    if error.get_type(py).is(PyTypeError::type_object(py)) {
        let remapped_error =
            PyTypeError::new_err(format!("argument '{}': {}", arg_name, error.value(py)));
        remapped_error.set_cause(py, error.cause(py));
        remapped_error
    } else {
        error
    }
}

/// Unwraps the Option<&PyAny> produced by the FunctionDescription `extract_arguments_` methods.
/// They check if required methods are all provided.
///
/// # Safety
/// `argument` must not be `None`
#[doc(hidden)]
#[inline]
pub unsafe fn unwrap_required_argument(argument: Option<&PyAny>) -> &PyAny {
    match argument {
        Some(value) => value,
        #[cfg(debug_assertions)]
        None => unreachable!("required method argument was not extracted"),
        #[cfg(not(debug_assertions))]
        None => std::hint::unreachable_unchecked(),
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
}

impl FunctionDescription {
    fn full_name(&self) -> String {
        if let Some(cls_name) = self.cls_name {
            format!("{}.{}()", cls_name, self.func_name)
        } else {
            format!("{}()", self.func_name)
        }
    }

    /// Equivalent of `extract_arguments_tuple_dict` which uses the Python C-API "fastcall" convention.
    ///
    /// # Safety
    /// - `args` must be a pointer to a C-style array of valid `ffi::PyObject` pointers.
    /// - `kwnames` must be a pointer to a PyTuple, or NULL.
    /// - `nargs + kwnames.len()` is the total length of the `args` array.
    #[cfg(not(Py_LIMITED_API))]
    pub unsafe fn extract_arguments_fastcall<'py, V, K>(
        &self,
        py: Python<'py>,
        args: *const *mut ffi::PyObject,
        nargs: ffi::Py_ssize_t,
        kwnames: *mut ffi::PyObject,
        output: &mut [Option<&'py PyAny>],
    ) -> PyResult<(V::Varargs, K::Varkeywords)>
    where
        V: VarargsHandler<'py>,
        K: VarkeywordsHandler<'py>,
    {
        // Safety: Option<&PyAny> has the same memory layout as `*mut ffi::PyObject`
        let args = args as *const Option<&PyAny>;
        let positional_args_provided = nargs as usize;
        let args_slice = std::slice::from_raw_parts(args, positional_args_provided);

        let num_positional_parameters = self.positional_parameter_names.len();

        debug_assert!(self.positional_only_parameters <= num_positional_parameters);
        debug_assert!(self.required_positional_parameters <= num_positional_parameters);
        debug_assert_eq!(
            output.len(),
            num_positional_parameters + self.keyword_only_parameters.len()
        );

        let varargs = if positional_args_provided > num_positional_parameters {
            let (positional_parameters, varargs) = args_slice.split_at(num_positional_parameters);
            output[..num_positional_parameters].copy_from_slice(positional_parameters);
            V::handle_varargs_fastcall(py, varargs, self)?
        } else {
            output[..positional_args_provided].copy_from_slice(args_slice);
            V::handle_varargs_fastcall(py, &[], self)?
        };

        // Handle keyword arguments
        let mut varkeywords = Default::default();
        if let Some(kwnames) = py.from_borrowed_ptr_or_opt::<PyTuple>(kwnames) {
            let mut positional_only_keyword_arguments = Vec::new();

            // Safety: &PyAny has the same memory layout as `*mut ffi::PyObject`
            let kwargs =
                ::std::slice::from_raw_parts((args as *const &PyAny).offset(nargs), kwnames.len());

            for (kwarg_name_py, &value) in kwnames.iter().zip(kwargs) {
                // All keyword arguments should be UTF8 strings, but we'll check, just in case.
                if let Ok(kwarg_name) = kwarg_name_py.downcast::<PyString>()?.to_str() {
                    // Try to place parameter in keyword only parameters
                    if let Some(i) = self.find_keyword_parameter_in_keyword_only(kwarg_name) {
                        if output[i + num_positional_parameters]
                            .replace(value)
                            .is_some()
                        {
                            return Err(self.multiple_values_for_argument(kwarg_name));
                        }
                        continue;
                    }

                    // Repeat for positional parameters
                    if let Some(i) = self.find_keyword_parameter_in_positional(kwarg_name) {
                        if i < self.positional_only_parameters {
                            positional_only_keyword_arguments.push(kwarg_name);
                        } else if output[i].replace(value).is_some() {
                            return Err(self.multiple_values_for_argument(kwarg_name));
                        }
                        continue;
                    }
                };

                K::handle_unexpected_keyword(&mut varkeywords, kwarg_name_py, value, self)?
            }

            if !positional_only_keyword_arguments.is_empty() {
                return Err(
                    self.positional_only_keyword_arguments(&positional_only_keyword_arguments)
                );
            }
        }

        // Once all inputs have been processed, check that all required arguments have been provided.

        self.ensure_no_missing_required_positional_arguments(output, positional_args_provided)?;
        self.ensure_no_missing_required_keyword_arguments(output)?;

        Ok((varargs, varkeywords))
    }

    /// Extracts the `args` and `kwargs` provided into `output`, according to this function
    /// definition.
    ///
    /// `output` must have the same length as this function has positional and keyword-only
    /// parameters (as per the `positional_parameter_names` and `keyword_only_parameters`
    /// respectively).
    ///
    /// Unexpected, duplicate or invalid arguments will cause this function to return `TypeError`.
    ///
    /// # Safety
    /// - `args` must be a pointer to a PyTuple.
    /// - `kwargs` must be a pointer to a PyDict, or NULL.
    pub unsafe fn extract_arguments_tuple_dict<'py, V, K>(
        &self,
        py: Python<'py>,
        args: *mut ffi::PyObject,
        kwargs: *mut ffi::PyObject,
        output: &mut [Option<&'py PyAny>],
    ) -> PyResult<(V::Varargs, K::Varkeywords)>
    where
        V: VarargsHandler<'py>,
        K: VarkeywordsHandler<'py>,
    {
        let args = py.from_borrowed_ptr::<PyTuple>(args);
        let kwargs: ::std::option::Option<&PyDict> = py.from_borrowed_ptr_or_opt(kwargs);

        let num_positional_parameters = self.positional_parameter_names.len();

        debug_assert!(self.positional_only_parameters <= num_positional_parameters);
        debug_assert!(self.required_positional_parameters <= num_positional_parameters);
        debug_assert_eq!(
            output.len(),
            num_positional_parameters + self.keyword_only_parameters.len()
        );

        // Copy positional arguments into output
        for (i, arg) in args.iter().take(num_positional_parameters).enumerate() {
            output[i] = Some(arg);
        }

        // If any arguments remain, push them to varargs (if possible) or error
        let varargs = V::handle_varargs_tuple(args, self)?;

        // Handle keyword arguments
        let mut varkeywords = Default::default();
        if let Some(kwargs) = kwargs {
            let mut positional_only_keyword_arguments = Vec::new();
            for (kwarg_name_py, value) in kwargs {
                // All keyword arguments should be UTF8 strings, but we'll check, just in case.
                if let Ok(kwarg_name) = kwarg_name_py.downcast::<PyString>()?.to_str() {
                    // Try to place parameter in keyword only parameters
                    if let Some(i) = self.find_keyword_parameter_in_keyword_only(kwarg_name) {
                        if output[i + num_positional_parameters]
                            .replace(value)
                            .is_some()
                        {
                            return Err(self.multiple_values_for_argument(kwarg_name));
                        }
                        continue;
                    }

                    // Repeat for positional parameters
                    if let Some(i) = self.find_keyword_parameter_in_positional(kwarg_name) {
                        if i < self.positional_only_parameters {
                            positional_only_keyword_arguments.push(kwarg_name);
                        } else if output[i].replace(value).is_some() {
                            return Err(self.multiple_values_for_argument(kwarg_name));
                        }
                        continue;
                    }
                };

                K::handle_unexpected_keyword(&mut varkeywords, kwarg_name_py, value, self)?
            }

            if !positional_only_keyword_arguments.is_empty() {
                return Err(
                    self.positional_only_keyword_arguments(&positional_only_keyword_arguments)
                );
            }
        }

        // Once all inputs have been processed, check that all required arguments have been provided.

        self.ensure_no_missing_required_positional_arguments(output, args.len())?;
        self.ensure_no_missing_required_keyword_arguments(output)?;

        Ok((varargs, varkeywords))
    }

    #[inline]
    fn find_keyword_parameter_in_positional(&self, kwarg_name: &str) -> Option<usize> {
        self.positional_parameter_names
            .iter()
            .position(|&param_name| param_name == kwarg_name)
    }

    #[inline]
    fn find_keyword_parameter_in_keyword_only(&self, kwarg_name: &str) -> Option<usize> {
        // Compare the keyword name against each parameter in turn. This is exactly the same method
        // which CPython uses to map keyword names. Although it's O(num_parameters), the number of
        // parameters is expected to be small so it's not worth constructing a mapping.
        self.keyword_only_parameters
            .iter()
            .position(|param_desc| param_desc.name == kwarg_name)
    }

    #[inline]
    fn ensure_no_missing_required_positional_arguments(
        &self,
        output: &[Option<&PyAny>],
        positional_args_provided: usize,
    ) -> PyResult<()> {
        if positional_args_provided < self.required_positional_parameters {
            for out in &output[positional_args_provided..self.required_positional_parameters] {
                if out.is_none() {
                    return Err(self.missing_required_positional_arguments(output));
                }
            }
        }
        Ok(())
    }

    #[inline]
    fn ensure_no_missing_required_keyword_arguments(
        &self,
        output: &[Option<&PyAny>],
    ) -> PyResult<()> {
        let keyword_output = &output[self.positional_parameter_names.len()..];
        for (param, out) in self.keyword_only_parameters.iter().zip(keyword_output) {
            if param.required && out.is_none() {
                return Err(self.missing_required_keyword_arguments(keyword_output));
            }
        }
        Ok(())
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

/// A trait used to control whether to accept varargs in FunctionDescription::extract_argument_(method) functions.
pub trait VarargsHandler<'py> {
    type Varargs;
    /// Called by `FunctionDescription::extract_arguments_fastcall` with any additional arguments.
    fn handle_varargs_fastcall(
        py: Python<'py>,
        varargs: &[Option<&PyAny>],
        function_description: &FunctionDescription,
    ) -> PyResult<Self::Varargs>;
    /// Called by `FunctionDescription::extract_arguments_tuple_dict` with the original tuple.
    ///
    /// Additional arguments are those in the tuple slice starting from `function_description.positional_parameter_names.len()`.
    fn handle_varargs_tuple(
        args: &'py PyTuple,
        function_description: &FunctionDescription,
    ) -> PyResult<Self::Varargs>;
}

/// Marker struct which indicates varargs are not allowed.
pub struct NoVarargs;

impl<'py> VarargsHandler<'py> for NoVarargs {
    type Varargs = ();

    #[inline]
    fn handle_varargs_fastcall(
        _py: Python<'py>,
        varargs: &[Option<&PyAny>],
        function_description: &FunctionDescription,
    ) -> PyResult<Self::Varargs> {
        let extra_arguments = varargs.len();
        if extra_arguments > 0 {
            return Err(function_description.too_many_positional_arguments(
                function_description.positional_parameter_names.len() + extra_arguments,
            ));
        }
        Ok(())
    }

    #[inline]
    fn handle_varargs_tuple(
        args: &'py PyTuple,
        function_description: &FunctionDescription,
    ) -> PyResult<Self::Varargs> {
        let positional_parameter_count = function_description.positional_parameter_names.len();
        let provided_args_count = args.len();
        if provided_args_count <= positional_parameter_count {
            Ok(())
        } else {
            Err(function_description.too_many_positional_arguments(provided_args_count))
        }
    }
}

/// Marker struct which indicates varargs should be collected into a `PyTuple`.
pub struct TupleVarargs;

impl<'py> VarargsHandler<'py> for TupleVarargs {
    type Varargs = &'py PyTuple;
    #[inline]
    fn handle_varargs_fastcall(
        py: Python<'py>,
        varargs: &[Option<&PyAny>],
        _function_description: &FunctionDescription,
    ) -> PyResult<Self::Varargs> {
        Ok(PyTuple::new(py, varargs))
    }

    #[inline]
    fn handle_varargs_tuple(
        args: &'py PyTuple,
        function_description: &FunctionDescription,
    ) -> PyResult<Self::Varargs> {
        let positional_parameters = function_description.positional_parameter_names.len();
        Ok(args.get_slice(positional_parameters, args.len()))
    }
}

/// A trait used to control whether to accept unrecognised keywords in FunctionDescription::extract_argument_(method) functions.
pub trait VarkeywordsHandler<'py> {
    type Varkeywords: Default;
    fn handle_unexpected_keyword(
        varkeywords: &mut Self::Varkeywords,
        name: &'py PyAny,
        value: &'py PyAny,
        function_description: &FunctionDescription,
    ) -> PyResult<()>;
}

/// Marker struct which indicates unknown keywords are not permitted.
pub struct NoVarkeywords;

impl<'py> VarkeywordsHandler<'py> for NoVarkeywords {
    type Varkeywords = ();
    #[inline]
    fn handle_unexpected_keyword(
        _varkeywords: &mut Self::Varkeywords,
        name: &'py PyAny,
        _value: &'py PyAny,
        function_description: &FunctionDescription,
    ) -> PyResult<()> {
        Err(function_description.unexpected_keyword_argument(name))
    }
}

/// Marker struct which indicates unknown keywords should be collected into a `PyDict`.
pub struct DictVarkeywords;

impl<'py> VarkeywordsHandler<'py> for DictVarkeywords {
    type Varkeywords = Option<&'py PyDict>;
    #[inline]
    fn handle_unexpected_keyword(
        varkeywords: &mut Self::Varkeywords,
        name: &'py PyAny,
        value: &'py PyAny,
        _function_description: &FunctionDescription,
    ) -> PyResult<()> {
        varkeywords
            .get_or_insert_with(|| PyDict::new(name.py()))
            .set_item(name, value)
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
    use crate::{
        types::{IntoPyDict, PyTuple},
        AsPyPointer, PyAny, Python, ToPyObject,
    };

    use super::{push_parameter_list, FunctionDescription, NoVarargs, NoVarkeywords};

    #[test]
    fn unexpected_keyword_argument() {
        let function_description = FunctionDescription {
            cls_name: None,
            func_name: "example",
            positional_parameter_names: &[],
            positional_only_parameters: 0,
            required_positional_parameters: 0,
            keyword_only_parameters: &[],
        };

        Python::with_gil(|py| {
            let err = unsafe {
                function_description
                    .extract_arguments_tuple_dict::<NoVarargs, NoVarkeywords>(
                        py,
                        PyTuple::new(py, Vec::<&PyAny>::new()).as_ptr(),
                        [("foo".to_object(py).into_ref(py), 0u8)]
                            .into_py_dict(py)
                            .as_ptr(),
                        &mut [],
                    )
                    .unwrap_err()
            };
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
        };

        Python::with_gil(|py| {
            let err = unsafe {
                function_description
                    .extract_arguments_tuple_dict::<NoVarargs, NoVarkeywords>(
                        py,
                        PyTuple::new(py, Vec::<&PyAny>::new()).as_ptr(),
                        [(1u8.to_object(py).into_ref(py), 1u8)]
                            .into_py_dict(py)
                            .as_ptr(),
                        &mut [],
                    )
                    .unwrap_err()
            };
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
        };

        Python::with_gil(|py| {
            let mut output = [None, None];
            let err = unsafe {
                function_description.extract_arguments_tuple_dict::<NoVarargs, NoVarkeywords>(
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
