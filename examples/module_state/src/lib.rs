//! Example: Module State API (Experimental/Future)
//!
//! This example demonstrates the declarative module state API that will be available
//! once the `experimental-module-state` feature is stabilized.
//!
//! The key features shown here:
//! - `#[pymodule_state]` marker on a state struct
//! - `#[pymodule(state = ModuleState)]` declarative state binding
//! - `module_state()` and `module_state_mut()` API for accessing state
//! - Type-safe state storage per module

use pyo3::prelude::*;
use std::sync::Mutex;

/// Module state struct marked with `#[pymodule_state]`
///
/// This struct will be automatically instantiated during module initialization
/// and made available to all functions in the module via `module_state()`.
struct ModuleState {
    /// Counter that increments with each call to `increment()`
    counter: Mutex<i32>,
    /// Configuration string set at init time
    config: String,
}

impl ModuleState {
    /// Initialize the module state
    ///
    /// This is called once when the module is first imported.
    /// Any initialization errors here are propagated to Python.
    fn new() -> PyResult<Self> {
        println!("Initializing module state...");
        Ok(ModuleState {
            counter: Mutex::new(0),
            config: "initialized".to_string(),
        })
    }
}

/// Get the current counter value from module state
///
/// Functions that need to access module state must use `pass_module` to receive
/// the module as a parameter.
#[pyfunction(pass_module)]
fn get_counter(m: &Bound<'_, PyModule>) -> PyResult<i32> {
    // Safe API: returns Option<&T>, handles missing/wrong type
    if let Some(state) = m.module_state::<ModuleState>() {
        Ok(*state.counter.lock().unwrap())
    } else {
        Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
            "Module state not available",
        ))
    }
}

/// Increment the counter in module state
///
/// Note: This requires `unsafe` access to get a mutable reference to state.
/// Generally safe during initialization or under the GIL.
#[pyfunction(pass_module)]
fn increment_counter(m: &Bound<'_, PyModule>) -> PyResult<i32> {
    let new_value = {
        if let Some(state) = m.module_state::<ModuleState>() {
            *state.counter.lock().unwrap() += 1;
            *state.counter.lock().unwrap()
        } else {
            return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "Module state not available",
            ));
        }
    };
    Ok(new_value)
}

/// Get the configuration from module state
#[pyfunction(pass_module)]
fn get_config(m: &Bound<'_, PyModule>) -> PyResult<String> {
    if let Some(state) = m.module_state::<ModuleState>() {
        Ok(state.config.clone())
    } else {
        Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
            "Module state not available",
        ))
    }
}

/// Example of accessing state from a class method
///
/// Classes defined in a module with state can access that state via Python token.
#[pyclass(module = "module_state")]
struct Counter {
    name: String,
}

#[pymethods]
impl Counter {
    #[new]
    fn new(name: String) -> Self {
        Counter { name }
    }

    /// Access module state from a class method using Python token
    fn get_shared_counter_value(slf: &Bound<'_, Self>) -> PyResult<i32> {
        if let Some(state) = slf.py().type_module_state::<Counter, ModuleState>()? {
            Ok(*state.counter.lock().unwrap())
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "Module state not available",
            ))
        }
    }

    /// Update module state from a class method
    fn increment_shared_counter(slf: &Bound<'_, Self>) -> PyResult<i32> {
        if let Some(state) = slf.py().type_module_state::<Counter, ModuleState>()? {
            *state.counter.lock().unwrap() += 1;
            Ok(*state.counter.lock().unwrap())
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "Module state not available",
            ))
        }
    }
}

/// Module definition using declarative state API
#[pymodule(state = ModuleState)]
fn module_state(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<ModuleState> {
    // Add functions to the module
    m.add_function(wrap_pyfunction!(get_counter, m)?)?;
    m.add_function(wrap_pyfunction!(increment_counter, m)?)?;
    m.add_function(wrap_pyfunction!(get_config, m)?)?;

    // Add classes to the module
    m.add_class_with_module::<Counter>()?;

    // Add module docstring
    m.add(
        "__doc__",
        "Module State Example\n\
         \n\
         This module demonstrates the experimental module state API.\n\
         It maintains per-module state that persists across function calls.\n\
         \n\
         Functions:\n\
         - get_counter(): Get the current counter value from module state\n\
         - increment_counter(): Increment the counter and return new value\n\
         - get_config(): Get the current configuration string\n\
         \n\
         Classes:\n\
         - Counter: Example class with method accessing module state\
        ",
    )?;

    ModuleState::new()
}
