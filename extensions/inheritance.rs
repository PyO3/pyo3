#![crate_type = "dylib"]

#[macro_use] extern crate cpython;

py_module_initializer!(inheritance, initinheritance, PyInit_inheritance, |py, m| {
    try!(m.add(py, "__doc__", "Module documentation string"));
    let mut base_class_builder = m.add_type::<()>(py, "BaseClass");
    base_class_builder.doc("Type doc string");
    let base_class = try!(base_class_builder.finish());
    for i in 1..10 {
        let mut class_builder = m.add_type::<()>(py, &format!("C{}", i)).base(&base_class);
        class_builder.doc(&format!("Derived class #{}", i));
        try!(class_builder.finish());
    }
    Ok(())
});

