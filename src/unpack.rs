use crate::{
    exceptions::PyValueError,
    types::{PyAnyMethods, PyIterator},
    Borrowed, Bound, FromPyObject, PyAny, PyResult,
};

/// TODO
pub trait Unpackable<'py>: Sized {
    /// TODO
    fn unpack(obj: Borrowed<'_, 'py, PyAny>) -> PyResult<Self>;
}

fn get_value<'py, T>(iter: &mut Bound<'py, PyIterator>, expected: usize) -> PyResult<T>
where
    T: for<'a> FromPyObject<'a, 'py>,
{
    let Some(item) = iter.next() else {
        return Err(PyValueError::new_err(format!(
            "not enough values to unpack (expected {expected})",
        )));
    };
    match item?.extract::<T>() {
        Ok(v) => Ok(v),
        Err(e) => return Err(e.into()),
    }
}

fn one<T>() -> usize {
    1
}

macro_rules! tuple_impls {
    ($T:ident $num:literal) => {
        tuple_impls!(@impl $T $num);
    };
    ($T:ident $num:literal $( $U:ident $unum:literal )+) => {
        tuple_impls!($( $U $unum )+);
        tuple_impls!(@impl $T $num $( $U $unum )+);
    };
    (@impl $( $T:ident $num:literal )+) => {
        impl<'py, $($T,)+> Unpackable<'py> for ($($T,)+)
        where
            $($T: for<'a> FromPyObject<'a, 'py>),+
        {
            fn unpack(obj: Borrowed<'_, 'py, PyAny>) -> PyResult<Self> {
                let total = $(one::<$T>() +)+ 0;
                let mut iter = obj.try_iter()?;
                let out = ($(
		    get_value::<$T>(&mut iter, total)?,
                )+);

                if iter.next().is_some() {
                    return Err(PyValueError::new_err(format!(
                        "too many values to unpack (expected {total})"
                    )));
                }

                Ok(out)
            }
        }
    };
}

tuple_impls! {
    T11 11 T10 10 T9 9 T8 8 T7 7 T6 6 T5 5 T4 4 T3 3 T2 2 T1 1 T0 0
}
