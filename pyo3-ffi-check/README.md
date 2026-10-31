# pyo3-ffi-check

This is a simple program which compares ffi definitions from `pyo3-ffi` against those produced by `bindgen`.

It checks type layouts, function signatures, and the addresses of functions and statics. Any differences are printed to stdout and the process exits nonzero.

The main purpose of this program is to be run as part of PyO3's continuous integration pipeline to catch possible errors in PyO3's ffi definitions.
