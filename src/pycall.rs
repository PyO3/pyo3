mod args;
mod as_pyobject;
mod kwargs;
mod kwargs_args_adapter;
mod storage;
mod trusted_len;

pub mod select_traits {
    pub use super::args::select_traits::*;
    pub use super::kwargs::select_traits::*;
}

use std::mem::MaybeUninit;

pub use args::{ArgsStorageSelector, EmptyArgsStorage};
pub use kwargs::{EmptyKwargsStorage, KwargsStorageSelector};
pub use pyo3_macros::pycall_impl;

use crate::exceptions::PyTypeError;
use crate::ffi_ptr_ext::FfiPtrExt;
use crate::prelude::IntoPyObject;
use crate::types::{PyAnyMethods, PyDict, PyDictMethods, PyString, PyTuple};
use crate::{ffi, Borrowed, Bound, BoundObject, PyAny, PyResult, Python};
use args::{
    AppendEmptyArgForVectorcall, ArgumentsOffsetFlag, ArrayArgsStorage, ConcatArgsStorages,
    ResolveArgs,
};
use kwargs::{ArrayKwargsStorage, ConcatKwargsStorages, KnownKwargsNames};
use kwargs_args_adapter::{CombineArgsKwargs, KwargsArgsAdapter};
use storage::RawStorage;

type PPPyObject = *mut *mut ffi::PyObject;

#[inline(always)]
pub fn concat_args<'py, A, B>(a: A, b: B) -> <A as ConcatArgsStorages<'py, B>>::Output
where
    A: ConcatArgsStorages<'py, B>,
    A: args::FundamentalStorage<'py>,
    B: args::FundamentalStorage<'py>,
{
    A::concat(a, b)
}

#[inline(always)]
pub fn concat_kwargs<'py, A, B>(a: A, b: B) -> <A as ConcatKwargsStorages<'py, B>>::Output
where
    A: ConcatKwargsStorages<'py, B>,
    A: kwargs::FundamentalStorage<'py>,
    B: kwargs::FundamentalStorage<'py>,
{
    A::concat(a, b)
}

#[inline(always)]
pub fn non_unpacked_args<'py, T: args::Tuple<'py>>(tuple: T) -> ArrayArgsStorage<T> {
    ArrayArgsStorage(tuple)
}

#[inline(always)]
pub fn non_unpacked_kwargs<'py, T: kwargs::Tuple<'py>>(tuple: T) -> ArrayKwargsStorage<T> {
    ArrayKwargsStorage(tuple)
}

#[inline(always)]
pub fn first_known_kwarg<'py, Kwarg: IntoPyObject<'py>>(
    kwarg: Kwarg,
) -> kwargs::TypeLevelPyObjectListCons<Kwarg, kwargs::TypeLevelPyObjectListNil> {
    kwargs::TypeLevelPyObjectListCons(kwarg, kwargs::TypeLevelPyObjectListNil)
}

#[inline(always)]
pub fn add_known_kwarg<
    'py,
    Kwarg: IntoPyObject<'py>,
    T: kwargs::TypeLevelPyObjectListTrait<'py>,
>(
    kwarg: Kwarg,
    existing_known: T,
) -> kwargs::TypeLevelPyObjectListCons<Kwarg, T> {
    kwargs::TypeLevelPyObjectListCons(kwarg, existing_known)
}

#[inline(always)]
pub fn known_kwargs_with_names<'py, Values: kwargs::TypeLevelPyObjectListTrait<'py>>(
    names: &'static KnownKwargsNames,
    values: Values,
) -> kwargs::KnownKwargsStorage<'py, Values> {
    kwargs::KnownKwargsStorage {
        names: names
            .0
            .bind_borrowed(unsafe { Python::assume_gil_acquired() }),
        values,
    }
}

