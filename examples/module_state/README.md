# Module State Example

This example demonstrates the **experimental module state API** that will be available in PyO3 once the `experimental-module-state` feature is stabilized.

## Overview

The module state API allows you to:

- Define per-module state that persists across function calls
- Initialize state once during module import
- Access state from functions and methods safely
- Share data across the module (like configuration, caches, counters, etc.)

## Key API Features

### 1. **State Struct Definition**

```rust
#[pymodule_state]
struct ModuleState {
    counter: i32,
    config: String,
    data: Arc<String>,
}
```

The `#[pymodule_state]` marker tells PyO3 that this struct represents the module's state.

### 2. **Module Declaration with State**

```rust
#[pymodule(state = ModuleState)]
fn module_state(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<ModuleState> {
    m.add_function(wrap_pyfunction!(get_counter, m)?)?;
    // ... add more functions/classes
    ModuleState::new()  // Initialize and return state
}
```

The `state = ModuleState` parameter tells PyO3 to manage `ModuleState` for this module.

### 3. **Accessing State from Functions with `pass_module`**

Functions that need to access module state must use the `pass_module` attribute:

```rust
/// Functions need pass_module to receive the module parameter
#[pyfunction(pass_module)]
fn get_counter(m: &Bound<'_, PyModule>) -> PyResult<i32> {
    // Safe, immutable access returns Option<&T>
    if let Some(state) = m.module_state::<ModuleState>() {
        Ok(state.counter)
    } else {
        Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
            "Module state not available"
        ))
    }
}
```

**Key Points**:

- `#[pyfunction(pass_module)]` attribute tells PyO3 to inject the module as a parameter
- The module parameter comes before regular arguments
- Returns `Option<&T>` - handles missing state gracefully

### 4. **Mutating State from Functions**

```rust
#[pyfunction(pass_module)]
fn increment(m: &Bound<'_, PyModule>) -> PyResult<i32> {
    // Unsafe, mutable access returns Option<&mut T>
    unsafe {
        if let Some(state) = m.module_state_mut::<ModuleState>() {
            state.counter += 1;
            Ok(state.counter)
        } else {
            Err(...)
        }
    }
}
```

**Safety**: Requires explicit `unsafe` block.
Safe because:

- You're under the GIL (PyModule access implies it)
- You have sole mutable access to the module's state
- Generally only used during initialization

### 5. **Accessing State from Classes with `PyAnyMethods` (Optimized)**

Classes can access their defining module's state via `PyAnyMethods` - available on any object without reference counting overhead:

```rust
#[pymethods]
impl Counter {
    fn get_module_state_value(slf: &Bound<'_, Self>) -> PyResult<i32> {
        // Bound<'_, Counter> coerces to Bound<'_, PyAny>
        // type_module_state() uses Py_TYPE() internally - no incref/decref!
        if let Some(state) = slf.type_module_state::<ModuleState>() {
            Ok(state.counter)
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "Module state not available"
            ))
        }
    }

    fn update_config(slf: &Bound<'_, Self>, new_config: String) -> PyResult<()> {
        // Mutable access from class method
        unsafe {
            if let Some(state) = slf.type_module_state_mut::<ModuleState>() {
                state.config = new_config;
                Ok(())
            } else {
                Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                    "Module state not available"
                ))
            }
        }
    }
}
```

**Key Points**:

- `type_module_state::<T>()` - safe immutable access via Py_TYPE() + PyType_GetModuleState
- `type_module_state_mut::<T>()` - mutable access (requires unsafe)
- **No reference counting overhead** - uses `Py_TYPE()` directly instead of creating a `Bound<'_, PyType>`
- Works on any `Bound<'_, PyAny>`, including instances, not just types
- No need to call `get_type()` first

### 6. **Subinterpreter Isolation**

Each subinterpreter gets its own isolated state instance.
This is handled automatically by PyO3's module lifecycle system.

## What This Example Shows

### Immutable State Access

- `get_counter()` - read-only counter with `pass_module`
- `get_config()` - read-only configuration with `pass_module`
- `get_data()` - access shared Arc data with `pass_module`

### Mutable State Access

- `increment()` - modify counter with `pass_module`
- `set_config()` - update configuration with `pass_module`

### Class Integration with PyAnyMethods

- `Counter` class uses `type_module_state::<T>()` to access module state
- `get_module_state_value()` - immutable access from class method
- `update_config()` - mutable access from class method
- No need to import module or pass it around
- **Optimized**: Uses Py_TYPE() directly - no reference counting overhead

## Running the Example (Once Implemented)

