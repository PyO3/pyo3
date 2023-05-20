use pyo3::prelude::*;

#[pyclass(frozen, unsendable)]
struct ClassThatIsSyncButNotSend {
    ptr: *mut (),
}

unsafe impl Sync for ClassThatIsSyncButNotSend {}

fn test_py_get_class_that_is_not_send_on_different_thread() {
    let class = Python::with_gil(|py| {
        Py::new(py, ClassThatIsSyncButNotSend { ptr: &mut () }).unwrap()
    });

    std::thread::spawn(move || {
        let _ptr = class.get().ptr;
    });
}


fn test_pycell_get_class_that_is_not_send_on_different_thread() {
    let class = Python::with_gil(|py| {
        Py::new(py, ClassThatIsSyncButNotSend { ptr: &mut () }).unwrap()
    });

    std::thread::spawn(move || {
        Python::with_gil(|py| {
            let _ptr = class.as_ref(py).get().ptr;
        });
    });
}

fn main() {
    test_py_get_class_that_is_not_send_on_different_thread();
    test_pycell_get_class_that_is_not_send_on_different_thread();
}