/// Call any Python callable, with a syntax similar to Python and maximum performance.
///
/// The syntax for calling a callable is:
/// ```
/// pycall!(
///     callable(
///         arg1, arg2, (*)unpack_args1, arg3, (*)unpack_args2,
///         (**)unpack_kwargs1, kwarg1=value, kwarg2=value, (**)unpack_kwargs2,
///         (kwarg_name_expression)=value,
///     )
/// )
/// ```
/// An argument can be any expression that implements [`IntoPyObject`].
///
/// An unpacked argument can be any `IntoIterator` whose `Item` implements `IntoPyObject`,
/// or any Python iterable.
///
/// A keyword argument's name can be either an identifier, or an expression surrounded in parentheses
/// that produces any type that implements `IntoPyObject<Target = PyString>`. Its value can be
/// of any type that implements `IntoPyObject`.
///
/// An unpacked keyword argument can be either any `IntoIterator<Item = (K, V)>` where `K` implements
/// `IntoPyObject<Target = PyString>` and `V` implements `IntoPyObject`, or any Python mapping.
///
/// `callable` can either be an identifier or an expression surrounded in parentheses that produces
/// [`Bound`] or [`Borrowed`] or any reference to them.
///
/// Similarly, a method can be called:
/// ```
/// pycall!(object.method(...))
/// pycall!(object.(method_name_expression)(...))
/// ```
///
/// `object` can be any [`Bound`] or [`Borrowed`] or a reference to them. It can either be a single identifier
/// or an expression surrounded in parentheses (similar to `callable`).
///
/// The method name after the dot
/// can either be an identifier, in which case it is treated as the name of the method to call,
/// or an expression surrounded in parenthesis that produces a type that implements `IntoPyObject<Target = PyString>`,
/// in which case its value is the method name.
///
/// Note, `object.method()` and `object.(method)()` are **not** the same thing! The former calls the method named
/// `"method"` on `object`, while the latter calls the method whose name is stored in `method` on `object`.
///
/// Method arguments rules are identical to non-method.
///
/// The call returns <code>[PyResult]&lt;[PyAny]&gt;</code>.
///
/// The macro will try its best to pick the most performant way to call the function.
#[macro_export]
macro_rules! pycall {
    ( $($t:tt)* ) => { $crate::pycall::pycall_impl!($crate $($t)*) };
}

pub trait BoundPyObject<'py> {
    fn py(&self) -> Python<'py>;
    fn as_borrowed(&self) -> Borrowed<'_, 'py, PyAny>;
}

impl<'py, T> BoundPyObject<'py> for Bound<'py, T> {
    #[inline(always)]
    fn py(&self) -> Python<'py> {
        self.py()
    }
    #[inline(always)]
    fn as_borrowed(&self) -> Borrowed<'_, 'py, PyAny> {
        self.as_borrowed().into_any()
    }
}

impl<'py, T> BoundPyObject<'py> for Borrowed<'_, 'py, T> {
    #[inline(always)]
    fn py(&self) -> Python<'py> {
        (**self).py()
    }
    #[inline(always)]
    fn as_borrowed(&self) -> Borrowed<'_, 'py, PyAny> {
        self.into_any()
    }
}

impl<'py, T: ?Sized + BoundPyObject<'py>> BoundPyObject<'py> for &'_ T {
    #[inline(always)]
    fn py(&self) -> Python<'py> {
        T::py(self)
    }
    #[inline(always)]
    fn as_borrowed(&self) -> Borrowed<'_, 'py, PyAny> {
        T::as_borrowed(*self)
    }
}

impl<'py, T: ?Sized + BoundPyObject<'py>> BoundPyObject<'py> for &'_ mut T {
    #[inline(always)]
    fn py(&self) -> Python<'py> {
        T::py(self)
    }
    #[inline(always)]
    fn as_borrowed(&self) -> Borrowed<'_, 'py, PyAny> {
        T::as_borrowed(*self)
    }
}

type AppendVectorcallOffset<'py, Args> =
    <AppendEmptyArgForVectorcall as ConcatArgsStorages<'py, Args>>::Output;

#[inline(always)]
fn kwargs_to_dict<'py, Kwargs>(
    py: Python<'py>,
    kwargs: Kwargs,
    kwargs_can_be_cheaply_converted_to_pydict: bool,
) -> PyResult<Bound<'py, PyDict>>
where
    Kwargs: kwargs::ResolveKwargs<'py>,
{
    if kwargs_can_be_cheaply_converted_to_pydict {
        return kwargs.into_pydict(py);
    }
    let kwargs_dict = PyDict::new(py);
    let expected_len = kwargs.write_to_dict(kwargs_dict.as_borrowed())?;
    // Python doesn't allow us to check efficiently if `PyDict_SetItem()` overwrote
    // an existing value, so we check the length instead.
    if kwargs_dict.len() != expected_len {
        return Err(PyTypeError::new_err(
            intern!(py, "got multiple values for keyword argument")
                .clone()
                .unbind(),
        ));
    }
    Ok(kwargs_dict)
}

