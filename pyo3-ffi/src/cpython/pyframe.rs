// NB used in `_PyEval_EvalFrameDefault`, maybe we remove this too.
#[cfg(all(Py_3_11, not(PyPy)))]
opaque_struct!(pub _PyInterpreterFrame);
