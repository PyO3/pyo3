use std::marker::PhantomData;

use crate::{conversion::IntoPyObject, Py};

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

probe!(IsOption);

impl<T> IsOption<Option<T>> {
    pub const VALUE: bool = true;
}

probe!(HasNewTextSignature);

impl<T: super::doc::PyClassNewTextSignature> HasNewTextSignature<T> {
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
