//! This crate declares only the proc macro attributes, as a crate defining proc macro attributes
//! must not contain any other public items.

#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use pyo3_macros_backend::{
    build_derive_from_pyobject, build_derive_into_pydict, build_py_class, build_py_enum,
    build_py_function, build_py_methods, get_doc, parse_generics, process_functions_in_module,
    pymodule_impl, PyClassArgs, PyClassMethodsType, PyFunctionOptions, PyModuleOptions,
    Pyo3Collection,
};
use quote::{quote, ToTokens};
use syn::{parse::Nothing, parse_macro_input, DeriveInput};

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
/// Due to technical limitations on how `#[pymodule]` is implemented, a function marked
/// `#[pymodule]` cannot have a module with the same name in the same scope. (The
/// `#[pymodule]` implementation generates a hidden module with the same name containing
/// metadata about the module, which is used by `wrap_pymodule!`).
///
/// [1]: https://pyo3.rs/latest/module.html
#[proc_macro_attribute]
pub fn pymodule(args: TokenStream, input: TokenStream) -> TokenStream {
    parse_macro_input!(args as Nothing);

    let mut ast = parse_macro_input!(input as syn::ItemFn);
    let options = match PyModuleOptions::from_attrs(&mut ast.attrs) {
        Ok(options) => options,
        Err(e) => return e.into_compile_error().into(),
    };

    if let Err(err) = process_functions_in_module(&options, &mut ast) {
        return err.into_compile_error().into();
    }

    let doc = get_doc(&ast.attrs, None);

    let expanded = pymodule_impl(&ast.sig.ident, options, doc, &ast.vis);

    quote!(
        #ast
        #expanded
    )
    .into()
}

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
/// | [`#[args]`][10]  | Deprecated way to define a method's default arguments and allows the function to receive `*args` and `**kwargs`. Use `#[pyo3(signature = (...))]` instead. |
/// | <nobr>[`#[pyo3(<option> = <value>)`][pyo3-method-options]</nobr> | Any of the `#[pyo3]` options supported on [`macro@pyfunction`]. |
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
pub fn pymethods(attr: TokenStream, input: TokenStream) -> TokenStream {
    let methods_type = if cfg!(feature = "multiple-pymethods") {
        PyClassMethodsType::Inventory
    } else {
        PyClassMethodsType::Specialization
    };
    pymethods_impl(attr, input, methods_type)
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

#[proc_macro_derive(IntoPyDict, attributes(pyo3, into_py_dict_ignore))]
pub fn derive_into_pydict(item: TokenStream) -> TokenStream {
    let cloned = item.clone();
    let ast = parse_macro_input!(cloned as DeriveInput);
    let ident = ast.ident.into_token_stream();
    let clause_wrapped = ast.generics.where_clause.clone();
    let mut where_clause: TokenStream2 = TokenStream2::new();
    let generic_params: TokenStream2 = parse_generics(&ast.generics).parse().unwrap();
    let generics = ast.generics.into_token_stream();

    if let Some(clause) = clause_wrapped {
        where_clause = clause.into_token_stream();
    }
    let mut dict_fields: Pyo3Collection = Pyo3Collection(Vec::new());
    for token in item {
        let token_stream: syn::__private::TokenStream = token.into();
        dict_fields += parse_macro_input!(token_stream as Pyo3Collection);
    }
    let body: TokenStream2 = build_derive_into_pydict(dict_fields);
    let out = quote! {
        impl #generics IntoPyDict for #ident #generic_params  #where_clause {
            fn into_py_dict(self, py: pyo3::Python<'_>) -> &PyDict {
                #body
            }
        }
    };

    out.into()
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

fn pymethods_impl(
    attr: TokenStream,
    input: TokenStream,
    methods_type: PyClassMethodsType,
) -> TokenStream {
    let mut ast = parse_macro_input!(input as syn::ItemImpl);
    // Apply all options as a #[pyo3] attribute on the ItemImpl
    // e.g. #[pymethods(crate = "crate")] impl Foo { }
    // -> #[pyo3(crate = "crate")] impl Foo { }
    let attr: TokenStream2 = attr.into();
    ast.attrs.push(syn::parse_quote!( #[pyo3(#attr)] ));
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
