// ---------------------------------------------------------------------------
// Common types which are used for `pyo3-ffi` to define the types needed
// to call the Python C API.

use core::ffi::*;
// TODO: it _might_ be better to have this not in the namespace and instead
// use full-path names in atomic.rs
#[cfg(not(Py_LIMITED_API))]
use core::sync::atomic::*;
#[cfg(any(not(Py_3_10), not(Py_LIMITED_API)))]
use libc::FILE;
use libc::{size_t, wchar_t};

// ---------------------------------------------------------------------------
// Headers from the Python C API, rewritten as Rust equivalents.

include!("pyport.rs");
// skipped exports.h
// skipped pymacro.h
// skipped pymath.h
include!("pymem.rs");
include!("pytypedefs.rs");
#[cfg(Py_3_11)]
include!("pybuffer.rs");
// skipped pystats.h
include!("pyatomic.rs");
#[cfg(not(Py_LIMITED_API))]
include!("cpython/pylock.rs");
include!("critical_section.rs");
include!("object.rs");
include!("refcount.rs");
include!("objimpl.rs");
include!("slots.rs");
include!("slots_generated.rs");
include!("pyhash.rs");
#[cfg(not(Py_LIMITED_API))]
include!("cpython/pydebug.rs");
include!("bytearrayobject.rs");
include!("bytesobject.rs");
include!("unicodeobject.rs");
include!("pyerrors.rs");
include!("longobject.rs");
// skipped cpython/longintrepr.h
include!("boolobject.rs");
include!("floatobject.rs");
include!("complexobject.rs");
include!("rangeobject.rs");
include!("memoryobject.rs");
include!("tupleobject.rs");
include!("listobject.rs");
include!("dictobject.rs");
// skipped cpython/odictobject.h
include!("enumobject.rs");
include!("setobject.rs");
include!("methodobject.rs");
include!("moduleobject.rs");
// skipped cpython/monitoring.h
#[cfg(not(Py_LIMITED_API))]
include!("cpython/funcobject.rs");
// skipped cpython/classobject.h
include!("fileobject.rs");
include!("pycapsule.rs");
include!("cpython/code.rs");
include!("pyframe.rs");
include!("traceback.rs");
include!("sliceobject.rs");
#[cfg(not(Py_LIMITED_API))]
include!("cpython/cellobject.rs");
include!("iterobject.rs");
#[cfg(not(Py_LIMITED_API))]
include!("cpython/initconfig.rs");
include!("pystate.rs");
#[cfg(not(Py_LIMITED_API))]
include!("cpython/genobject.rs");
include!("descrobject.rs");
include!("genericaliasobject.rs");
// skipped cpython/sentinelobject.h
include!("warnings.rs");
include!("weakrefobject.rs");
include!("structseq.rs");
// skipped cpython/picklebufobject.h
// skipped cpython/pytime.h
include!("codecs.rs");
// skipped pythread.h
#[cfg(not(Py_LIMITED_API))]
include!("cpython/context.rs");
#[cfg(not(Py_3_10))]
include!("pyarena.rs");
include!("modsupport.rs");
include!("compile.rs");
include!("pythonrun.rs");
include!("pylifecycle.rs");
include!("ceval.rs");
include!("sysmodule.rs");
// skipped audit.h
include!("osmodule.rs");
include!("intrcheck.rs");
include!("import.rs");
include!("abstract.rs");
include!("bltinmodule.rs");
// skipped cpython/pyctype.h
include!("pystrtod.rs");
// skipped pystrcmp.h
include!("fileutils.rs");
// skipped cpython/pyfpe.h
// skipped cpython/tracemalloc.h
