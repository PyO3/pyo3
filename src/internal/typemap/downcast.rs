use core::{
    any::{Any, TypeId},
    fmt,
};

use macros::{make_clone, make_downcast};

/// Methods for downcasting from an `Any` trait object.
///
/// These should only be implemented for types that satisfy:
/// 1. Implements `Any` (including transitively)
///
/// This includes most types, *excluding* ones that have a
/// non static lifetime -- references, `Struct<'a>`'s, etc
pub trait Downcast {
    /// Gets the `TypeId` of `self`.
    ///
    /// If you can't implement this via a naive call to
    /// Self::type_id() you probably shouldn't implement
    /// this trait for your type(s).
    fn type_id(&self) -> TypeId;

    /// Downcast from `Box<Any>` to `Box<T>`, without
    /// checking the type matches.
    ///
    /// ## Safety
    ///
    /// The caller must ensure that `T` matches the trait
    /// object, via external means.
    unsafe fn downcast_unchecked<T: 'static>(self: Box<Self>) -> Box<T>;

    /// Downcast from `&Any` to `&T`, without checking the
    /// type matches.
    ///
    /// ## Safety
    ///
    /// The caller must ensure that `T` matches the trait
    /// object, via external means.
    unsafe fn downcast_ref_unchecked<T: 'static>(&self) -> &T;

    /// Downcast from `&mut Any` to `&mut T`, without
    /// checking the type matches.
    ///
    /// ## Safety
    ///
    /// The caller must ensure that `T` matches the trait
    /// object, via external means.
    unsafe fn downcast_mut_unchecked<T: 'static>(&mut self) -> &mut T;
}

/// A generic conversion of a type to a dyn trait object
pub trait IntoBox<T: ?Sized + Downcast>: Any {
    fn into_box(self) -> Box<T>;
}

/// [`Any`], but with cloning.
///
/// Every type with no non-`'static` references that
/// implements `Clone` implements `CloneAny`.
/// See [`core::any`] for more details on `Any` in general.
pub trait CloneAny: Any + CloneToAny {}
impl<T: Any + Clone> CloneAny for T {}

/// This trait is used for library internals, please ignore
#[doc(hidden)]
pub trait CloneToAny {
    /// Clone `self` into a new `Box<dyn CloneAny>` object.
    fn clone_to_any(&self) -> Box<dyn CloneAny>;
}

impl<T: Any + Clone> CloneToAny for T {
    #[inline]
    fn clone_to_any(&self) -> Box<dyn CloneAny> {
        Box::new(self.clone())
    }
}

/* Any */

make_downcast!(Any);
make_downcast!(Any + Send);
make_downcast!(Any + Send + Sync);

/* CloneAny */

make_downcast!(CloneAny);
make_downcast!(CloneAny + Send);
make_downcast!(CloneAny + Send + Sync);
make_clone!(dyn CloneAny);
make_clone!(dyn CloneAny + Send);
make_clone!(dyn CloneAny + Send + Sync);

mod macros {
    /// Implement `Downcast` for the given $trait
    macro_rules! make_downcast {
        ($any_trait:ident $(+ $auto_traits:ident)*) => {
            impl Downcast for dyn $any_trait $(+ $auto_traits)* {
                #[inline]
                fn type_id(&self) -> TypeId {
                    self.type_id()
                }

                #[inline]
                unsafe fn downcast_ref_unchecked<T: 'static>(&self) -> &T {
                    unsafe { &*(self as *const Self as *const T) }
                }

                #[inline]
                unsafe fn downcast_mut_unchecked<T: 'static>(&mut self) -> &mut T {
                    unsafe { &mut *(self as *mut Self as *mut T) }
                }

                #[inline]
                unsafe fn downcast_unchecked<T: 'static>(self: Box<Self>) -> Box<T> {
                    unsafe { Box::from_raw(Box::into_raw(self) as *mut T) }
                }
            }

            impl<T: $any_trait $(+ $auto_traits)*> IntoBox<dyn $any_trait $(+ $auto_traits)*> for T {
                #[inline]
                fn into_box(self) -> Box<dyn $any_trait $(+ $auto_traits)*> {
                    Box::new(self)
                }
            }
        }
    }

    /// Implement `Clone` for the given $type
    ///
    /// We also implement a naive `Debug` output that prints
    /// the $type name
    macro_rules! make_clone {
        ($t:ty) => {
            impl Clone for Box<$t> {
                #[inline]
                fn clone(&self) -> Box<$t> {
                    let clone: Box<dyn CloneAny> = (**self).clone_to_any();
                    let raw: *mut dyn CloneAny = Box::into_raw(clone);

                    // We can't do a normal ptr cast here as we get a lint about
                    // a future hard
                    // error, `ptr_cast_add_auto_to_object`.
                    //
                    // This issue doesn't apply here, because we don't have any
                    // conditional methods (`CloneAny` always and only
                    // requires `Any`). Alas, we still have to
                    // transmute(), however to avoid the pesky lint
                    //
                    // https://github.com/rust-lang/rust/issues/127323
                    unsafe { Box::from_raw(std::mem::transmute::<*mut dyn CloneAny, *mut _>(raw)) }
                }
            }

            impl fmt::Debug for $t {
                #[inline]
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    f.pad(stringify!($t))
                }
            }
        };
    }

    pub(super) use make_clone;
    pub(super) use make_downcast;
}
