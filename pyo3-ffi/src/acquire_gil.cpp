#if defined(_WIN32)
#include <windows.h>
#include <synchapi.h>
#else
#include <unistd.h>
#endif

#if defined(PYTHON_IS_PYPY)
#define gil_func_name PyPyGILState_Ensure
#define wrapped_func_name PyPyGILState_Ensure_Safe
#else
#define gil_func_name PyGILState_Ensure
#define wrapped_func_name PyGILState_Ensure_Safe
#endif

extern "C" {
    int wrapped_func_name(void);
    int gil_func_name(void);
};

#if !defined(_WIN32)
// mark the wrapped function as visibility("hidden") to avoid causing namespace pollution
__attribute__((visibility("hidden")))
#endif
int wrapped_func_name(void) {
    // Do the equivalent of https://github.com/python/cpython/issues/87135 (included
    // in Python 3.14) to avoid pthread_exit unwinding the current thread, which tends
    // to cause undefined behavior in Rust.
    //
    // Unfortunately, I don't know of a way to do a catch(...) from Rust.
    try {
        return gil_func_name();
    } catch(...) {
        while(1) {
#if defined(_WIN32)
            SleepEx(INFINITE, TRUE);
#elif defined(__wasi__)
            sleep(9999999);  // WASI doesn't have pause() ?!
#else
            pause();
#endif
        }
    }
}

