use std::marker::PhantomData;

use crate::Python;

/// A marker type that makes the type !Send.
/// Workaround for lack of !Send on stable (<https://github.com/rust-lang/rust/issues/68318>).
pub(crate) struct NotSend(PhantomData<*mut Python<'static>>);

#[cfg(feature = "gil-refs")]
pub(crate) const NOT_SEND: NotSend = NotSend(PhantomData);
