use crate::{
    conversion::FromPyObjectBound,
    exceptions::PyTypeError,
    ffi,
    pyclass::boolean_struct::False,
    types::{any::PyAnyMethods, dict::PyDictMethods, tuple::PyTupleMethods, PyDict, PyTuple},
    Borrowed, Bound, PyAny, PyClass, PyClassGuard, PyClassGuardMut, PyErr, PyResult, PyTypeCheck,
    Python,
};

/// Helper type used to keep implementation more concise.
///
/// (Function argument extraction borrows input arguments.)
type PyArg<'py> = Borrowed<'py, 'py, PyAny>;

/// A trait which is used to help PyO3 macros extract function arguments.
///
/// `#[pyclass]` structs need to extract as `PyRef<T>` and `PyRefMut<T>`
/// wrappers rather than extracting `&T` and `&mut T` directly. The `Holder` type is used
/// to hold these temporary wrappers - the way the macro is constructed, these wrappers
/// will be dropped as soon as the pyfunction call ends.
///
/// There exists a trivial blanket implementation for `T: FromPyObject` with `Holder = ()`.
pub trait PyFunctionArgument<'a, 'holder, 'py, const IS_OPTION: bool>: Sized {
    type Holder: FunctionArgumentHolder;

    /// Provides the type hint information for which Python types are allowed.
    #[cfg(feature = "experimental-inspect")]
    const INPUT_TYPE: &'static str;

    fn extract(obj: &'a Bound<'py, PyAny>, holder: &'holder mut Self::Holder) -> PyResult<Self>;
}

impl<'a, 'holder, 'py, T> PyFunctionArgument<'a, 'holder, 'py, false> for T
where
    T: FromPyObjectBound<'a, 'py>,
{
    type Holder = ();

    #[cfg(feature = "experimental-inspect")]
    const INPUT_TYPE: &'static str = T::INPUT_TYPE;

    #[inline]
    fn extract(obj: &'a Bound<'py, PyAny>, _: &'holder mut ()) -> PyResult<Self> {
        obj.extract()
    }
}

impl<'a, 'holder, 'py, T: 'py> PyFunctionArgument<'a, 'holder, 'py, false> for &'a Bound<'py, T>
where
    T: PyTypeCheck,
{
    type Holder = ();

    #[cfg(feature = "experimental-inspect")]
    const INPUT_TYPE: &'static str = T::PYTHON_TYPE;

    #[inline]
    fn extract(obj: &'a Bound<'py, PyAny>, _: &'holder mut ()) -> PyResult<Self> {
        obj.cast().map_err(Into::into)
    }
}

impl<'a, 'holder, 'py, T> PyFunctionArgument<'a, 'holder, 'py, true> for Option<T>
where
    T: PyFunctionArgument<'a, 'holder, 'py, false>, // inner `Option`s will use `FromPyObject`
{
    type Holder = T::Holder;

    #[cfg(feature = "experimental-inspect")]
    const INPUT_TYPE: &'static str = "typing.Any | None";

    #[inline]
    fn extract(obj: &'a Bound<'py, PyAny>, holder: &'holder mut T::Holder) -> PyResult<Self> {
        if obj.is_none() {
            Ok(None)
        } else {
            Ok(Some(T::extract(obj, holder)?))
        }
    }
}

#[cfg(all(Py_LIMITED_API, not(Py_3_10)))]
impl<'a, 'holder> PyFunctionArgument<'a, 'holder, '_, false> for &'holder str {
    type Holder = Option<std::borrow::Cow<'a, str>>;

    #[cfg(feature = "experimental-inspect")]
    const INPUT_TYPE: &'static str = "str";

    #[inline]
    fn extract(
        obj: &'a Bound<'_, PyAny>,
        holder: &'holder mut Option<std::borrow::Cow<'a, str>>,
    ) -> PyResult<Self> {
        Ok(holder.insert(obj.extract()?))
    }
}

/// Trait for types which can be a function argument holder - they should
/// to be able to const-initialize to an empty value.
pub trait FunctionArgumentHolder: Sized {
    const INIT: Self;
}

impl FunctionArgumentHolder for () {
    const INIT: Self = ();
}

impl<T> FunctionArgumentHolder for Option<T> {
    const INIT: Self = None;
}

