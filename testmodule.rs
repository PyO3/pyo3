#![crate_type = "dylib"] 
#![feature(phase)]

#[phase(plugin, link)] extern crate abort_on_panic;
extern crate "rust-cpython" as cpython;
extern crate rustrt;
extern crate libc;

use cpython::{PyModule, PyResult, Python};

#[no_mangle]
pub extern "C" fn inittestmodule() {
	//abort_on_panic!({
		let py = unsafe { Python::assume_gil_acquired() };
		if let Err(e) = init(py) {
			println!("Restore error")
			e.restore()
		}
	//})
}

fn init(py : Python) -> PyResult<()> {
	let m : &PyModule = try!(py.init_module("testmodule", None));
	//println!("init_module done")
	try!(m.add_object("__author__", "Daniel Grunwald"));
	try!(m.add_object("__version__", "0.0.1"));
	Ok(())
}

