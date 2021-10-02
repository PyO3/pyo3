mod hygiene {
    mod misc;
    mod pyclass;
    mod pyfunction;
    // cannot implement protos in #[pymethods] using `multiple-pymethods` feature
    #[cfg(not(feature = "multiple-pymethods"))]
    mod pymethods;
    mod pymodule;
    mod pyproto;
}