#[inline]
pub fn extract_pyclass_ref<'a, 'holder, 'py, T: PyClass>(
    obj: &'a Bound<'py, PyAny>,
    holder: &'holder mut Option<PyClassGuard<'a, T>>,
) -> PyResult<&'holder T> {
    Ok(&*holder.insert(PyClassGuard::try_borrow(obj.downcast()?.as_unbound())?))
}

#[inline]
pub fn extract_pyclass_ref_mut<'a, 'holder, 'py, T: PyClass<Frozen = False>>(
    obj: &'a Bound<'py, PyAny>,
    holder: &'holder mut Option<PyClassGuardMut<'a, T>>,
) -> PyResult<&'holder mut T> {
    Ok(&mut *holder.insert(PyClassGuardMut::try_borrow_mut(
        obj.downcast()?.as_unbound(),
    )?))
}

/// The standard implementation of how PyO3 extracts a `#[pyfunction]` or `#[pymethod]` function argument.
#[doc(hidden)]
pub fn extract_argument<'a, 'holder, 'py, T, const IS_OPTION: bool>(
    obj: &'a Bound<'py, PyAny>,
    holder: &'holder mut T::Holder,
    arg_name: &str,
) -> PyResult<T>
where
    T: PyFunctionArgument<'a, 'holder, 'py, IS_OPTION>,
{
    match PyFunctionArgument::extract(obj, holder) {
        Ok(value) => Ok(value),
        Err(e) => Err(argument_extraction_error(obj.py(), arg_name, e)),
    }
}

/// Alternative to [`extract_argument`] used for `Option<T>` arguments. This is necessary because Option<&T>
/// does not implement `PyFunctionArgument` for `T: PyClass`.
#[doc(hidden)]
pub fn extract_optional_argument<'a, 'holder, 'py, T, const IS_OPTION: bool>(
    obj: Option<&'a Bound<'py, PyAny>>,
    holder: &'holder mut T::Holder,
    arg_name: &str,
    default: fn() -> Option<T>,
) -> PyResult<Option<T>>
where
    T: PyFunctionArgument<'a, 'holder, 'py, IS_OPTION>,
{
    match obj {
        Some(obj) => {
            if obj.is_none() {
                // Explicit `None` will result in None being used as the function argument
                Ok(None)
            } else {
                extract_argument(obj, holder, arg_name).map(Some)
            }
        }
        _ => Ok(default()),
    }
}

/// Alternative to [`extract_argument`] used when the argument has a default value provided by an annotation.
#[doc(hidden)]
pub fn extract_argument_with_default<'a, 'holder, 'py, T, const IS_OPTION: bool>(
    obj: Option<&'a Bound<'py, PyAny>>,
    holder: &'holder mut T::Holder,
    arg_name: &str,
    default: fn() -> T,
) -> PyResult<T>
where
    T: PyFunctionArgument<'a, 'holder, 'py, IS_OPTION>,
{
    match obj {
        Some(obj) => extract_argument(obj, holder, arg_name),
        None => Ok(default()),
    }
}

/// Alternative to [`extract_argument`] used when the argument has a `#[pyo3(from_py_with)]` annotation.
#[doc(hidden)]
pub fn from_py_with<'a, 'py, T>(
    obj: &'a Bound<'py, PyAny>,
    arg_name: &str,
    extractor: fn(&'a Bound<'py, PyAny>) -> PyResult<T>,
) -> PyResult<T> {
    match extractor(obj) {
        Ok(value) => Ok(value),
        Err(e) => Err(argument_extraction_error(obj.py(), arg_name, e)),
    }
}

