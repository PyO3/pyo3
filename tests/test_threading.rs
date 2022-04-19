use pyo3::prelude::*;

#[test]
fn test_threading_id() {
    const SOURCE: &str = r#"
import threading

current = threading.current_thread()
main = threading.main_thread()

assert main is not current, (main, current)
"#;

    // If somehow the interpreter has not yet been initialized,
    // this will initialize it
    Python::with_gil(|_| {});

    // spawn an `alien thread` unknown to the `threading` module.
    let handle = std::thread::spawn(|| Python::with_gil(|py| py.run(SOURCE, None, None)));

    if let Err(err) = handle.join().unwrap() {
        Python::with_gil(|py| {
            let value = err.value(py);
            panic!(
                "`threading` module identified a newly spawned thread as the main thread: {:?}.",
                value
            );
        });
    }
}
