#![cfg(not(any(PyPy, GraalPy)))]
#![cfg(not(target_arch = "wasm32"))]

use pyo3::types::PyAnyMethods;

// Regression test for `try_attach` fast-path returning before site.py had
// finished initializing.
#[test]
fn test_concurrent_init_site_race() {
    let tmpdir = std::env::temp_dir().join("pyo3_test_init_race");
    std::fs::create_dir_all(&tmpdir).unwrap();

    std::fs::write(
        tmpdir.join("sitecustomize.py"),
        "import sys, time\n\
         sys._pyo3_site_done = False\n\
         time.sleep(2)\n\
         sys._pyo3_site_done = True\n",
    )
    .unwrap();

    std::env::set_var("PYTHONPATH", &tmpdir);

    std::thread::scope(|s| {
        s.spawn(|| {
            pyo3::Python::initialize();
        });

        s.spawn(|| loop {
            let result = pyo3::Python::try_attach(|py| {
                let done = py
                    .import("sys")
                    .unwrap()
                    .getattr("_pyo3_site_done")
                    .unwrap()
                    .extract::<bool>()
                    .unwrap();
                assert!(done);
            });
            if result.is_some() {
                break;
            }
            std::hint::spin_loop();
        });
    });
}
