macro_rules! py_dict {
    ($py:ident, {$key:literal : $value:expr}) => {
        $crate::types::dict::IntoPyDict::into_py_dict(&[($key, $value)], $py)
    };

    ($py:ident, {$key:literal : $value:expr, $($keys:literal : $values:expr),+}) => {{
        let dict = py_dict!($py, {$($keys : $values),+});
        dict.set_item($key, $value).expect("failed to set item on dict");
        dict
    }};
}

macro_rules! py_list {
    ($py:ident, [$($items:expr),+]) => {{
        let list = py_list!(impl ($py, [$($items),+]));
        list.reverse().expect("failed to reverse list");
        list
    }};

    (impl ($py:ident, [$item:expr])) => {
        $crate::types::list::PyList::new($py, &[$item])
    };

    (impl ($py:ident, [$head:expr, $($rest:expr),+])) => {{
        let list = py_list!(impl ($py, [$($rest),+]));
        list.append($head).expect("failed to append item");
        list
    }};
}

macro_rules! py_tuple {
    ($py:ident, ($($items:expr),+)) => {{
        let items_vec = py_tuple!(impl ($py, ($($items),+)));

        $crate::types::tuple::PyTuple::new($py, items_vec.iter().rev())
    }};

    (impl ($py:ident, ($item:expr))) => {
        vec![$crate::conversion::IntoPy::into_py($item, $py)]
    };

    (impl ($py:ident, ($head:expr, $($rest:expr),+))) => {{
        let mut items_vec = py_tuple!(impl ($py, ($($rest),+)));
        items_vec.push($crate::conversion::IntoPy::into_py($head, $py));
        items_vec
    }};
}

#[cfg(test)]
mod test {
    use crate::Python;
    #[test]
    fn test_dict_macro() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let single_elem_dict = py_dict!(py, { "a": 2 });
        assert_eq!(
            2,
            single_elem_dict
                .get_item("a")
                .unwrap()
                .extract::<i32>()
                .unwrap()
        );

        let value = "value";
        let multi_elem_dict = py_dict!(py, {"key1": value, 143: "abcde", "name": "Даня"});
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

        let multi_item_list =
            py_list!(py, ["elem1", "elem2", 3, 4, py_dict!(py, {"type": "user"})]);

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

        let multi_item_tuple =
            py_tuple!(py, ("elem1", "elem2", 3, 4, py_dict!(py, {"type": "user"})));

        assert_eq!(
            "('elem1', 'elem2', 3, 4, {'type': 'user'})",
            multi_item_tuple.str().unwrap().extract::<&str>().unwrap()
        );
    }
}
