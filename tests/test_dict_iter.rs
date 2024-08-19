use pyo3::prelude::*;
use pyo3::types::IntoPyDict;

#[test]
#[cfg_attr(target_arch = "wasm32", ignore)] // Not sure why this fails.
#[cfg_attr(Py_GIL_DISABLED, ignore)] // test deadlocks in GIL-disabled build, TODO: fix deadlock
fn iter_dict_nosegv() {
    Python::with_gil(|py| {
        const LEN: usize = 10_000_000;
        let dict = (0..LEN as u64).map(|i| (i, i * 2)).into_py_dict(py);
        let mut sum = 0;
        for (k, _v) in dict {
            let i: u64 = k.extract().unwrap();
            sum += i;
        }
        assert_eq!(sum, 49_999_995_000_000);
    });
}
