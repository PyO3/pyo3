use plugin_api::plugin_api as pylib_module;
use pyo3::prelude::*;
use pyo3::types::PyList;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    //"export" our API module to the python runtime
    pyo3::append_to_inittab!(pylib_module);
    //spawn runtime
    pyo3::prepare_freethreaded_python();
    //import path for python
    let path = Path::new("./python_plugin/");
    //do useful work
    Python::with_gil(|py| {
        //add the current directory to import path of Python (do not use this in production!)
        let syspath: &PyList = py.import("sys")?.getattr("path")?.extract()?;
        syspath.insert(0, &path)?;
        println!("Import path is: {:?}", syspath);

        // Now we can load our python_plugin/gadget_init_plugin.py file.
        // It can in turn import other stuff as it deems appropriate
        let plugin = PyModule::import_bound(py, "gadget_init_plugin")?;
        // and call start function there, which will return a python reference to Gadget.
        // Gadget here is a "pyclass" object reference
        let gadget = plugin.getattr("start")?.call0()?;

        //now we extract (i.e. mutably borrow) the rust struct from python object
        {
            //this scope will have mutable access to the gadget instance, which will be dropped on
            //scope exit so Python can access it again.
            let mut gadget_rs: PyRefMut<'_, plugin_api::Gadget> = gadget.extract()?;
            // we can now modify it as if it was a native rust struct
            gadget_rs.prop = 42;
            //which includes access to rust-only fields that are not visible to python
            println!("rust-only vec contains {:?}", gadget_rs.rustonly);
            gadget_rs.rustonly.clear();
        }

        //any modifications we make to rust object are reflected on Python object as well
        let res: usize = gadget.getattr("prop")?.extract()?;
        println!("{res}");
        Ok(())
    })
}
