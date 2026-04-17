use pyo3::prelude::*;
use std::ffi::c_void;

#[pyclass]
struct NotSyncNotSend(*mut c_void);
//~^ ERROR: `*mut c_void` cannot be sent between threads safely
//~| ERROR: `*mut c_void` cannot be shared between threads safely

#[pyclass]
struct SendNotSync(*mut c_void);
//~^ ERROR: `*mut c_void` cannot be shared between threads safely
unsafe impl Send for SendNotSync {}

#[pyclass]
struct SyncNotSend(*mut c_void);
//~^ ERROR: `*mut c_void` cannot be sent between threads safely
unsafe impl Sync for SyncNotSend {}

// None of the `unsendable` forms below should fail to compile

#[pyclass(unsendable)]
struct NotSyncNotSendUnsendable(*mut c_void);

#[pyclass(unsendable)]
struct SendNotSyncUnsendable(*mut c_void);
unsafe impl Send for SendNotSyncUnsendable {}

#[pyclass(unsendable)]
struct SyncNotSendUnsendable(*mut c_void);
unsafe impl Sync for SyncNotSendUnsendable {}

fn main() {}
