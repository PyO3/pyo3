#![crate_type = "dylib"] 


#[macro_use] extern crate cpython;
extern crate "python27-sys" as py27;
//extern crate libc;

use cpython::{PyModule, PyResult, Python};

/*
py_module_initializer!("testmodule", inittestmodule, |py, m| {
    println!("in initializer");
    //try!(m.add(cstr!("__doc__"), "Module documentation string"));
	//try!(m.add(cstr!("__author__"), "Daniel Grunwald"));
	//try!(m.add(cstr!("__version__"), "0.0.1"));
	Ok(())    
});
*/

#[no_mangle]
pub extern "C" fn inittestmodule() {
	//abort_on_panic!({
		let py = unsafe { Python::assume_gil_acquired() };
		if let Err(e) = init(py) {
			println!("Restore error");
			e.restore()
		}
	//})
}

fn init(py : Python) -> PyResult<()> {
	let m : &PyModule = try!(py.init_module("testmodule", None));
	//unsafe { py27::Py_InitModule(cstr!("testmodule").as_ptr(), std::ptr::null_mut()) };
	println!("init_module done");
    Ok(())
}

