# Sharing `#[pyclass]` types between multiple PyO3 extension modules

> [!WARNING]
> This is an advanced topic which requires reaching deeply into `unsafe` code.
> PyO3 does not have a stable API for doing this, and the approach documented here may break when updating to newer versions of PyO3.
> See [issue #1444](https://github.com/pyo3/pyo3/issues/1444) for ongoing discussion about this topic.

Some Python extension modules such as NumPy expose a C API which can be consumed by other extension modules to build functionality which directly exchanges native data without needing to go via Python objects.
This allows for higher performance than continually moving data in and out of Python objects.

It is a common request for PyO3 extension modules to be able to share `#[pyclass]` types (and other native data) across multiple crate/package boundaries in a similar fashion to NumPy. Because PyO3 extension modules are each compiled as individual `cdylib` binaries, they cannot depend on each other in the typical Rust way of adding a Cargo dependency (which typically just includes the dependency statically inside the final compiled binary).

Instead, the correct way to share `#[pyclass]` data between separate PyO3 extension modules is to use `#[repr(C)]` Rust types and an FFI-like API to exchange data across the `cdylib` boundaries. The solution in this subchapter will explain how to do this correctly.

## Quick summary

The solution which this subchapter describes shares types across extension module boundaries by defining three Rust crates (see also the [example project] on GitHub).

1. A crate `base-package-core` contains:
   1. `#[repr(C)]` Rust types and APIs to be shared with downstream extension modules (not `#[pyclass]` types or `#[pymethods]`).
   2. An API struct which contains function pointers to the API functions, version information, and any additional data.
   3. A global static variable which stores the API struct, and a function to initialise this at runtime by importing from the `base_package` Python package.
      The implementations of the APIs defined in step 1 will delegate to the function pointers in the API struct.
2. A crate `base-package` which is a PyO3 project containing `#[pyclass]` thin wrappers around the types exposed in `base-package-core`.
   1. Implement the API functionality defined in `base-package-core` and populate the API struct accordingly.
   2. As part of the Python module, export the API struct inside a Python `capsule` object. The initialisation function defined in `base-package-core` will import this capsule and populate the global static copy of the API struct backing the shared APIs.

3. A crate `derived-package` which is a PyO3 project depending on `base-package-core` (not `base-package`).
   1. As part of the Python module, import the API struct from the `capule` exposed by the `base_package` Python package and store it in `derived-package`'s copy of the global static variable defined in `base-package-core`.
   2. Use the APIs from `base-package-core` to implement functionality which directly uses the shared types.

The sections below go into more detail about how to implement various parts of this solution and why the solution is architected this way.

## Technical background & limitations

The Python ecosystem provides a well-established mechanism for sharing C APIs between native extension modules using [Python `capsule` objects](#capsules).
This avoids traditional "dynamic linking" between native extension modules, which would rely on system-specific behavior to locate and load `base_package` during import of `derived_package` and to resolve any errors in version mismatch between the two packages.

The `capsule` mechanism works as follows:

1. `base_package` defines a `#[repr(C)]` "API" struct which is exported in a Python `capsule` at runtime.
2. `derived_package` delegates to Python's extension loading mechanism to locate `base_package` and load the API from the capsule.
3. `derived_package` then contains its own logic to [check a compatible version](#api-versioning) of `base_package` was loaded; this is necessary to ensure safe exchange of the API struct.

When sharing types between multiple PyO3 extension modules through a `capsule`, the complexity arises from two main sources:

- Each Rust extension module may be built with completely separate Rust toolchains and build settings.
  - This means anything which is implementation-defined, such as the layout of `#[repr(Rust)] struct`s, the implementation of `std`, and even optimizations, might disagree between the two extension modules.
- Each Rust extension module contains a full statically-linked copy of its own dependencies.
  - Any `static` global variables which are compiled in the `base-package` will have a **completely independent** copy in the `derived-package`. This includes all `std` globals such as the [global allocator] and [panic hook].
  - Any dependency version mismatches might mean that bugs in dependencies of `base-package` may not reproduce in the copy in `derived-package`, (e.g. if the common dependency `base-package-core` depends on `foo` 0.1, it is possible `base-package` will compile with `foo` 0.1.1 and `base-package-core` will compile with `foo` 0.1.2).

Practically speaking, this introduces the following limitations on extension modules wanting to share data in this way:

- Extensions must take extreme care to ensure that only `#[repr(C)]` types are shared across the package boundary.

  In particular the default `#[repr(Rust)]` layout has no stability guarantee; _two extension modules sharing a `#[repr(Rust)]` data type is undefined behavior_.
  It is also very easy to accidentally share `#[repr(Rust)]` types, see the [safety note on the `PyCapsule` type documentation]({{#PYO3_DOCS_URL}}/pyo3/types/struct.PyCapsule.html) for cases to consider when sharing data.

  > [!WARNING]
  > Beware that PyO3's error type, `PyErr`, is not `#[repr(C)]` and cannot be shared across the package boundary
  > This is an easy mistake to make when exposing fallible APIs which cross the boundary.
  > There is a [later section on error handling](#error-handling) which suggests alternative strategies

- APIs which rely on global variables will not work as expected across the package boundary. For example:
  - The `#[global_allocator]` used by each extension will likely be different - each will need to ensure that any allocations are freed by the same allocator.
  - The `std::io` locks (e.g. for `stdout`) will not be shared, so concurrent output from the two extensions may interleave in unexpected ways.

- PyO3 currently stores `#[pyclass]` types as global variables in static storage in each `cdylib` crate which compiles them.
  This means that directly sharing a `#[pyclass]` type across multiple `cdylib` crates will currently silently create multiple distinct Python types.
  To avoid this, the shared types cannot be `#[pyclass]`, instead the package exporting the type to Python needs to make private `#[pyclass]` which wraps the shared type.
  PyO3 may remove this limitation in future.

## Using a capsule to create a shared API { #capsules }

Let's start with the example of NumPy as an extension which wants to offer a C API for other native extensions to consume.
The section above already established that Python native extensions can reuse data using the Python `capsule` type.
It is an opaque wrapper which can be used to exchange arbitrary native data between Python extension modules.
PyO3 provides the [`PyCapsule` type]({{#PYO3_DOCS_URL}}/pyo3/types/struct.PyCapsule.html) to create and consume capsules from Rust code.

NumPy creates a `capsule` which contains a pointer to the structure mapping the implementation of the ["NumPy C API"](https://numpy.org/doc/stable/reference/c-api/index.html).
While the [exact contents of this structure are generated](https://github.com/numpy/numpy/blob/40a1b0283beacb0d723f51e2046c8dc049da6eee/numpy/_core/code_generators/generate_numpy_api.py#L202-L210), the resulting API structure looks something like the following C code:

```C
// The API is defined as a fully type-erased array of "void pointers".
//
// This copy of the array is not public API, but internal to the NumPy implementation.
void* PyArray_API[] = {
   // Some fields contain function pointers, type erased
   (void *) PyArray_GetNDArrayCVersion,
   // Some fields are empty
   NULL,
   // Some fields point to Python type objects
   (void *) &PyArray_Type,

   // ... the real API is a few hundred elements long. contents and length are version specific.
};
```

To consume this array from downstream C projects, NumPy also defines a C header file
which uses C macros to define the downstream API in terms of cast indexing into this API structure:

```C
static void* PyArray_API[] = NULL;

// cast function pointers back to their correct type
#define PyArray_GetNDArrayCVersion (*(unsigned int (*)(void))PyArray_API[0])

// cast type objects back to `PyTypeObject *`
#define PyArray_Type (* (PyTypeObject *)PyArray_API[2])

// downstream packages must call this function before using any of the other APIS
static int PyArray_ImportNumPyAPI(void)
{
    PyArray_API = /* ... */;
}

```

To expose an API from Rust code, we'll need to take a similar approach.
We have the choice of either matching NumPy and using an array of opaque pointers, or using a more typed API `struct` ([as long as it is `#[repr(C)]` - see the safety docs on the [`PyCapsule` type]({{#PYO3_DOCS_URL}}/pyo3/types/struct.PyCapsule.html)).
A typed `struct` helps to avoid mistakes in casting fields to the wrong type incorrectly, however additional care needs to be taken to ensure that the layout of the struct does not change incompatibly across non-breaking versions of the API.
This is discussed further in [the next section](#api-versioning).

The snippets below sketch out what the NumPy approach looks like in Rust, using either an array of opaque pointers or a typed API struct. In both cases the public API functions are thin wrappers around the function pointers in the API struct, to provide ergonomics similar to the C macros in the NumPy example.

{{#tabs }}
{{#tab name="Using a pointer array" }}

If using a pointer array, `base-package-core` will define the pointer array, `BaseApi`, which contains function pointers and other data as opaque `*mut c_void` pointers:

```rust
// The number of fields in the API will grow over time as future
// versions add more APIs.
const BASE_API_ENTRIES: usize = 2;

// The pointer array itself
#[repr(transparent)]
pub struct BaseApi([*mut c_void; BASE_API_ENTRIES]);

// SAFETY: BaseApi never changes once loaded, so it will be shared between threads.
// (Manual implementations necessary due to `*mut c_void` not being `Send` or `Sync`).
// 
// (This is likely not necessary if using the typed struct approach, as the compiler can see the real function pointers which are likely `Send` / `Sync`).
unsafe impl Send for BaseApi { }
unsafe impl Sync for BaseApi { }

/// Global variable which will be used by the API functions
static BASE_API: PyOnceLock<BaseApi> = PyOnceLock::new();

impl BaseApi {
    /// Internal method used by the public methods to read pointers from the API
    fn get(py: Python<'_>) -> &'static Self {
        BASE_API.get(py).expect("`base_package` not yet imported")
    }
}

/// Version identifiers for the API, returned by `get_api_version`.
#[repr(C)]
pub struct ApiVersion {
    /* details omitted for now, see below section regarding API versioning */
}

/// Downstream packages must call this method to initialise the library API before
/// calling other functions.
pub fn import_base_package(py: Python<'_>) -> PyResult<()> {
    BASE_API.get_or_try_init(py, || /* details of importing omitted for now, see future sections */)?;
}

// Public functions exported by `base_package-core` are thin wrappers around the function pointers
// in the API struct.

/// Returns the version of the `base_package` API loaded. `ApiVersion` is a `#[repr(C)]` struct
/// so can be safely shared across package boundaries.
#[inline]
pub fn get_api_version(py: Python<'_>) -> ApiVersion {
    // SAFETY: BASE_API slot 0 is known to be the `get_api_version` function (`base_package` will set it).
    let get_api_version: extern "C" fn() -> ApiVersion = unsafe { std::mem::transmute(BASE_API.get(py).0[0]) };
    get_api_version()
}
```

{{#endtab }}
{{#tab name="Using a typed struct" }}

If using a typed struct, `base-package-core` will define `BaseApi` as a `#[repr(C)]` struct with typed fields:

```rust
// The API struct #[repr(C)] to ensure a stable layout across package boundaries.
// IMPORTANT: all fields must also be `repr(C)`, function pointers must be `extern "C"`,
// and fields can only be added to the end of the struct in non-breaking versions of the API.
#[repr(C)]
pub struct BaseApi {
    get_api_version: extern "C" fn() -> ApiVersion,
}

/// Global variable which will be used by the API functions
static BASE_API: PyOnceLock<BaseApi> = PyOnceLock::new();

impl BaseApi {
    /// Internal method used by the public methods to read pointers from the API
    fn get(py: Python<'_>) -> &'static Self {
        BASE_API.get(py).expect("`base_package` not yet imported")
    }
}

/// Version identifiers for the API, returned by `get_api_version`.
#[repr(C)]
pub struct ApiVersion {
    /* details omitted for now, see below section regarding API versioning */
}

/// Downstream packages must call this method to initialise the library API before
/// calling other functions.
pub fn import_base_package(py: Python<'_>) -> PyResult<()> {
    BASE_API.get_or_try_init(py, || /* details of importing omitted for now, see future sections */)?;
}

// Public functions exported by `base_package-core` are thin wrappers around the function pointers
// in the API struct.

/// Returns the version of the `base_package` API loaded. `ApiVersion` is a `#[repr(C)]` struct
/// so can be safely shared across package boundaries.
#[inline]
pub fn get_api_version(py: Python<'_>) -> ApiVersion {
    (BASE_API.get(py).get_api_version)()
}
```

{{#endtab }}
{{#endtabs }}

The consumers of the API struct will be compiled against a specific version of the `base-package-core` crate.
It will only be safe to use the API struct if the version in the API struct matches the version expected by the consumer.
Regardless of the choice made, due to backwards compatibility, the API struct can only grow over time, except when the version signals a breaking change.

The [example project] demonstrates how to do this with Rust.

## API versioning

To safely consume the API struct from downstream packages it is first necessary to perform a version check.
This version check needs to establish that the ABI (Application Binary Interface), i.e. the layout of the API struct,
matches the expectations of the consumer.

This means that the `ApiVersion` type exposed by `base-package` in the example uses four fields: the three `major`, `minor`, and `patch` version fields from semver, plus an additional `abi_version` field which is only incremented for breaking changes to the ABI.
Having the `abi_version` field allows for consumers potentially be compatible even across semver-breaking versions of the API.
This means that e.g. `derived-package` compiled with version `0.0.3` of `base-package` could potentially be compatible with `base-package` version `0.0.2` if the API struct layout did not change between these versions, and the `abi_version` field was not incremented. 

To make the version check straightforward, it is recommended to place the version information at the start of the API struct.
This allows the consumer to first read the capsule data as an `ApiVersion` structure, and only if the version check passes, reinterpret the rest of the data as the full API struct.

To demonstrate, the following code shows the approximate implementation of the `import_base_package` function from the [example project], which performs the version check and if successful reads the API struct from the capsule:

```rust
#[repr(C)]
pub struct BaseApi {
    get_api_version: extern "C" fn() -> ApiVersion,
    // real code will contain additional fields to satisfy all functionality
}

/// Version identifiers for the API, returned by `get_api_version`.
#[repr(C)]
pub struct ApiVersion {
    major: u32,
    minor: u32,
    patch: u32,
    abi: u32,
}

/// Downstream packages must call this method to initialise the library API before
/// calling other functions.
pub fn import_base_package(py: Python<'_>) -> PyResult<()> {
    BASE_API.get_or_try_init(|| do_import(py))?;
}

fn do_import(py: Python<'_>) -> PyResult<BaseApi> {
    // First: import the capsule as a pointer to retrieve version information. It is necessary to validate the API version
    // before attempting to access any of the rest of the API.
    //
    // SAFETY: The function to get the version info is the first field in the API struct.
    let capsule_base = unsafe { PyCapsule::import::<extern "C" fn() -> ApiVersion>(py, c"base_package._BASE_API")? };
    
    // Read the version information via the function pointer.
    let versions = (*capsule_base)();

    // Use environment variables set by Cargo to validate the API is
    // compatible with the version of the base package that is currently running.
    let current_major: u32 = env!("CARGO_PKG_VERSION_MAJOR")
        .parse()
        .expect("invalid cargo package version");
    let current_minor: u32 = env!("CARGO_PKG_VERSION_MINOR")
        .parse()
        .expect("invalid cargo package version");

    // Critical: the ABI version must match exactly, otherwise the layout of the API struct
    // is not known by this consumer
    if versions.abi != BaseApi::CURRENT_ABI_VERSION {
        return Err(PyErr::new::<pyo3::exceptions::PyImportError, _>(format!(
            "base_package ABI version mismatch: expected {}, got {}",
            BaseApi::CURRENT_ABI_VERSION,
            versions.abi
        )));
    }

    // In this example, the consumer allows for newer versions of the API to be used, as long
    // as they had a compatible ABI version. Real projects may have different policies on breakage
    // and forwards/backwards compatibility they are prepared to maintain.
    if (versions.major, versions.minor) < (current_major, current_minor) {
        return Err(PyErr::new::<pyo3::exceptions::PyImportError, _>(format!(
            "base_package API version mismatch: expected at least {}.{}, got {}.{}",
            env!("CARGO_PKG_VERSION_MAJOR"),
            env!("CARGO_PKG_VERSION_MINOR"),
            versions.major,
            versions.minor
        )));
    }

    // SAFETY: The version fields have been validated, so it is now known it
    // is safe to cast the data to the known struct.
    // 
    // The capsule contains a pointer to the full API struct, so this cast is sound.
    let api = unsafe { NonNull::from_ref(capsule_base).cast::<BaseApi>().as_ref() };

    Ok(BaseApi {
        // all fields in the API are function pointers, so are copied trivially
        ..*api
    })
}
```

## Creating a shared `#[pyclass]` type

Once the version checks are complete and the API struct loading is established, this technique can be expanded to share `#[pyclass]` types across the package boundary.

For a type named `SharedType`, the steps to achieve this are as follows:

1. `base-package-core` defines a `#[repr(C)]` struct which contains the data to be shared across the boundary.

2. The `BaseApi` struct defined in the previous sections is extended to include functions to manipulate this struct (these functions will later be provided by `base-package`).

   At a minimum, this will probably include:
  
   - `get_shared_type: extern "C" fn() -> Py<PyType>` - a function to get the `#[pyclass]` Python type object for `SharedType`.
  
   - `create_shared_type: unsafe extern "C" fn(SharedType) -> Option<Py<SharedType>>` - a function to create a new instance of the `SharedType` struct and return it as a Python object.
  
     This function is `unsafe` because the caller must ensure that the thread is attached to the interpreter (`Python<'py>` is a zero-sized type and not FFI-safe).
  
     The return type on this function is wrapped in `Option` to allow for failure - see [the error handling section](#error-handling) for more details.
  
   - `cast_shared_type: for<'a> extern "C" fn(Borrowed<'a, '_, SharedType>) -> &'a SharedType` - a function to extract a reference to the `SharedType` Rust struct from inside a Python object.

3. With the API struct extended to include these functions, the `base-package-core` crate can now implement PyO3 traits for `SharedType` in terms of those functions.
  
   The crucial traits are:

   - `PyTypeInfo` - the `get_type` function can delegate to the `get_shared_type` function pointer in the API struct.
  
   - `IntoPyPyObject` - the `into_pyobject` function can delegate to the `create_shared_type` function pointer in the API struct.
  
   - `FromPyObject<'_>` - the `extract` function can delegate to the `cast_shared_type` function pointer in the API struct.

4. `base-package` implements a `#[pyclass]` which is a thin wrapper around the `SharedType` struct, defining its Python functionality.

5. `base-package` implements the functions defined in the API struct to manipulate the `SharedType`, and populates the API struct with pointers to these functions as part of creating the `capsule`.

6. `derived-package` uses the existing `import_base_package()` function to load the API struct, and then can interact with it as a Python type via PyO3's smart pointers.

The [example project] demonstrates how to do this with a `Series` type implementing a "mini DataFrame API", to show how to use these stages to perform real work.

## Practical considerations for sharing data across the package boundary

As a reminder, there are two key restrictions to data which is shared across the package boundary:
- Only types with a stable layout, such as `#[repr(C)]` types, can be shared.
- Global variables are not shared across the boundary, and in particular for data sharing, the `#[global_allocator]` is not shared, so data allocated by one package must be freed by the same package.

This means that all Rust standard library types, such as `Vec`, `String`, and `Box`, cannot be shared across the boundary.
Even Rust tuples do not have a stable layout and cannot be shared.

There [is a proposal for a `#[repr(crabi)]` ABI](https://github.com/rust-lang/rust/issues/111423) which would define stable layouts for many Rust types, which would make it easier to share data.
This would still not solve types which contain heap allocations.

For now, a practical solution is to use the [`abi_stable`] crate, which provides many equivalents to Rust standard library types with stable internal layouts.
It uses vtables to allow for heap-allocated data to be shared, automatically ensuring the same allocator is used to free data as was used to allocate it.
This is the solution used in the [example project] to share `Vec`-like and `String`-like data across the boundary.

Using vtables introduces overhead (e.g. prevents inlining), however this is a necessary consequence of the limitations of sharing data across the package boundary.

### Error handling across the boundary { #error-handling }

Similar to many other types, PyO3's `PyErr` type is is not currently `#[repr(C)]`, so cannot be shared across the package boundary.

The simplest approach to handling errors across the boundary is to use `Option` return types in the API struct, and to return `None` on error. This is the approach taken in the [example project].

The downside of this approach is that `?` does not trivially work in the implementation of the API functions.
The suggested strategy is:
- Inside the API function implementations, convert `PyResult` to `Option` by using `PyErr::restore` to write the error to the Python thread state, and returning `None` on error.
- The wrappers in `base-package-core` which delegate to the API functions can then convert the `Option` back to `PyResult` by using `PyErr::fetch` to read the error from the Python thread state.

## Wrap-up, limitations & future work

This subchapter has attempted to detail how to use `capsule` objects to create an API for sharing Rust data between multiple PyO3 extension modules.
While complex, this is a workable technique for projects which need to have this functionality.

PyO3 does not yet offer any particular support for this use case and will likely be unable to provide a fully safe API to achieve this while data exchange is limited to `#[repr(C)]` types with pitfalls such as duplicate global variables.
Over time PyO3 might accumulate utilities for common pieces of this process.

Some places where this process process easier in the future include:

### Better `#[pyclass]` support

At the moment the `base-package-core` Rust API cannot use PyO3's `#[pyclass]` macro, because of the global variable backing the `#[pyclass]` implementation being a problem when duplicated into `base-package` and `derived-package`.

A possible solution is that PyO3 could have an option like `#[pyclass(shareable)]`, which could automatically generate the kind of `PyTypeInfo` / `IntoPyObject` / `FromPyObject` implementations which need to be hand-rolled at present.

To make this work, PyO3 would probably need to support [module state](https://github.com/PyO3/pyo3/pull/5600), and require the `base-package` which exports the shared type to manually initialize it and store it in module state.

There are many open design questions about how to make that work elegantly.

### `#[repr(C)]` error handling

At present, PyO3's `PyErr` type is a complex internal state machine which allows for lazy creation of Python exceptions.
This was convenient in early implementations of PyO3 but carried internal complexity and overhead.

PyO3's APIs have been trending in recent years towards allowing Rust code to use Rust error types for full control of overhead, e.g. the `IntoPyObject` and `FromPyObject` traits have a `type Error`, and the `#[pyfunction]` macros accept any `Result<T, E>` as long as the error type implements a conversion to `PyErr`.

It is likely that PyO3 will eventually transition the `PyErr` type to be a thin wrapper around `Py<PyBaseException>`, which would allow it to have a stable layout and participate in error handling across the boundary.
This primarily requires consideration about how to nudge existing dependents of `PyErr`'s "lazy" internals towards better practices.

### Better support for stable-layout types

The restriction of needing `#[repr(C)]` types to achieve a stable ABI for data sharing creates a lot of friction at the boundary.
It can also have implications for both efficiency and implementation of the `base-package-core` types.

The author's experience is that the Rust compiler doesn't yet have a mechanism to comprehensively lint against accidentally sharing types with unstable layouts across the boundary.
There may be value in an upstream effort to implement this so that projects using this `capsule` mechanism can avoid easy mistakes.

Outside of the Rust project itself, [`abi_stable`] crate is the most complete solution currently available to have a convenient way to correctly create a stable ABI.
However, `abi_stable` also appears to be largely unmaintained, so users wanting to depend on its functionality may need to consider reviving or forking it.

Furthermore, PyO3 currently doesn't implement `FromPyObject` or `IntoPyObject` conversions for `abi_stable` types.
This makes working with these types somewhat awkward / inefficient at the Python boundary.
PyO3 _could_ add an optional feature for this, however given the API surface involved it would be better to resolve the question of maintenance of `abi_stable` before adding such functionality to PyO3.

[global allocator]: https://doc.rust-lang.org/stable/std/alloc/trait.GlobalAlloc.html
[panic hook]: https://doc.rust-lang.org/stable/std/panic/fn.set_hook.html
[example project]: https://github.com/davidhewitt/pyo3-1444-prototype
[`abi_stable`]: https://github.com/rodrimati1992/abi_stable_crates/
