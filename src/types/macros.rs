macro_rules! py_object_vec {
    ($py:ident, [$($items:expr),+]) => {{
        let mut items_vec: Vec<$crate::instance::PyObject> = py_object_vec!(impl ($py, [$($items),+]));
        items_vec.reverse();
        items_vec
    }};

    (impl ($py:ident, [$item:expr])) => {
        vec![$crate::conversion::IntoPy::into_py($item, $py)]
    };

    (impl ($py:ident, [$head:expr, $($rest:expr),+])) => {{
        let mut items_vec = py_object_vec!(impl ($py, [$($rest),+]));
        items_vec.push($crate::conversion::IntoPy::into_py($head, $py));
        items_vec
    }};

}

macro_rules! py_dict {
    ($py:ident, {$($keys:literal : $values:expr),+}) => {{
        let items: $crate::instance::PyObject = py_list!($py, [$(($keys, $values)),+]).into();

        $crate::types::dict::PyDict::from_sequence($py, items)
    }};
}

macro_rules! py_list {
    ($py:ident, [$($items:expr),+]) => {{
        let items_vec = py_object_vec!($py, [$($items),+]);
        $crate::types::list::PyList::new($py, items_vec)
    }};
}

macro_rules! py_tuple {
    ($py:ident, ($($items:expr),+)) => {{
        let items_vec = py_object_vec!($py, [$($items),+]);
        $crate::types::tuple::PyTuple::new($py, items_vec)
    }};
}

macro_rules! py_set {
    ($py:ident, {$($items:expr),+}) => {{
        let items_vec = py_object_vec!($py, [$($items),+]);
        $crate::types::set::PySet::new($py, items_vec.as_slice())
    }};
}

macro_rules! py_frozenset {
    ($py:ident, {$($items:expr),+}) => {{
        let items_vec = py_object_vec!($py, [$($items),+]);
        $crate::types::set::PyFrozenSet::new($py, items_vec.as_slice())
    }};
}

#[cfg(test)]
mod test {
    use crate::types::PyFrozenSet;
    use crate::Python;

    #[test]
    fn test_dict_macro() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let single_elem_dict = py_dict!(py, { "a": 2 }).expect("failed to create dict");
        assert_eq!(
            2,
            single_elem_dict
                .get_item("a")
                .unwrap()
                .extract::<i32>()
                .unwrap()
        );

        let value = "value";
        let multi_elem_dict = py_dict!(py, {"key1": value, 143: "abcde", "name": "Даня"})
            .expect("failed to create dict");
        assert_eq!(
            "value",
            multi_elem_dict
                .get_item("key1")
                .unwrap()
                .extract::<&str>()
                .unwrap()
        );
        assert_eq!(
            "abcde",
            multi_elem_dict
                .get_item(143)
                .unwrap()
                .extract::<&str>()
                .unwrap()
        );
    }

    #[test]
    fn test_list_macro() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let single_item_list = py_list!(py, ["elem"]);
        assert_eq!(
            "elem",
            single_item_list.get_item(0).extract::<&str>().unwrap()
        );

        let multi_item_list = py_list!(
            py,
            [
                "elem1",
                "elem2",
                3,
                4,
                py_dict!(py, {"type": "user"}).unwrap()
            ]
        );

        assert_eq!(
            "['elem1', 'elem2', 3, 4, {'type': 'user'}]",
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
            single_item_tuple.get_item(0).extract::<&str>().unwrap()
        );

        let multi_item_tuple = py_tuple!(
            py,
            (
                "elem1",
                "elem2",
                3,
                4,
                py_dict!(py, {"type": "user"}).unwrap()
            )
        );

        assert_eq!(
            "('elem1', 'elem2', 3, 4, {'type': 'user'})",
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

        for expected_elem in vec!["set_elem", "new_elem1", "new_elem2"] {
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
