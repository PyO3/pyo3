# PyPy Support

Using PyPy is supported via cpyext.

Support is only provided for building rust extension for code running under PyPy. This means PyPy **cannot** be called from rust via cpyext. Note that there some differences in the ffi module between pypy and cpython. 
 
This is a limitation of cpyext and supported for embedding cpyext is not planned.

Compilation against PyPy is done by exporting the `PYTHON_SYS_EXECUTABLE` to a pypy binary or by compiling in a PyPy virtualenv.

For example, `PYTHON_SYS_EXECUTABLE="/path/to/pypy3" /path/to/pypy3 setup.py install`


## Unsupported Features 

These are features currently supported by PyO3, but not yet implemented in cpyext.

- Complex number functions (`_Py_c_sum`, `_Py_c_sum` ..)
- Conversion to rust's i128, u128 types.
- `PySequence_Count` (which is used to count number of element in array)
- `PyDict_MergeFromSeq2` (used in `PyDict::from_sequence`)