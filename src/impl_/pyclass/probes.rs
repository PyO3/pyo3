use std::marker::PhantomData;

use crate::conversion::IntoPyObject;
use crate::impl_::pyclass::PyClassBaseType;
use crate::impl_::pyclass_init::PyNativeTypeInitializer;
use crate::{FromPyObject, Py, PyClass, PyClassInitializer};

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

probe!(IsIntoPyObjectRef);

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

probe!(IsSend);

impl<T: Send> IsSend<T> {
    pub const VALUE: bool = true;
}

probe!(IsSync);

impl<T: Sync> IsSync<T> {
    pub const VALUE: bool = true;
}

probe!(IsFromPyObject);

impl<'a, 'py, T> IsFromPyObject<T>
where
    T: FromPyObject<'a, 'py>,
{
    pub const VALUE: bool = true;
}

probe!(HasNewTextSignature);

impl<T: super::doc::PyClassNewTextSignature> HasNewTextSignature<T> {
    pub const VALUE: bool = true;
}

probe!(IsClone);

impl<T: Clone> IsClone<T> {
    pub const VALUE: bool = true;
}

probe!(IsReturningEmptyTuple);

impl IsReturningEmptyTuple<()> {
    pub const VALUE: bool = true;
}

impl<E> IsReturningEmptyTuple<Result<(), E>> {
    pub const VALUE: bool = true;
}

probe!(IsPyClass);
impl<T> IsPyClass<T>
where
    T: PyClass,
{
    pub const VALUE: bool = true;
}

impl<T, E> IsPyClass<Result<T, E>>
where
    T: PyClass,
{
    pub const VALUE: bool = true;
}

probe!(IsInitializerTuple);
impl<S, B> IsInitializerTuple<(S, B)>
where
    S: PyClass<BaseType = B>,
    B: PyClass + PyClassBaseType<Initializer = PyClassInitializer<B>>,
    B::BaseType: PyClassBaseType<Initializer = PyNativeTypeInitializer<B::BaseType>>,
{
    pub const VALUE: bool = true;
}
impl<S, B, E> IsInitializerTuple<Result<(S, B), E>>
where
    S: PyClass<BaseType = B>,
    B: PyClass + PyClassBaseType<Initializer = PyClassInitializer<B>>,
    B::BaseType: PyClassBaseType<Initializer = PyNativeTypeInitializer<B::BaseType>>,
{
    pub const VALUE: bool = true;
}

#[cfg(test)]
macro_rules! value_of {
    ($probe:ident, $ty:ty) => {{
        #[allow(unused_imports)] // probe trait not used if VALUE is true
        use crate::impl_::pyclass::Probe as _;
        $probe::<$ty>::VALUE
    }};
}

#[cfg(test)]
pub(crate) use value_of;
