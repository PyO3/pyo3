#include <Python.h>

int py_is_initialized(void) {
    return Py_IsInitialized();
}