/// Alternative to [`extract_argument`] used when the argument has a `#[pyo3(from_py_with)]` annotation and also a default value.
#[doc(hidden)]
pub fn from_py_with_with_default<'a, 'py, T>(
    obj: Option<&'a Bound<'py, PyAny>>,
    arg_name: &str,
    extractor: fn(&'a Bound<'py, PyAny>) -> PyResult<T>,
    default: fn() -> T,
) -> PyResult<T> {
    match obj {
        Some(obj) => from_py_with(obj, arg_name, extractor),
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
    if error.get_type(py).is(py.get_type::<PyTypeError>()) {
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
pub unsafe fn unwrap_required_argument<'a, 'py>(
    argument: Option<&'a Bound<'py, PyAny>>,
) -> &'a Bound<'py, PyAny> {
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
    /// - `args` must be a pointer to a C-style array of valid `ffi::PyObject` pointers, or NULL.
    /// - `kwnames` must be a pointer to a PyTuple, or NULL.
    /// - `nargs + kwnames.len()` is the total length of the `args` array.
    #[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
    pub unsafe fn extract_arguments_fastcall<'py, V, K>(
        &self,
        py: Python<'py>,
        args: *const *mut ffi::PyObject,
        nargs: ffi::Py_ssize_t,
        kwnames: *mut ffi::PyObject,
        output: &mut [Option<PyArg<'py>>],
    ) -> PyResult<(V::Varargs, K::Varkeywords)>
    where
        V: VarargsHandler<'py>,
        K: VarkeywordsHandler<'py>,
    {
        let num_positional_parameters = self.positional_parameter_names.len();

        debug_assert!(nargs >= 0);
        debug_assert!(self.positional_only_parameters <= num_positional_parameters);
        debug_assert!(self.required_positional_parameters <= num_positional_parameters);
        debug_assert_eq!(
            output.len(),
            num_positional_parameters + self.keyword_only_parameters.len()
        );

        // Handle positional arguments
        // Safety:
        //  - Option<PyArg> has the same memory layout as `*mut ffi::PyObject`
        //  - we both have the GIL and can borrow these input references for the `'py` lifetime.
        let args: *const Option<PyArg<'py>> = args.cast();
        let positional_args_provided = nargs as usize;
        let remaining_positional_args = if args.is_null() {
            debug_assert_eq!(positional_args_provided, 0);
            &[]
        } else {
            // Can consume at most the number of positional parameters in the function definition,
            // the rest are varargs.
            let positional_args_to_consume =
                num_positional_parameters.min(positional_args_provided);
            let (positional_parameters, remaining) = unsafe {
                std::slice::from_raw_parts(args, positional_args_provided)
                    .split_at(positional_args_to_consume)
            };
            output[..positional_args_to_consume].copy_from_slice(positional_parameters);
            remaining
        };
        let varargs = V::handle_varargs_fastcall(py, remaining_positional_args, self)?;

        // Handle keyword arguments
        let mut varkeywords = K::Varkeywords::default();

        // Safety: kwnames is known to be a pointer to a tuple, or null
        //  - we both have the GIL and can borrow this input reference for the `'py` lifetime.
        let kwnames: Option<Borrowed<'_, '_, PyTuple>> = unsafe {
            Borrowed::from_ptr_or_opt(py, kwnames).map(|kwnames| kwnames.cast_unchecked())
        };
        if let Some(kwnames) = kwnames {
            let kwargs = unsafe {
                ::std::slice::from_raw_parts(
                    // Safety: PyArg has the same memory layout as `*mut ffi::PyObject`
                    args.offset(nargs).cast::<PyArg<'py>>(),
                    kwnames.len(),
                )
            };

            self.handle_kwargs::<K, _>(
                kwnames.iter_borrowed().zip(kwargs.iter().copied()),
                &mut varkeywords,
                num_positional_parameters,
                output,
            )?
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
        output: &mut [Option<PyArg<'py>>],
    ) -> PyResult<(V::Varargs, K::Varkeywords)>
    where
        V: VarargsHandler<'py>,
        K: VarkeywordsHandler<'py>,
    {
        // Safety:
        //  - `args` is known to be a tuple
        //  - `kwargs` is known to be a dict or null
        //  - we both have the GIL and can borrow these input references for the `'py` lifetime.
        let args: Borrowed<'py, 'py, PyTuple> =
            unsafe { Borrowed::from_ptr(py, args).cast_unchecked::<PyTuple>() };
        let kwargs: Option<Borrowed<'py, 'py, PyDict>> =
            unsafe { Borrowed::from_ptr_or_opt(py, kwargs).map(|kwargs| kwargs.cast_unchecked()) };

        let num_positional_parameters = self.positional_parameter_names.len();

        debug_assert!(self.positional_only_parameters <= num_positional_parameters);
        debug_assert!(self.required_positional_parameters <= num_positional_parameters);
        debug_assert_eq!(
            output.len(),
            num_positional_parameters + self.keyword_only_parameters.len()
        );

        // Copy positional arguments into output
        for (i, arg) in args
            .iter_borrowed()
            .take(num_positional_parameters)
            .enumerate()
        {
            output[i] = Some(arg);
        }

        // If any arguments remain, push them to varargs (if possible) or error
        let varargs = V::handle_varargs_tuple(&args, self)?;

        // Handle keyword arguments
        let mut varkeywords = K::Varkeywords::default();
        if let Some(kwargs) = kwargs {
            self.handle_kwargs::<K, _>(
                unsafe { kwargs.iter_borrowed() },
                &mut varkeywords,
                num_positional_parameters,
                output,
            )?
        }

        // Once all inputs have been processed, check that all required arguments have been provided.

        self.ensure_no_missing_required_positional_arguments(output, args.len())?;
        self.ensure_no_missing_required_keyword_arguments(output)?;

        Ok((varargs, varkeywords))
    }

    #[inline]
    fn handle_kwargs<'py, K, I>(
        &self,
        kwargs: I,
        varkeywords: &mut K::Varkeywords,
        num_positional_parameters: usize,
        output: &mut [Option<PyArg<'py>>],
    ) -> PyResult<()>
    where
        K: VarkeywordsHandler<'py>,
        I: IntoIterator<Item = (PyArg<'py>, PyArg<'py>)>,
    {
        debug_assert_eq!(
            num_positional_parameters,
            self.positional_parameter_names.len()
        );
        debug_assert_eq!(
            output.len(),
            num_positional_parameters + self.keyword_only_parameters.len()
        );
        let mut positional_only_keyword_arguments = Vec::new();
        for (kwarg_name_py, value) in kwargs {
            // Safety: All keyword arguments should be UTF-8 strings, but if it's not, `.to_str()`
            // will return an error anyway.
            #[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
            let kwarg_name =
                unsafe { kwarg_name_py.cast_unchecked::<crate::types::PyString>() }.to_str();

            #[cfg(all(not(Py_3_10), Py_LIMITED_API))]
            let kwarg_name = kwarg_name_py.extract::<crate::pybacked::PyBackedStr>();

            if let Ok(kwarg_name_owned) = kwarg_name {
                #[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
                let kwarg_name = kwarg_name_owned;
                #[cfg(all(not(Py_3_10), Py_LIMITED_API))]
                let kwarg_name: &str = &kwarg_name_owned;

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
                        // If accepting **kwargs, then it's allowed for the name of the
                        // kwarg to conflict with a postional-only argument - the value
                        // will go into **kwargs anyway.
                        if K::handle_varkeyword(varkeywords, kwarg_name_py, value, self).is_err() {
                            positional_only_keyword_arguments.push(kwarg_name_owned);
                        }
                    } else if output[i].replace(value).is_some() {
                        return Err(self.multiple_values_for_argument(kwarg_name));
                    }
                    continue;
                }
            };

            K::handle_varkeyword(varkeywords, kwarg_name_py, value, self)?
        }

        if !positional_only_keyword_arguments.is_empty() {
            #[cfg(all(not(Py_3_10), Py_LIMITED_API))]
            let positional_only_keyword_arguments: Vec<_> = positional_only_keyword_arguments
                .iter()
                .map(std::ops::Deref::deref)
                .collect();
            return Err(self.positional_only_keyword_arguments(&positional_only_keyword_arguments));
        }

        Ok(())
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
        output: &[Option<PyArg<'_>>],
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
        output: &[Option<PyArg<'_>>],
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
    fn unexpected_keyword_argument(&self, argument: PyArg<'_>) -> PyErr {
        PyTypeError::new_err(format!(
            "{} got an unexpected keyword argument '{}'",
            self.full_name(),
            argument.as_any()
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
    fn missing_required_keyword_arguments(&self, keyword_outputs: &[Option<PyArg<'_>>]) -> PyErr {
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
    fn missing_required_positional_arguments(&self, output: &[Option<PyArg<'_>>]) -> PyErr {
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
        varargs: &[Option<PyArg<'py>>],
        function_description: &FunctionDescription,
    ) -> PyResult<Self::Varargs>;
    /// Called by `FunctionDescription::extract_arguments_tuple_dict` with the original tuple.
    ///
    /// Additional arguments are those in the tuple slice starting from `function_description.positional_parameter_names.len()`.
    fn handle_varargs_tuple(
        args: &Bound<'py, PyTuple>,
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
        varargs: &[Option<PyArg<'py>>],
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
        args: &Bound<'py, PyTuple>,
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
    type Varargs = Bound<'py, PyTuple>;
    #[inline]
    fn handle_varargs_fastcall(
        py: Python<'py>,
        varargs: &[Option<PyArg<'py>>],
        _function_description: &FunctionDescription,
    ) -> PyResult<Self::Varargs> {
        PyTuple::new(py, varargs)
    }

    #[inline]
    fn handle_varargs_tuple(
        args: &Bound<'py, PyTuple>,
        function_description: &FunctionDescription,
    ) -> PyResult<Self::Varargs> {
        let positional_parameters = function_description.positional_parameter_names.len();
        Ok(args.get_slice(positional_parameters, args.len()))
    }
}

/// A trait used to control whether to accept varkeywords in FunctionDescription::extract_argument_(method) functions.
pub trait VarkeywordsHandler<'py> {
    type Varkeywords: Default;
    fn handle_varkeyword(
        varkeywords: &mut Self::Varkeywords,
        name: PyArg<'py>,
        value: PyArg<'py>,
        function_description: &FunctionDescription,
    ) -> PyResult<()>;
}

/// Marker struct which indicates unknown keywords are not permitted.
pub struct NoVarkeywords;

impl<'py> VarkeywordsHandler<'py> for NoVarkeywords {
    type Varkeywords = ();
    #[inline]
    fn handle_varkeyword(
        _varkeywords: &mut Self::Varkeywords,
        name: PyArg<'py>,
        _value: PyArg<'py>,
        function_description: &FunctionDescription,
    ) -> PyResult<()> {
        Err(function_description.unexpected_keyword_argument(name))
    }
}

/// Marker struct which indicates unknown keywords should be collected into a `PyDict`.
pub struct DictVarkeywords;

impl<'py> VarkeywordsHandler<'py> for DictVarkeywords {
    type Varkeywords = Option<Bound<'py, PyDict>>;
    #[inline]
    fn handle_varkeyword(
        varkeywords: &mut Self::Varkeywords,
        name: PyArg<'py>,
        value: PyArg<'py>,
        _function_description: &FunctionDescription,
    ) -> PyResult<()> {
        varkeywords
            .get_or_insert_with(|| PyDict::new(name.py()))
            .set_item(name, value)
    }
}

fn push_parameter_list(msg: &mut String, parameter_names: &[&str]) {
    let len = parameter_names.len();
    for (i, parameter) in parameter_names.iter().enumerate() {
        if i != 0 {
            if len > 2 {
                msg.push(',');
            }

            if i == len - 1 {
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
    use crate::types::{IntoPyDict, PyTuple};
    use crate::Python;

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

        Python::attach(|py| {
            let args = PyTuple::empty(py);
            let kwargs = [("foo", 0u8)].into_py_dict(py).unwrap();
            let err = unsafe {
                function_description
                    .extract_arguments_tuple_dict::<NoVarargs, NoVarkeywords>(
                        py,
                        args.as_ptr(),
                        kwargs.as_ptr(),
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

        Python::attach(|py| {
            let args = PyTuple::empty(py);
            let kwargs = [(1u8, 1u8)].into_py_dict(py).unwrap();
            let err = unsafe {
                function_description
                    .extract_arguments_tuple_dict::<NoVarargs, NoVarkeywords>(
                        py,
                        args.as_ptr(),
                        kwargs.as_ptr(),
                        &mut [],
                    )
                    .unwrap_err()
            };
            assert_eq!(
                err.to_string(),
                "TypeError: example() got an unexpected keyword argument '1'"
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

        Python::attach(|py| {
            let args = PyTuple::empty(py);
            let mut output = [None, None];
            let err = unsafe {
                function_description.extract_arguments_tuple_dict::<NoVarargs, NoVarkeywords>(
                    py,
                    args.as_ptr(),
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
