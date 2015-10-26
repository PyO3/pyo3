// Copyright (c) 2015 Dmitry Trofimov
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of this
// software and associated documentation files (the "Software"), to deal in the Software
// without restriction, including without limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons
// to whom the Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all copies or
// substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED,
// INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR
// PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE
// FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR
// OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

use {Python, PyDict, ToPyObject, PyInt};
use std::collections::{BTreeMap, HashMap};

// TODO: move these tests into the dict module
#[test]
fn test_hashmap_to_python() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let mut map = HashMap::<i32, i32>::new();
    map.insert(1, 1);

    let py_map = map.to_py_object(py);

    assert!(py_map.len(py) == 1);
    assert!( py_map.get_item(py, 1).unwrap().extract::<i32>(py).unwrap() == 1);
}

#[test]
fn test_btreemap_to_python() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let mut map = BTreeMap::<i32, i32>::new();
    map.insert(1, 1);

    let py_map = map.to_py_object(py);

    assert!(py_map.len(py) == 1);
    assert!( py_map.get_item(py, 1).unwrap().extract::<i32>(py).unwrap() == 1);
}

