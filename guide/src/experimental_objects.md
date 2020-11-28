# Experimental: Objects

Practical differences:
 - No longer possible to extract `HashMap<&str, &str>` (e.g.) from a PyDict - these strings cannot be guaranteed to be safe to borrow without pyo3 owned references. Instead you should use `HashMap<String, String>`.
