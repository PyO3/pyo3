// Copyright (c) 2017-present PyO3 Project and Contributors
//! This crate declares only the proc macro attributes, as a crate defining proc macro attributes
//! must not contain any other public items.

#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use pyo3_macros_backend::{
    build_derive_from_pyobject, build_py_class, build_py_enum, build_py_function, build_py_methods,
    get_doc, process_functions_in_module, pymodule_function_impl, pymodule_module_impl,
    PyClassArgs, PyClassMethodsType, PyFunctionOptions,
    PyModuleOptions
};
use quote::quote;
use syn::{parse::Nothing, parse_macro_input};

/// A proc macro used to implement Python modules.
///
/// The name of the module will be taken from the function name, unless `#[pyo3(name = "my_name")]`
/// is also annotated on the function to override the name. **Important**: the module name should
/// match the `lib.name` setting in `Cargo.toml`, so that Python is able to import the module
/// without needing a custom import loader.
///
/// Functions annotated with `#[pymodule]` can also be annotated with the following:
///
/// |  Annotation  |  Description |
/// | :-  | :- |
/// | `#[pyo3(name = "...")]` | Defines the name of the module in Python. |
///
/// For more on creating Python modules see the [module section of the guide][1].
///
/// [1]: https://pyo3.rs/latest/module.html
#[proc_macro_attribute]
pub fn pymodule(args: TokenStream, input: TokenStream) -> TokenStream {
    parse_macro_input!(args as Nothing);

    if let Ok(module) = syn::parse(input.clone()) {
        pymodule_module_impl(module)
            .unwrap_or_compile_error()
            .into()
    } else {
        let mut ast = parse_macro_input!(input as syn::ItemFn);
        let options = match PyModuleOptions::from_attrs(&mut ast.attrs) {
            Ok(options) => options,
            Err(e) => return e.into_compile_error().into(),
        };

        if let Err(err) = process_functions_in_module(&options, &mut ast) {
            return err.into_compile_error().into();
        }

        let doc = get_doc(&ast.attrs, None);

        let expanded = pymodule_function_impl(&ast.sig.ident, options, doc, &ast.vis);
        quote!(
            #ast
            #expanded
        )
        .into()
    }
}

/// A proc macro used to implement Python's [dunder methods][1].
///
/// This attribute is required on blocks implementing [`PyObjectProtocol`][2],
/// [`PyNumberProtocol`][3], [`PyGCProtocol`][4] and [`PyIterProtocol`][5].
///
/// [1]: https://docs.python.org/3/reference/datamodel.html#special-method-names
/// [2]: ../class/basic/trait.PyObjectProtocol.html
/// [3]: ../class/number/trait.PyNumberProtocol.html
/// [4]: ../class/gc/trait.PyGCProtocol.html
/// [5]: ../class/iter/trait.PyIterProtocol.html
#[proc_macro_attribute]
#[cfg(feature = "pyproto")]
pub fn pyproto(_: TokenStream, input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as syn::ItemImpl);
    let expanded = pyo3_macros_backend::build_py_proto(&mut ast).unwrap_or_compile_error();

    quote!(
        #ast
        #expanded
    )
    .into()
}

/// A proc macro used to expose Rust structs and fieldless enums as Python objects.
///
#[cfg_attr(docsrs, cfg_attr(docsrs, doc = include_str!("../docs/pyclass_parameters.md")))]
///
/// For more on creating Python classes,
/// see the [class section of the guide][1].
///
/// [1]: https://pyo3.rs/latest/class.html
/// [params-1]: ../prelude/struct.PyAny.html
/// [params-2]: https://en.wikipedia.org/wiki/Free_list
/// [params-3]: std::marker::Send
/// [params-4]: std::rc::Rc
/// [params-5]: std::sync::Arc
/// [params-6]: https://docs.python.org/3/library/weakref.html
/// [params-mapping]: https://pyo3.rs/latest/class/protocols.html#mapping--sequence-types
#[proc_macro_attribute]
pub fn pyclass(attr: TokenStream, input: TokenStream) -> TokenStream {
    use syn::Item;
    let item = parse_macro_input!(input as Item);
    match item {
        Item::Struct(struct_) => pyclass_impl(attr, struct_, methods_type()),
        Item::Enum(enum_) => pyclass_enum_impl(attr, enum_, methods_type()),
        unsupported => {
            syn::Error::new_spanned(unsupported, "#[pyclass] only supports structs and enums.")
                .into_compile_error()
                .into()
        }
    }
}

