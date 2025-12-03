use crate::conversion::IntoPyObject;
use crate::inspect::TypeHint;
use std::marker::PhantomData;
use std::ops::Deref;

/// Trait to guess a function Python return type
///
/// It is useful to properly get the return type `T` when the Rust implementation returns e.g. `PyResult<T>`
pub trait PyReturnType {
    /// The function return type
    const OUTPUT_TYPE: TypeHint;
}

impl<'a, T: IntoPyObject<'a>> PyReturnType for T {
    const OUTPUT_TYPE: TypeHint = T::OUTPUT_TYPE;
}

impl<T: PyReturnType, E> PyReturnType for Result<T, E> {
    const OUTPUT_TYPE: TypeHint = T::OUTPUT_TYPE;
}

/// Hack to guess if the output type is the empty tuple
///
/// Inspiration: <https://github.com/GoldsteinE/gh-blog/blob/master/const_deref_specialization/src/lib.md>
/// TL;DR: with closure we can get the compiler to return us the output type of the usual deref-based specialization
pub const fn is_empty_tuple_from_closure<B: IsEmptyTuple>(
    _closure_returning_bool: &impl FnOnce() -> B,
) -> bool {
    B::VALUE
}

pub trait IsEmptyTuple {
    const VALUE: bool;
}

pub struct IsEmptyTupleFalse;

impl IsEmptyTuple for IsEmptyTupleFalse {
    const VALUE: bool = false;
}

pub struct IsEmptyTupleTrue;

impl IsEmptyTuple for IsEmptyTupleTrue {
    const VALUE: bool = true;
}

pub struct IsEmptyTupleChecker<T> {
    _marker: PhantomData<T>,
}

impl<T> IsEmptyTupleChecker<T> {
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl IsEmptyTupleChecker<()> {
    pub fn check(&self) -> IsEmptyTupleTrue {
        IsEmptyTupleTrue
    }
}

impl<E> IsEmptyTupleChecker<Result<(), E>> {
    pub fn check(&self) -> IsEmptyTupleTrue {
        IsEmptyTupleTrue
    }
}

impl<T> Deref for IsEmptyTupleChecker<T> {
    type Target = IsEmptyTupleCheckerFalse;

    fn deref(&self) -> &Self::Target {
        &IsEmptyTupleCheckerFalse
    }
}

pub struct IsEmptyTupleCheckerFalse;

impl IsEmptyTupleCheckerFalse {
    pub fn check(&self) -> IsEmptyTupleFalse {
        IsEmptyTupleFalse
    }
}

#[repr(C)]
pub struct SerializedIntrospectionFragment<const LEN: usize> {
    pub length: u32,
    pub fragment: [u8; LEN],
}
