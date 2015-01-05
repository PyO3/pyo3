pub use self::object::PyObject;
pub use self::typeobject::PyType;
pub use self::module::PyModule;

macro_rules! pythonobject_newtype_only_pythonobject(
    ($name: ident) => (
        pub struct $name<'p>(::objects::PyObject<'p>);
        
        impl <'p> ::python::PythonObject<'p> for $name<'p> {
            #[inline]
            fn as_object<'a>(&'a self) -> &'a ::objects::PyObject<'p> {
                &self.0
            }
            
            #[inline]
            unsafe fn unchecked_downcast_from<'a>(obj: &'a ::objects::PyObject<'p>) -> &'a $name<'p> {
                ::std::mem::transmute(obj)
            }
        }
    )
);

macro_rules! pyobject_newtype(
    ($name: ident, $checkfunction: ident, $typeobject: ident) => (
        pythonobject_newtype_only_pythonobject!($name);
        
        impl <'p> ::python::PythonObjectWithCheckedDowncast<'p> for $name<'p> {
            #[inline]
            fn downcast_from<'a>(obj : &'a ::objects::PyObject<'p>) -> Option<&'a $name<'p>> {
                unsafe {
                    if ::ffi::$checkfunction(::python::PythonObject::as_ptr(obj)) {
                        Some(::python::PythonObject::unchecked_downcast_from(obj))
                    } else {
                        None
                    }
                }
            }
        }

        impl <'p> ::python::PythonObjectWithTypeObject<'p> for $name<'p> {
            #[inline]
            fn type_object(py: ::python::Python<'p>, _ : Option<&Self>) -> &'p ::objects::PyType<'p> {
                unsafe { ::objects::PyType::from_type_ptr(py, &mut ffi::$typeobject) }
            }
        }
    )
);

mod object;
mod typeobject;
mod module;
mod dict;