```bash
# Build the module
cargo build --example module_state --features experimental-module-state

# In Python:
import module_state

# Read state via functions with pass_module
print(module_state.get_counter())  # 0

# Modify state
print(module_state.increment())    # 1
print(module_state.increment())    # 2

# Configure state
module_state.set_config("custom")
print(module_state.get_config())   # "custom"

# Access from class via PyTypeMethods
counter = module_state.Counter("test")
print(counter.get_module_state_value())  # 2
counter.update_config("from_class")
print(module_state.get_config())   # "from_class"
```

## Design Rationale

### Single State Type Per Module

Only **one** state type per module (not multiple types).
This:

- Simplifies the API and implementation
- Makes the state type known at compile-time
- Enables macro validation of init function return types
- Matches real-world usage patterns

### `pass_module` for Functions

The `pass_module` attribute on `#[pyfunction]`:

- Explicitly declares that a function needs the module
- Makes it clear in the code where module state is accessed
- PyO3 injects the module automatically before other arguments
- Better than implicit module parameter passing

### PyAnyMethods for Classes

Classes can access module state from any instance efficiently:

- `type_module_state::<T>()` uses Py_TYPE() + PyType_GetModuleState internally
- **No reference counting overhead** - works directly with borrowed type pointer
- Available on any `Bound<'_, PyAny>` (includes class instances)
- Works automatically for any class defined in a module with state
- No need to import or pass the module around in class methods
- More general than type-only API (works on instances too)

### Option-Based Access

`module_state()` and `type_module_state()` return `Option<&T>`:

- Safe handling of initialization failures
- Clear error path in code
- No surprising panics from type mismatches

### Explicit Unsafe for Mutation

Mutable access requires `unsafe`:

- Signals that mutable access needs care
- Documents the GIL assumption
- Encourages thinking about thread safety

## Comparison with PR #5600

This example shows the **redesigned API**, which differs from PR #5600:

| Aspect | PR #5600 | New Design |
|--------|----------|-----------|
| State Storage | `TypeMap` (multiple types) | `Box<dyn Any>` (single type) |
| PyModule API | `state_ref()`, `state_mut()` | `module_state()`, `module_state_mut()` |
| PyAny API | None | `type_module_state()`, `type_module_state_mut()` (via PyAnyMethods) |
| Function Access | No mechanism | `#[pyfunction(pass_module)]` |
| Class Access | No mechanism | Direct via `PyAnyMethods` on instance |
| Error Handling | `PyResult` | `Option` (None for missing) |
| Feature Gate | No | `experimental-module-state` |
| Type Validation | Runtime downcast | Compile-time macro validation |

## Key Differences from PR #5600

### State Storage

- **PR #5600**: Uses `TypeMap` to store multiple state types (complex)
- **New Design**: Single `Box<dyn Any>` per module (simple, matches real use cases)

### Function Access Pattern

- **PR #5600**: No standard pattern for function access
- **New Design**: Use `#[pyfunction(pass_module)]` to receive module and call `m.module_state::<T>()`

### Class Access Pattern

- **PR #5600**: No standard pattern for class access
- **New Design**: Use `PyAnyMethods` on any object instance to call `obj.type_module_state::<T>()` via Py_TYPE() + PyType_GetModuleState (no refcount overhead)

### API Method Names

- **PR #5600**: `state_ref()`, `state_mut()` (generic names)
- **New Design**:
  - `module_state()`, `module_state_mut()` on PyModule (domain-specific)
  - `type_module_state()`, `type_module_state_mut()` on PyAny (optimized, no refcount)

## Implementation Status

This example serves as the **guiding specification** for implementation.
Phases:

1. ✅ Phase 2.0: Fix proc macro slot counting bug
2. 🟡 Phase 2.1-2.7: Implement the API (in progress)
   - Phase 2.1: `#[pymodule_state]` macro
   - Phase 2.2: Parser extension + auto-detection
   - Phase 2.3: Function-level module handling
   - Phase 2.4: Return type validation
   - Phase 2.5: State initialization code generation
   - Phase 2.6: PyModule API methods
   - **Phase 2.6.3: PyAnyMethods for optimized class access** (NEW - uses Py_TYPE, no refcount)
   - Phase 2.7: Cleanup
3. ❌ Phase 3: Full testing and validation

See [PHASE2_DETAILED_IMPLEMENTATION_PLAN.md](../../PHASE2_DETAILED_IMPLEMENTATION_PLAN.md) for details.

## Next Steps

To make this example compile:

1. Implement `#[pymodule_state]` marker macro
2. Extend `#[pymodule(state = ...)]` parser
3. Implement state type auto-detection
4. Add `pass_module` support to `#[pyfunction]`
5. Add `module_state()` and `module_state_mut()` methods
6. Add PyType_GetModuleState FFI binding
7. Implement `PyAnyMethods` with `type_module_state()` and `type_module_state_mut()`
   - Uses Py_TYPE() directly for zero-cost access
   - Available on any `Bound<'_, PyAny>`
8. Generate state initialization code

Each step is tracked in the implementation plan.
