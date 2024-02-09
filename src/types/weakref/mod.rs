pub use callableproxy::PyWeakCallableProxy;
pub use proxy::PyWeakProxy;
pub use reference::{PyWeakRef, PyWeakRefMethods};

pub(crate) mod callableproxy;
pub(crate) mod proxy;
pub(crate) mod reference;
