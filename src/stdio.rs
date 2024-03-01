//! Enables direct write access to I/O streams in Python's `sys` module.

//! In some cases printing to Rust's `std::io::stdout` or `std::io::stderr` will not appear
//! in the Python interpreter, e.g. in Jupyter notebooks. This module provides a way to write
//! directly to Python's I/O streams from Rust in such cases.

//! ```rust
//! let mut stdout = pyo3::stdio::stdout();
//!   
//! // This may not appear in Jupyter notebooks...
//! println!("Hello, world!");
//!
//! // ...but this will.
//! writeln!(stdout, "Hello, world!").unwrap();
//! ```

use crate::ffi::{PySys_WriteStderr, PySys_WriteStdout};
use crate::prelude::*;
use std::io::{LineWriter, Write};
use std::os::raw::c_char;

macro_rules! make_python_stdio {
    ($rawtypename:ident, $typename:ident, $pyfunc:ident, $stdio:ident) => {
        struct $rawtypename {
            cbuffer: Vec<u8>,
        }
        impl $rawtypename {
            fn new() -> Self {
                Self {
                    cbuffer: Vec::new(),
                }
            }
        }
        impl Write for $rawtypename {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                //clear internal buffer and then overwrite with the
                //new buffer and a null terminator
                self.cbuffer.clear();
                self.cbuffer.extend_from_slice(buf);
                self.cbuffer.push(0);
                Python::with_gil(|_py| unsafe {
                    $pyfunc(self.cbuffer.as_ptr() as *const c_char);
                });
                Ok(buf.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                // call the python flush() on sys.$pymodname
                Python::with_gil(|py| -> std::io::Result<()> {
                    py.run_bound(
                        std::concat!("import sys; sys.", stringify!($stdio), ".flush()"),
                        None,
                        None,
                    )
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
                    Ok(())
                })
            }
        }

        #[doc=std::concat!("A handle to Python's `sys.", stringify!($stdio),"` stream.")]
        pub struct $typename {
            inner: LineWriter<$rawtypename>,
        }

        impl $typename {
             fn new() -> Self {
                Self {
                    inner: LineWriter::new($rawtypename::new()),
                }
            }
        }

        impl Write for $typename {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                self.inner.write(buf)
            }
            fn flush(&mut self) -> std::io::Result<()> {
                self.inner.flush()
            }
        }

        #[doc=std::concat!("Construct a new handle to Python's `sys.", stringify!($stdio),"` stream.")]
        pub fn $stdio() -> $typename {
            $typename::new()
        }

    };

}
make_python_stdio!(PyStdoutRaw, PyStdout, PySys_WriteStdout, stdout);
make_python_stdio!(PyStderrRaw, PyStderr, PySys_WriteStderr, stderr);
