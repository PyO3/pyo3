use std::marker::PhantomData;
use std::rc::Rc;

/// A marker type that makes the type !Send.
/// Temporal hack until https://github.com/rust-lang/rust/issues/13231 is resolved.
pub(crate) type Unsendable = PhantomData<Rc<()>>;

pub struct PrivateMarker;

macro_rules! private_decl {
    () => {
        /// This trait is private to implement; this method exists to make it
        /// impossible to implement outside the crate.
        fn __private__(&self) -> crate::internal_tricks::PrivateMarker;
    };
}

macro_rules! private_impl {
    () => {
        #[doc(hidden)]
        fn __private__(&self) -> crate::internal_tricks::PrivateMarker {
            crate::internal_tricks::PrivateMarker
        }
    };
}

macro_rules! pyo3_exception {
    ($name: ident, $base: ty) => {
        $crate::impl_exception_boilerplate!($name);
        $crate::create_exception_type_object!(pyo3_runtime, $name, $base);
    };
}
