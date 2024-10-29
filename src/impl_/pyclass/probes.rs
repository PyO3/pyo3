use std::marker::PhantomData;

use crate::{conversion::IntoPyObject, Py};
#[allow(deprecated)]
use crate::{IntoPy, ToPyObject};

/// Trait used to combine with zero-sized types to calculate at compile time
/// some property of a type.
///
/// The trick uses the fact that an associated constant has higher priority
/// than a trait constant, so we can use the trait to define the false case.
///
/// The true case is defined in the zero-sized type's impl block, which is
/// gated on some property like trait bound or only being implemented
/// for fixed concrete types.
pub trait Probe {
    const VALUE: bool = false;
}

macro_rules! probe {
    ($name:ident) => {
        pub struct $name<T>(PhantomData<T>);
        impl<T> Probe for $name<T> {}
    };
}

probe!(IsPyT);

impl<T> IsPyT<Py<T>> {
    pub const VALUE: bool = true;
}

probe!(IsToPyObject);

#[allow(deprecated)]
impl<T: ToPyObject> IsToPyObject<T> {
    pub const VALUE: bool = true;
}

probe!(IsIntoPy);

#[allow(deprecated)]
impl<T: IntoPy<crate::PyObject>> IsIntoPy<T> {
    pub const VALUE: bool = true;
}

probe!(IsIntoPyObjectRef);

// Possible clippy beta regression,
// see https://github.com/rust-lang/rust-clippy/issues/13578
#[allow(clippy::extra_unused_lifetimes)]
impl<'a, 'py, T: 'a> IsIntoPyObjectRef<T>
where
    &'a T: IntoPyObject<'py>,
{
    pub const VALUE: bool = true;
}

probe!(IsIntoPyObject);

impl<'py, T> IsIntoPyObject<T>
where
    T: IntoPyObject<'py>,
{
    pub const VALUE: bool = true;
}

probe!(IsSync);

impl<T: Sync> IsSync<T> {
    pub const VALUE: bool = true;
}