#[inline(always)]
unsafe fn call_tuple_dict<'py, Args>(
    py: Python<'py>,
    args: Args,
    kwargs: *mut ffi::PyObject,
    do_call: impl FnOnce(*mut ffi::PyObject, *mut ffi::PyObject) -> *mut ffi::PyObject,
) -> PyResult<*mut ffi::PyObject>
where
    Args: args::FundamentalStorage<'py>,
{
    if let Some(args_tuple) = args.as_pytuple(py) {
        return Ok(do_call(args_tuple.as_ptr(), kwargs));
    }

    let len = args.len();
    let tuple = if args.has_known_size() {
        let tuple = ffi::PyTuple_New(
            len.try_into()
                .expect("too many arguments requested for a call"),
        )
        .assume_owned_or_err(py)?
        .downcast_into_unchecked();
        args.write_to_tuple(tuple.as_borrowed(), &mut 0)?;
        tuple
    } else {
        let mut storage = Args::RawStorage::new(len);
        let mut base_storage = storage.as_ptr();
        // DO NOT remove the `as *mut PPPyObject`, due to a rustc bug without it you have aliasing violations.
        let guard = args.init(
            py,
            storage.as_init_param(),
            &mut base_storage as *mut PPPyObject as *const PPPyObject,
        )?;
        let tuple = ffi::PyTuple_New(storage.len() as ffi::Py_ssize_t)
            .assume_owned_or_err(py)?
            .downcast_into_unchecked();
        Args::write_initialized_to_tuple(tuple.as_borrowed(), guard, &mut storage.as_ptr(), &mut 0);
        tuple
    };
    Ok(do_call(tuple.as_ptr(), kwargs))
}

const MAX_STACK_LEN: usize = 11;

#[inline(always)]
unsafe fn call_vectorcall_with_kwargs_names<'py, Args>(
    py: Python<'py>,
    all_args: Args,
    kwargs_names: *mut ffi::PyObject,
    kwargs_len: usize,
    do_call: impl FnOnce(PPPyObject, usize, *mut ffi::PyObject) -> *mut ffi::PyObject,
) -> PyResult<*mut ffi::PyObject>
where
    Args: args::FundamentalStorage<'py>,
{
    if Args::USE_STACK_FOR_SMALL_LEN && all_args.has_known_size() && all_args.len() <= MAX_STACK_LEN
    {
        let mut storage = MaybeUninit::<[*mut ffi::PyObject; MAX_STACK_LEN]>::uninit();
        let mut base_storage = storage.as_mut_ptr().cast::<*mut ffi::PyObject>();
        let positional_len = all_args.len() - kwargs_len;
        let _guard = all_args.init(
            py,
            Args::RawStorage::init_param_from_ptr(base_storage),
            &mut base_storage as *mut PPPyObject as *const PPPyObject,
        )?;
        Ok(do_call(base_storage, positional_len, kwargs_names))
    } else {
        let mut storage = Args::RawStorage::new(all_args.len());
        let mut base_storage = storage.as_ptr();
        let _guard = all_args.init(
            py,
            storage.as_init_param(),
            &mut base_storage as *mut PPPyObject as *const PPPyObject,
        )?;
        let positional_len = storage.len() - kwargs_len;
        Ok(do_call(storage.as_ptr(), positional_len, kwargs_names))
    }
}

#[inline(always)]
unsafe fn call_vectorcall<'py, Args, Kwargs>(
    py: Python<'py>,
    args: Args,
    kwargs: Kwargs,
    do_call: impl FnOnce(PPPyObject, usize, *mut ffi::PyObject) -> *mut ffi::PyObject,
) -> PyResult<*mut ffi::PyObject>
where
    Args: args::FundamentalStorage<'py>,
    Kwargs: kwargs::FundamentalStorage<'py>,
    Args: for<'a> CombineArgsKwargs<'a, 'py, Kwargs>,
{
    debug_assert!(kwargs.has_known_size());
    if Kwargs::IS_EMPTY {
        return call_vectorcall_with_kwargs_names(py, args, std::ptr::null_mut(), 0, do_call);
    }
    let kwargs_len = kwargs.len();
    if let Some(kwargs_names) = kwargs.as_names_pytuple() {
        let all_args = Args::combine_no_names(args, kwargs);
        return call_vectorcall_with_kwargs_names(
            py,
            all_args,
            kwargs_names.as_ptr(),
            kwargs_len,
            do_call,
        );
    }
    // This will be filled inside `call_vectorcall_with_kwargs_names()`, when we initialize the storage.
    let kwargs_names = ffi::PyTuple_New(
        kwargs
            .len()
            .try_into()
            .expect("too many arguments requested for a call"),
    )
    .assume_owned_or_err(py)?
    .downcast_into_unchecked::<PyTuple>();
    let all_args = Args::combine(
        args,
        KwargsArgsAdapter {
            kwargs,
            kwargs_tuple: kwargs_names.as_borrowed(),
        },
    );
    call_vectorcall_with_kwargs_names(py, all_args, kwargs_names.as_ptr(), kwargs_len, do_call)
}

