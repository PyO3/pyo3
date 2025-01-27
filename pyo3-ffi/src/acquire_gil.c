#if defined(_WIN32)
#else
#include <pthread.h>
#include <unistd.h>
#endif

#if defined(PYTHON_IS_PYPY)
#define gil_func_name PyPyGILState_Ensure
#define wrapped_func_name PyPyGILState_Ensure_Safe
#else
#define gil_func_name PyGILState_Ensure
#define wrapped_func_name PyGILState_Ensure_Safe
#endif

int wrapped_func_name(void);
int gil_func_name(void);

#if defined(_WIN32)
int wrapped_func_name(void) {
    // In MSVC, PyThread_exit_thread calls _endthreadex(0), which does not use SEH. This can
    // cause Rust-level UB if there is pinned memory, but AFAICT there's not much we can do about it.
    return gil_func_name();
}
#else
static void hang_thread(void *ignore) {
    (void)ignore;
    while(1) {
#if defined(__wasi__)
            sleep(9999999);  // WASI doesn't have pause() ?!
#else
            pause();
#endif
    }
}

int wrapped_func_name(void) {
    // Do the equivalent of https://github.com/python/cpython/issues/87135 (included
    // in Python 3.14) to avoid pthread_exit unwinding the current thread, which tends
    // to cause undefined behavior in Rust.
    //
    // Unfortunately, I don't know of a way to do a catch(...) from Rust.
    int ret;
    pthread_cleanup_push(hang_thread, NULL);
    ret = gil_func_name();
    pthread_cleanup_pop(0);
    return ret;
}
#endif