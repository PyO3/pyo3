# Experimental: Objects

Practical differences:
 - No longer possible to extract `HashMap<&str, &str>` (e.g.) from a PyDict - these strings cannot be guaranteed to be safe to borrow without pyo3 owned references. Instead you should use `HashMap<String, String>`.
 - Iterators from PyOwned must now be prefixed with &* - e.g. `in set` -> `in &*set`
 - return values `&'py PyAny` -> `PyOwned<'py, Any>`
 - Distinction between _types_ `Any` and _objects_ `PyAny`.