#[inline]
pub fn call<'py, F, Args, Kwargs>(f: F, args: Args, kwargs: Kwargs) -> PyResult<Bound<'py, PyAny>>
where
    F: BoundPyObject<'py>,
    Args: args::FundamentalStorage<'py>,
    Kwargs: kwargs::FundamentalStorage<'py>,
    Args: for<'a> CombineArgsKwargs<'a, 'py, Kwargs>,
    AppendEmptyArgForVectorcall: ConcatArgsStorages<'py, Args>,
    AppendVectorcallOffset<'py, Args>: for<'a> CombineArgsKwargs<'a, 'py, Kwargs>,
{
    // Assertions for extra safety, not required.
    if Args::IS_EMPTY {
        debug_assert_eq!(args.len(), 0);
    }
    if Args::IS_ONE {
        debug_assert_eq!(args.len(), 1);
    }
    if Kwargs::IS_EMPTY {
        debug_assert_eq!(kwargs.len(), 0);
    }

    let py = f.py();
    let f = f.as_borrowed().as_ptr();
    unsafe {
        let result = 'result: {
            if Args::IS_EMPTY && Kwargs::IS_EMPTY {
                break 'result ffi::PyObject_CallNoArgs(f);
            }

            if Args::IS_ONE && Kwargs::IS_EMPTY {
                let mut storage = Args::RawStorage::new(1);
                let mut base_storage = storage.as_ptr();
                let _guard = args.init(
                    py,
                    storage.as_init_param(),
                    &mut base_storage as *mut PPPyObject as *const PPPyObject,
                );
                break 'result ffi::PyObject_CallOneArg(f, *storage.as_ptr());
            }

            let kwargs_can_be_cheaply_converted_to_pydict =
                kwargs.can_be_cheaply_converted_to_pydict(py);

            // The arguments are readily available as a tuple; do not spend time trying to figure out vectorcall -
            // just pass them directly with normal calling convention.
            if let Some(tuple_args) = args.as_pytuple(py) {
                if Kwargs::IS_EMPTY {
                    break 'result ffi::PyObject_Call(f, tuple_args.as_ptr(), std::ptr::null_mut());
                }
                if kwargs_can_be_cheaply_converted_to_pydict {
                    let kwargs_dict = kwargs.into_pydict(py)?;
                    break 'result ffi::PyObject_Call(f, tuple_args.as_ptr(), kwargs_dict.as_ptr());
                }
            }

            if Args::IS_EMPTY && kwargs_can_be_cheaply_converted_to_pydict {
                let kwargs_dict = kwargs.into_pydict(py)?;
                break 'result ffi::PyObject_Call(f, ffi::PyTuple_New(0), kwargs_dict.as_ptr());
            }

            if !kwargs.has_known_size() {
                let kwargs_dict =
                    kwargs_to_dict(py, kwargs, kwargs_can_be_cheaply_converted_to_pydict)?;
                break 'result call_tuple_dict(py, args, kwargs_dict.as_ptr(), |args, kwargs| {
                    ffi::PyObject_Call(f, args, kwargs)
                })?;
            }

            let vectorcall_fn = ffi::PyVectorcall_Function(f);
            match vectorcall_fn {
                Some(vectorcall_fn) => {
                    type CombinedArgsKwargs<'a, 'py, Args, Kwargs> =
                        <Args as CombineArgsKwargs<'a, 'py, Kwargs>>::Output;

                    match CombinedArgsKwargs::<'_, 'py, Args, Kwargs>::ARGUMENTS_OFFSET {
                        ArgumentsOffsetFlag::Normal => {
                            call_vectorcall(
                                py,
                                concat_args(AppendEmptyArgForVectorcall, args),
                                kwargs,
                                |args, args_len, kwargs_names| {
                                    // Add 1 to the arguments pointer and subtract 1 from the length because of `PY_VECTORCALL_ARGUMENTS_OFFSET`.
                                    vectorcall_fn(
                                        f,
                                        args.add(1),
                                        args_len - 1 + ffi::PY_VECTORCALL_ARGUMENTS_OFFSET,
                                        kwargs_names,
                                    )
                                },
                            )?
                        }
                        ArgumentsOffsetFlag::DoNotOffset
                        | ArgumentsOffsetFlag::DoNotOffsetButCanChangeArgs0 => {
                            call_vectorcall(py, args, kwargs, |args, args_len, kwargs_names| {
                                vectorcall_fn(f, args, args_len, kwargs_names)
                            })?
                        }
                    }
                }
                None => {
                    // vectorcall is not available; instead of spending time converting the arguments,
                    // when Python will convert them then again to a tuple, just create a tuple directly.
                    if Kwargs::IS_EMPTY {
                        break 'result call_tuple_dict(
                            py,
                            args,
                            std::ptr::null_mut(),
                            |args, kwargs| ffi::PyObject_Call(f, args, kwargs),
                        )?;
                    } else {
                        let kwargs_dict =
                            kwargs_to_dict(py, kwargs, kwargs_can_be_cheaply_converted_to_pydict)?;
                        break 'result call_tuple_dict(
                            py,
                            args,
                            kwargs_dict.as_ptr(),
                            |args, kwargs| ffi::PyObject_Call(f, args, kwargs),
                        )?;
                    }
                }
            }
        };
        result.assume_owned_or_err(py)
    }
}

