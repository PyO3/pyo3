// Copyright (c) 2017-present PyO3 Project and Contributors

use std::fmt;

use pointers::PyPtr;
use python::Python;
use objectprotocol::ObjectProtocol;

impl fmt::Debug for PyPtr {
    fn fmt(&self, f : &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let gil = Python::acquire_gil();
        let py = gil.python();

        // TODO: we shouldn't use fmt::Error when repr() fails
        let r = self.as_object(py);
        let repr_obj = try!(r.repr(py).map_err(|_| fmt::Error));
        f.write_str(&repr_obj.to_string_lossy(py))
    }
}

impl fmt::Display for PyPtr {
    default fn fmt(&self, f : &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let gil = Python::acquire_gil();
        let py = gil.python();

        // TODO: we shouldn't use fmt::Error when repr() fails
        let r = self.as_object(py);
        let repr_obj = try!(r.str(py).map_err(|_| fmt::Error));
        f.write_str(&repr_obj.to_string_lossy(py))
    }
}
