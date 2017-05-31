// Copyright (c) 2017-present PyO3 Project and Contributors

pub use self::object::{PyObject, PyObjectPtr};
pub use self::typeobject::{PyType, PyTypePtr};
pub use self::module::{PyModule, PyModulePtr};
pub use self::string::{PyBytes, PyBytesPtr, PyString, PyStringPtr, PyStringData};
pub use self::iterator::PyIterator;
pub use self::boolobject::{PyBool, PyBoolPtr};
pub use self::bytearray::{PyByteArray, PyByteArrayPtr};
pub use self::tuple::{PyTuple, PyTuplePtr, NoArgs};
pub use self::dict::{PyDict, PyDictPtr};
pub use self::list::{PyList, PyListPtr};
pub use self::num::{PyLong, PyLongPtr, PyFloat, PyFloatPtr};
pub use self::sequence::PySequence;
pub use self::slice::{PySlice, PySlicePtr};
pub use self::set::{PySet, PySetPtr, PyFrozenSet, PyFrozenSetPtr};


#[macro_export]
macro_rules! pyobject_nativetype(
    ($name: ident, $checkfunction: ident, $typeobject: ident) => {

        impl<'p> $crate::typeob::PyTypeInfo for $name<'p> {
            type Type = ();

            #[inline]
            fn size() -> usize {
                $crate::std::mem::size_of::<ffi::PyObject>()
            }

            #[inline]
            fn offset() -> isize {
                0
            }

            #[inline]
            fn type_name() -> &'static str {
                stringify!($name)
            }
            #[inline]
            fn type_object() -> &'static mut $crate::ffi::PyTypeObject {
                unsafe { &mut $crate::ffi::$typeobject }
            }
        }

        pyobject_nativetype!($name, $checkfunction);
    };
    ($name: ident, $checkfunction: ident, $typeobject: ident, $ptr: ident) => {

        impl<'p> $crate::python::Park for $name<'p> {
            type Target = $ptr;

            fn park(self) -> $ptr {
                unsafe {$crate::std::mem::transmute(self)}
            }
        }

        impl<'p> $crate::python::Unpark<'p> for $ptr {
            type Target = $name<'p>;

            fn unpark(self, _py: $crate::Python<'p>) -> $name<'p> {
                unsafe {$crate::std::mem::transmute(self)}
            }
        }

        impl $crate::std::ops::Deref for $ptr {
            type Target = PPyPtr;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl $crate::IntoPyObject for $ptr {
            #[inline]
            fn into_object(self, _py: $crate::Python) -> $crate::PyObjectPtr
            {
                unsafe { $crate::std::mem::transmute(self) }
            }
        }

        impl $crate::python::IntoPythonPointer for $ptr {
            /// Gets the underlying FFI pointer, returns a owned pointer.
            #[inline]
            #[must_use]
            fn into_ptr(self) -> *mut $crate::ffi::PyObject {
                unsafe {$crate::std::mem::transmute(self)}
            }
        }

        impl $crate::python::ToPythonPointer for $ptr {
            /// Gets the underlying FFI pointer, returns a owned pointer.
            #[inline]
            fn as_ptr(&self) -> *mut $crate::ffi::PyObject {
                self.0.as_ptr()
            }
        }

        impl<'p> $crate::python::PyDowncastInto<'p> for $ptr
        {
            fn downcast_into<I>(py: $crate::Python<'p>, ob: I)
                                -> Result<Self, $crate::PyDowncastError<'p>>
                where I: $crate::ToPythonPointer + $crate::IntoPythonPointer
            {
                unsafe{
                    let ptr = ob.into_ptr();
                    if ffi::$checkfunction(ptr) != 0 {
                        Ok($ptr($crate::PPyPtr::from_owned_ptr(ptr)))
                    } else {
                        $crate::ffi::Py_DECREF(ptr);
                        Err($crate::PyDowncastError(py, None))
                    }
                }
            }

            fn downcast_from_owned_ptr(py: $crate::Python<'p>, ptr: *mut $crate::ffi::PyObject)
                                       -> Result<$ptr, $crate::PyDowncastError<'p>>
            {
                unsafe{
                    if ffi::$checkfunction(ptr) != 0 {
                        Ok($ptr($crate::PPyPtr::from_owned_ptr(ptr)))
                    } else {
                        $crate::ffi::Py_DECREF(ptr);
                        Err($crate::PyDowncastError(py, None))
                    }
                }
            }
        }

        impl $crate::std::fmt::Debug for $ptr {
            fn fmt(&self, f : &mut $crate::std::fmt::Formatter)
                   -> Result<(), $crate::std::fmt::Error>
            {
                use objectprotocol::ObjectProtocol;
                let gil = $crate::Python::acquire_gil();
                let py = gil.python();

                // TODO: we shouldn't use fmt::Error when repr() fails
                let r = self.as_object(py);
                let repr_obj = try!(r.repr().map_err(|_| $crate::std::fmt::Error));
                f.write_str(&repr_obj.to_string_lossy())
            }
        }

        impl $crate::std::fmt::Display for $ptr {
            fn fmt(&self, f : &mut $crate::std::fmt::Formatter)
                   -> Result<(), $crate::std::fmt::Error>
            {
                use objectprotocol::ObjectProtocol;

                let gil = Python::acquire_gil();
                let py = gil.python();

                // TODO: we shouldn't use fmt::Error when repr() fails
                let r = self.as_object(py);
                let repr_obj = try!(r.str().map_err(|_| $crate::std::fmt::Error));
                f.write_str(&repr_obj.to_string_lossy())
            }
        }

        pyobject_nativetype!($name, $checkfunction, $typeobject);
    };

    ($name: ident, $checkfunction: ident) => (

        impl<'p> $crate::native::PyBaseObject for $name<'p> {}

        impl<'p> $crate::native::PyNativeObject<'p> for $name<'p> {
            //fn park(self) -> $crate::PyObjectPtr {
            //    unsafe { $crate::std::mem::transmute(self) }
            //}
            fn as_object(self) -> $crate::PyObject<'p> {
                unsafe {
                    $crate::ffi::Py_INCREF(self.as_ptr());
                    $crate::std::mem::transmute(self)
                }
            }
            fn clone_object(&self) -> $name<'p> {
                use $crate::{ToPythonPointer, PythonObjectWithGilToken};
                unsafe {
                    $name(pyptr::from_borrowed_ptr(self.gil(), self.as_ptr()))
                }
            }
        }

        impl<'p> $crate::token::PythonObjectWithGilToken<'p> for $name<'p> {
            fn gil(&self) -> $crate::python::Python<'p> {
                self.0.token()
            }
        }

        impl<'p> $crate::python::PyDowncastFrom<'p> for $name<'p>
        {
            fn downcast_from(py: &'p $crate::PyObject<'p>)
                             -> Result<&'p $name<'p>, $crate::PyDowncastError<'p>>
            {
                use $crate::{ToPythonPointer, PythonObjectWithGilToken};

                unsafe {
                    if $crate::ffi::$checkfunction(py.as_ptr()) > 0 {
                        let ptr = py as *const _ as *mut u8 as *mut $name;
                        Ok(ptr.as_ref().unwrap())
                    } else {
                        Err($crate::PyDowncastError(py.gil(), None))
                    }
                }
            }
        }

        impl<'p> $crate::python::PyDowncastInto<'p> for $name<'p>
        {
            fn downcast_into<I>(py: $crate::Python<'p>, ob: I)
                                -> Result<Self, $crate::PyDowncastError<'p>>
                where I: $crate::ToPythonPointer + $crate::IntoPythonPointer
            {
                unsafe{
                    let ptr = ob.into_ptr();
                    if ffi::$checkfunction(ptr) != 0 {
                        Ok($name(pyptr::from_owned_ptr(py, ptr)))
                    } else {
                        $crate::ffi::Py_DECREF(ptr);
                        Err($crate::PyDowncastError(py, None))
                    }
                }
            }

            fn downcast_from_owned_ptr(py: $crate::Python<'p>, ptr: *mut $crate::ffi::PyObject)
                                       -> Result<$name<'p>, $crate::PyDowncastError<'p>>
            {
                unsafe{
                    if ffi::$checkfunction(ptr) != 0 {
                        Ok($name(pyptr::from_owned_ptr(py, ptr)))
                    } else {
                        $crate::ffi::Py_DECREF(ptr);
                        Err($crate::PyDowncastError(py, None))
                    }
                }
            }
        }

        impl<'p> $crate::python::ToPythonPointer for $name<'p> {
            /// Gets the underlying FFI pointer, returns a borrowed pointer.
            #[inline]
            fn as_ptr(&self) -> *mut $crate::ffi::PyObject {
                self.0.as_ptr()
            }
        }

        impl<'p> $crate::python::IntoPythonPointer for $name<'p> {
            /// Gets the underlying FFI pointer, returns a owned pointer.
            #[inline]
            fn into_ptr(self) -> *mut $crate::ffi::PyObject {
                let ptr = self.0.as_ptr();
                $crate::std::mem::forget(self);
                ptr
            }
        }

        impl<'a> $crate::FromPyObject<'a> for $name<'a>
        {
            /// Extracts `Self` from the source `Py<PyObject>`.
            fn extract(py: &'a $crate::PyObject<'a>) -> $crate::PyResult<Self>
            {
                use $crate::token::PythonObjectWithGilToken;

                unsafe {
                    if ffi::$checkfunction(py.as_ptr()) != 0 {
                        Ok( $name($crate::pyptr::from_borrowed_ptr(py.gil(), py.as_ptr())) )
                    } else {
                        Err(::PyDowncastError(py.gil(), None).into())
                    }
                }
            }
        }

        impl<'a> $crate::FromPyObject<'a> for &'a $name<'a>
        {
            /// Extracts `Self` from the source `PyObject`.
            fn extract(py: &'a $crate::PyObject<'a>) -> $crate::PyResult<Self>
            {
                unsafe {
                    if ffi::$checkfunction(py.as_ptr()) != 0 {
                        Ok($crate::std::mem::transmute(py))
                    } else {
                        Err($crate::PyDowncastError(
                            $crate::token::PythonObjectWithGilToken::gil(py), None).into())
                    }
                }
            }
        }

        impl<'a> $crate::ToPyObject for $name<'a>
        {
            #[inline]
            fn to_object<'p>(&self, py: $crate::Python<'p>) -> $crate::PyObject<'p> {
                $crate::PyObject::from_borrowed_ptr(py, self.0.as_ptr())
            }

            #[inline]
            fn with_borrowed_ptr<F, R>(&self, _py: $crate::Python, f: F) -> R
                where F: FnOnce(*mut ffi::PyObject) -> R
            {
                f(self.0.as_ptr())
            }
        }

        impl<'a> $crate::IntoPyObject for $name<'a>
        {
            #[inline]
            fn into_object(self, _py: $crate::Python) -> $crate::PyObjectPtr
            {
                unsafe { $crate::std::mem::transmute(self) }
            }
        }

        impl<'p> $crate::std::fmt::Debug for $name<'p> {
            fn fmt(&self, f: &mut $crate::std::fmt::Formatter)
                   -> Result<(), $crate::std::fmt::Error>
            {
                use $crate::python::PyDowncastInto;

                let py = <$name as $crate::token::PythonObjectWithGilToken>::gil(self);
                let s = unsafe { $crate::PyString::downcast_from_owned_ptr(
                    py, $crate::ffi::PyObject_Repr(
                        $crate::python::ToPythonPointer::as_ptr(self))) };
                let repr_obj = try!(s.map_err(|_| $crate::std::fmt::Error));
                f.write_str(&repr_obj.to_string_lossy())
            }
        }

        impl<'p> $crate::std::fmt::Display for $name<'p> {
            fn fmt(&self, f: &mut $crate::std::fmt::Formatter)
                   -> Result<(), $crate::std::fmt::Error>
            {
                use $crate::python::PyDowncastInto;

                let py = <$name as $crate::token::PythonObjectWithGilToken>::gil(self);
                let s = unsafe { $crate::PyString::downcast_from_owned_ptr(
                    py, $crate::ffi::PyObject_Str(
                        $crate::python::ToPythonPointer::as_ptr(self))) };
                let str_obj = try!(s.map_err(|_| $crate::std::fmt::Error));
                f.write_str(&str_obj.to_string_lossy())
            }
        }
    );
);


macro_rules! pyobject_extract(
    ($obj:ident to $t:ty => $body: block) => {
        impl<'source> ::conversion::FromPyObject<'source>
            for $t
        {
            fn extract($obj: &'source ::PyObject<'source>) -> $crate::PyResult<Self>
                //where S: ::typeob::PyTypeInfo
            {
                $body
            }
        }
    }
);


mod typeobject;
mod module;
mod string;
mod dict;
mod iterator;
mod boolobject;
mod bytearray;
mod tuple;
mod list;
mod num;
mod sequence;
mod slice;
mod set;
mod object;
pub mod exc;