type AppendMethodReceiver<'a, 'py, Args> =
    <ArrayArgsStorage<(Borrowed<'a, 'py, PyAny>,)> as ConcatArgsStorages<'py, Args>>::Output;

#[inline]
pub fn call_method<'a, 'py, Obj, Name, Args, Kwargs>(
    obj: &'a Obj,
    method_name: Name,
    args: Args,
    kwargs: Kwargs,
) -> PyResult<Bound<'py, PyAny>>
where
    Obj: BoundPyObject<'py>,
    Name: IntoPyObject<'py, Target = PyString>,
    Args: args::FundamentalStorage<'py>,
    Kwargs: kwargs::FundamentalStorage<'py>,
    ArrayArgsStorage<(Borrowed<'a, 'py, PyAny>,)>: ConcatArgsStorages<'py, Args>,
    AppendMethodReceiver<'a, 'py, Args>: for<'b> CombineArgsKwargs<'b, 'py, Kwargs>,
{
    // Assertions for extra safety, not required.
    if Args::IS_EMPTY {
        debug_assert_eq!(args.len(), 0);
    }
    if Args::IS_ONE {
        debug_assert_eq!(args.len(), 1);
    }
    if Kwargs::IS_EMPTY {
        debug_assert_eq!(kwargs.len(), 0);
    }

    let py = obj.py();
    let obj = obj.as_borrowed();
    let method_name = method_name.into_pyobject(py).map_err(Into::into)?;
    let method_name = method_name.as_borrowed();
    unsafe {
        let result = 'result: {
            if Args::IS_EMPTY && Kwargs::IS_EMPTY {
                break 'result ffi::PyObject_CallMethodNoArgs(obj.as_ptr(), method_name.as_ptr());
            }

            if Args::IS_ONE && Kwargs::IS_EMPTY {
                let mut storage = Args::RawStorage::new(1);
                let mut base_storage = storage.as_ptr();
                let _guard = args.init(
                    py,
                    storage.as_init_param(),
                    &mut base_storage as *mut PPPyObject as *const PPPyObject,
                );
                break 'result ffi::PyObject_CallMethodOneArg(
                    obj.as_ptr(),
                    method_name.as_ptr(),
                    *storage.as_ptr(),
                );
            }

            let kwargs_can_be_cheaply_converted_to_pydict =
                kwargs.can_be_cheaply_converted_to_pydict(py);

            // The arguments are readily available as a tuple; do not spend time trying to figure out vectorcall -
            // just pass them directly with normal calling convention.
            if let Some(tuple_args) = args.as_pytuple(py) {
                // FIXME: Benchmark if this is faster than vectorcall.
                if Kwargs::IS_EMPTY {
                    let method = obj.getattr(method_name)?;
                    break 'result ffi::PyObject_Call(
                        method.as_ptr(),
                        tuple_args.as_ptr(),
                        std::ptr::null_mut(),
                    );
                }
                if kwargs_can_be_cheaply_converted_to_pydict {
                    let method = obj.getattr(method_name)?;
                    let kwargs_dict = kwargs.into_pydict(py)?;
                    break 'result ffi::PyObject_Call(
                        method.as_ptr(),
                        tuple_args.as_ptr(),
                        kwargs_dict.as_ptr(),
                    );
                }
            }

            if Args::IS_EMPTY && kwargs_can_be_cheaply_converted_to_pydict {
                let method = obj.getattr(method_name)?;
                let kwargs_dict = kwargs.into_pydict(py)?;
                break 'result ffi::PyObject_Call(
                    method.as_ptr(),
                    ffi::PyTuple_New(0),
                    kwargs_dict.as_ptr(),
                );
            }

            if !kwargs.has_known_size() {
                let method = obj.getattr(method_name)?;
                let kwargs_dict =
                    kwargs_to_dict(py, kwargs, kwargs_can_be_cheaply_converted_to_pydict)?;
                break 'result call_tuple_dict(py, args, kwargs_dict.as_ptr(), |args, kwargs| {
                    ffi::PyObject_Call(method.as_ptr(), args, kwargs)
                })?;
            }

            type CombinedArgsKwargs<'a, 'b, 'py, Args, Kwargs> =
                <AppendMethodReceiver<'a, 'py, Args> as CombineArgsKwargs<'b, 'py, Kwargs>>::Output;

            match CombinedArgsKwargs::<'_, '_, 'py, Args, Kwargs>::ARGUMENTS_OFFSET {
                ArgumentsOffsetFlag::Normal | ArgumentsOffsetFlag::DoNotOffsetButCanChangeArgs0 => {
                    call_vectorcall(
                        py,
                        concat_args(ArrayArgsStorage((obj,)), args),
                        kwargs,
                        |args, args_len, kwargs_names| {
                            ffi::PyObject_VectorcallMethod(
                                method_name.as_ptr(),
                                args,
                                args_len + ffi::PY_VECTORCALL_ARGUMENTS_OFFSET,
                                kwargs_names,
                            )
                        },
                    )?
                }
                ArgumentsOffsetFlag::DoNotOffset => {
                    unreachable!("since we concatenate the receiver this is unreachable")
                    // call_vectorcall(
                    //     py,
                    //     concat_args(ArrayArgsStorage((obj,)), args),
                    //     kwargs,
                    //     |args, args_len, kwargs_names| {
                    //         ffi::PyObject_VectorcallMethod(
                    //             method_name.as_ptr(),
                    //             args,
                    //             args_len,
                    //             kwargs_names,
                    //         )
                    //     },
                    // )?
                }
            }
        };
        result.assume_owned_or_err(py)
    }
}