/// A proc macro used to expose methods to Python.
///
/// Methods within a `#[pymethods]` block can be annotated with  as well as the following:
///
/// |  Annotation  |  Description |
/// | :-  | :- |
/// | [`#[new]`][4]  | Defines the class constructor, like Python's `__new__` method. |
/// | [`#[getter]`][5] and [`#[setter]`][5] | These define getters and setters, similar to Python's `@property` decorator. This is useful for getters/setters that require computation or side effects; if that is not the case consider using [`#[pyo3(get, set)]`][11] on the struct's field(s).|
/// | [`#[staticmethod]`][6]| Defines the method as a staticmethod, like Python's `@staticmethod` decorator.|
/// | [`#[classmethod]`][7]  | Defines the method as a classmethod, like Python's `@classmethod` decorator.|
/// | [`#[classattr]`][9]  | Defines a class variable. |
/// | [`#[args]`][10]  | Define a method's default arguments and allows the function to receive `*args` and `**kwargs`.  |
/// | <nobr>[`#[pyo3(<option> = <value>)`][pyo3-method-options]<nobr> | Any of the `#[pyo3]` options supported on [`macro@pyfunction`]. |
///
/// For more on creating class methods,
/// see the [class section of the guide][1].
///
/// If the [`multiple-pymethods`][2] feature is enabled, it is possible to implement
/// multiple `#[pymethods]` blocks for a single `#[pyclass]`.
/// This will add a transitive dependency on the [`inventory`][3] crate.
///
/// [1]: https://pyo3.rs/latest/class.html#instance-methods
/// [2]: https://pyo3.rs/latest/features.html#multiple-pymethods
/// [3]: https://docs.rs/inventory/
/// [4]: https://pyo3.rs/latest/class.html#constructor
/// [5]: https://pyo3.rs/latest/class.html#object-properties-using-getter-and-setter
/// [6]: https://pyo3.rs/latest/class.html#static-methods
/// [7]: https://pyo3.rs/latest/class.html#class-methods
/// [8]: https://pyo3.rs/latest/class.html#callable-objects
/// [9]: https://pyo3.rs/latest/class.html#class-attributes
/// [10]: https://pyo3.rs/latest/class.html#method-arguments
/// [11]: https://pyo3.rs/latest/class.html#object-properties-using-pyo3get-set
#[proc_macro_attribute]
pub fn pymethods(_: TokenStream, input: TokenStream) -> TokenStream {
    let methods_type = if cfg!(feature = "multiple-pymethods") {
        PyClassMethodsType::Inventory
    } else {
        PyClassMethodsType::Specialization
    };
    pymethods_impl(input, methods_type)
}

/// A proc macro used to expose Rust functions to Python.
///
/// Functions annotated with `#[pyfunction]` can also be annotated with the following `#[pyo3]`
/// options:
///
/// |  Annotation  |  Description |
/// | :-  | :- |
/// | `#[pyo3(name = "...")]` | Defines the name of the function in Python. |
/// | `#[pyo3(text_signature = "...")]` | Defines the `__text_signature__` attribute of the function in Python. |
/// | `#[pyo3(pass_module)]` | Passes the module containing the function as a `&PyModule` first argument to the function. |
///
/// For more on exposing functions see the [function section of the guide][1].
///
/// Due to technical limitations on how `#[pyfunction]` is implemented, a function marked
/// `#[pyfunction]` cannot have a module with the same name in the same scope. (The
/// `#[pyfunction]` implementation generates a hidden module with the same name containing
/// metadata about the function, which is used by `wrap_pyfunction!`).
///
/// [1]: https://pyo3.rs/latest/function.html
#[proc_macro_attribute]
pub fn pyfunction(attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as syn::ItemFn);
    let options = parse_macro_input!(attr as PyFunctionOptions);

    let expanded = build_py_function(&mut ast, options).unwrap_or_compile_error();

    quote!(
        #ast
        #expanded
    )
    .into()
}

#[proc_macro_derive(FromPyObject, attributes(pyo3))]
pub fn derive_from_py_object(item: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(item as syn::DeriveInput);
    let expanded = build_derive_from_pyobject(&ast).unwrap_or_compile_error();
    quote!(
        #expanded
    )
    .into()
}

fn pyclass_impl(
    attrs: TokenStream,
    mut ast: syn::ItemStruct,
    methods_type: PyClassMethodsType,
) -> TokenStream {
    let args = parse_macro_input!(attrs with PyClassArgs::parse_stuct_args);
    let expanded = build_py_class(&mut ast, args, methods_type).unwrap_or_compile_error();

    quote!(
        #ast
        #expanded
    )
    .into()
}

fn pyclass_enum_impl(
    attrs: TokenStream,
    mut ast: syn::ItemEnum,
    methods_type: PyClassMethodsType,
) -> TokenStream {
    let args = parse_macro_input!(attrs with PyClassArgs::parse_enum_args);
    let expanded = build_py_enum(&mut ast, args, methods_type).unwrap_or_compile_error();

    quote!(
        #ast
        #expanded
    )
    .into()
}

fn pymethods_impl(input: TokenStream, methods_type: PyClassMethodsType) -> TokenStream {
    let mut ast = parse_macro_input!(input as syn::ItemImpl);
    let expanded = build_py_methods(&mut ast, methods_type).unwrap_or_compile_error();

    quote!(
        #ast
        #expanded
    )
    .into()
}

fn methods_type() -> PyClassMethodsType {
    if cfg!(feature = "multiple-pymethods") {
        PyClassMethodsType::Inventory
    } else {
        PyClassMethodsType::Specialization
    }
}

trait UnwrapOrCompileError {
    fn unwrap_or_compile_error(self) -> TokenStream2;
}

impl UnwrapOrCompileError for syn::Result<TokenStream2> {
    fn unwrap_or_compile_error(self) -> TokenStream2 {
        self.unwrap_or_else(|e| e.into_compile_error())
    }
}
