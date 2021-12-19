pub use pyo3_macros::py_dict;

#[doc(hidden)]
#[macro_export]
macro_rules! py_object_vec {
    ($py:ident, [$($item:expr),+]) => {{
        let items_vec: Vec<$crate::PyObject> =
            vec![$($crate::conversion::IntoPy::into_py($item, $py)),+];
        items_vec
    }};
}

#[macro_export]
macro_rules! py_list {
    ($py:ident, [$($items:expr),+]) => {{
        let items_vec = $crate::py_object_vec!($py, [$($items),+]);
        $crate::types::list::PyList::new($py, items_vec)
    }};
}

#[macro_export]
macro_rules! py_tuple {
    ($py:ident, ($($items:expr),+)) => {{
        let items_vec = $crate::py_object_vec!($py, [$($items),+]);
        $crate::types::PyTuple::new($py, items_vec)
    }};
}

#[macro_export]
macro_rules! py_set {
    ($py:ident, {$($items:expr),+}) => {{
        let items_vec = $crate::py_object_vec!($py, [$($items),+]);
        $crate::types::set::PySet::new($py, items_vec.as_slice())
    }};
}

#[macro_export]
macro_rules! py_frozenset {
    ($py:ident, {$($items:expr),+}) => {{
        let items_vec = $crate::py_object_vec!($py, [$($items),+]);
        $crate::types::set::PyFrozenSet::new($py, items_vec.as_slice())
    }};
}

#[cfg(test)]
mod test {
    use crate::types::PyFrozenSet;
    use crate::Python;

    #[test]
    fn test_list_macro() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let single_item_list = py_list!(py, ["elem"]);
        assert_eq!(
            "elem",
            single_item_list
                .get_item(0)
                .expect("failed to get item")
                .extract::<&str>()
                .unwrap()
        );

        let multi_item_list = py_list!(py, ["elem1", "elem2", 3, 4]);

        assert_eq!(
            "['elem1', 'elem2', 3, 4]",
            multi_item_list.str().unwrap().extract::<&str>().unwrap()
        );
    }

    #[test]
    fn test_tuple_macro() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let single_item_tuple = py_tuple!(py, ("elem"));
        assert_eq!(
            "elem",
            single_item_tuple
                .get_item(0)
                .expect("failed to get item")
                .extract::<&str>()
                .unwrap()
        );

        let multi_item_tuple = py_tuple!(py, ("elem1", "elem2", 3, 4));

        assert_eq!(
            "('elem1', 'elem2', 3, 4)",
            multi_item_tuple.str().unwrap().extract::<&str>().unwrap()
        );
    }

    #[test]
    fn test_set_macro() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let set = py_set!(py, { "set_elem" }).expect("failed to create set");

        assert!(set.contains("set_elem").unwrap());

        set.call_method1(
            "update",
            py_tuple!(
                py,
                (py_set!(py, {"new_elem1", "new_elem2", "set_elem"}).unwrap())
            ),
        )
        .expect("failed to update set");

        for &expected_elem in &["set_elem", "new_elem1", "new_elem2"] {
            assert!(set.contains(expected_elem).unwrap());
        }
    }

    #[test]
    fn test_frozenset_macro() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let frozenset = py_frozenset!(py, { "set_elem" }).expect("failed to create frozenset");

        assert!(frozenset.contains("set_elem").unwrap());

        let intersection = frozenset
            .call_method1(
                "intersection",
                py_tuple!(
                    py,
                    (py_set!(py, {"new_elem1", "new_elem2", "set_elem"}).unwrap())
                ),
            )
            .expect("failed to call intersection()")
            .downcast::<PyFrozenSet>()
            .expect("failed to downcast to FrozenSet");

        assert_eq!(1, intersection.len());
        assert!(intersection.contains("set_elem").unwrap());
    }
}