// TODO: An option to call a method with a list of arguments that includes the receiver
// (as a first argument), that can be more efficient in case there is already a slice
// of pyobjects including the receiver.

#[cfg(test)]
mod tests {
    use crate::types::{PyAnyMethods, PyDict, PyDictMethods, PyModule, PyTuple};
    use crate::{PyResult, Python, ToPyObject};

    #[test]
    pub fn my_special_test() -> PyResult<()> {
        Python::with_gil(|py| {
            let unpack_args = (1, 2, 3);
            let unpack_args2 = ["a", "b", "c"];
            let m = PyModule::from_code(
                py,
                cr#"
def f(*args, **kwargs): print(args, kwargs)
class mydict(dict): pass
        "#,
                c"my_module.py",
                c"my_module",
            )?;
            let f = m.getattr("f")?;
            let unpack_kwargs = PyDict::new(py);
            unpack_kwargs.set_item("1", "hello")?;
            pycall!(f(1, 2, 3, (*)unpack_args, 5. + 1.2, (*)unpack_args2, a="b", ("c")="d", (**)unpack_kwargs, d=1, e=2))?;
            pycall!(f((*)(1, 2, 3)))?;
            pycall!(f(1, "a",))?;
            pycall!(f(a = 1))?;
            let my_dict = pycall!(m.mydict(a = 1, b = 2, c = "abc"))?.downcast_into::<PyDict>()?;
            pycall!(f((*)PyTuple::new(py, [1, 2, 3]), (**)&my_dict))?;
            pycall!(f((*)[1.to_object(py), 2.to_object(py), 3.to_object(py)]))?;
            pycall!(f((**)std::env::vars().filter(|(name, _)| name.starts_with(char::is_lowercase))))?;

            dbg!(&my_dict, my_dict.get_type());
            Ok(())
        })
    }
}
