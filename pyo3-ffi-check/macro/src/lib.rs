use std::{
    collections::HashSet,
    env, fs,
    path::{Path, PathBuf},
};

use proc_macro2::{Ident, Span, TokenStream, TokenTree};
use pyo3_build_config::PythonVersion;
use quote::quote;

/// Macro which expands to multiple macro calls, one per pyo3-ffi struct.
#[proc_macro]
pub fn for_all_structs(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: TokenStream = input.into();
    let mut input = input.into_iter();

    let macro_name = match input.next() {
        Some(TokenTree::Ident(i)) => i,
        _ => {
            return quote!(compile_error!(
                "for_all_structs!() takes only a single ident as input"
            ))
            .into()
        }
    };

    if input.next().is_some() {
        return quote!(compile_error!(
            "for_all_structs!() takes only a single ident as input"
        ))
        .into();
    }

    let doc_dir = get_doc_dir();
    let structs_glob = format!("{}/pyo3_ffi/struct.*.html", doc_dir.display());

    let mut output = TokenStream::new();

    for entry in glob::glob(&structs_glob).expect("Failed to read glob pattern") {
        let entry = entry
            .unwrap()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let struct_name = entry
            .strip_prefix("struct.")
            .unwrap()
            .strip_suffix(".html")
            .unwrap();

        if pyo3_build_config::get().abi.version < PythonVersion::PY315
            && struct_name == "PyBytesWriter"
        {
            // PyBytesWriter was added in Python 3.15
            continue;
        }

        let struct_ident = Ident::new(struct_name, Span::call_site());
        output.extend(quote!(#macro_name!(#struct_ident);));
    }

    if output.is_empty() {
        quote!(compile_error!(concat!(
            "No files found at `",
            #structs_glob,
            "`, try running `cargo doc -p pyo3-ffi` first."
        )))
    } else {
        output
    }
    .into()
}

fn get_doc_dir() -> PathBuf {
    PathBuf::from(env::var_os("PYO3_FFI_CHECK_DOC_DIR").unwrap())
}

/// Macro which expands to multiple macro calls, one per field in a pyo3-ffi
/// struct.
#[proc_macro]
pub fn for_all_fields(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: TokenStream = input.into();
    let mut input = input.into_iter();

    let struct_name = match input.next() {
        Some(TokenTree::Ident(i)) => i,
        _ => {
            return quote!(compile_error!(
                "for_all_fields!() takes exactly two idents as input"
            ))
            .into()
        }
    };

    match input.next() {
        Some(TokenTree::Punct(punct)) if punct.as_char() == ',' => (),
        _ => {
            return quote!(compile_error!(
                "for_all_fields!() takes exactly two idents as input"
            ))
            .into()
        }
    };

    let macro_name = match input.next() {
        Some(TokenTree::Ident(i)) => i,
        _ => {
            return quote!(compile_error!(
                "for_all_fields!() takes exactly two idents as input"
            ))
            .into()
        }
    };

    if input.next().is_some() {
        return quote!(compile_error!(
            "for_all_fields!() takes exactly two idents as input"
        ))
        .into();
    }

    let doc_dir = get_doc_dir();
    let pyo3_ffi_struct_file = doc_dir.join(format!("pyo3_ffi/struct.{}.html", struct_name));
    let mut bindgen_struct_file = doc_dir.join(format!("bindgen/struct.{}.html", struct_name));

    // might be a type alias
    if !bindgen_struct_file.exists() {
        let type_alias_file = doc_dir.join(format!("bindgen/type.{}.html", struct_name));
        if type_alias_file.exists() {
            bindgen_struct_file = type_alias_file;
        } else {
            let path = format!("{}", bindgen_struct_file.display());
            return quote!(compile_error!(concat!(
                "No file found at `",
                #path,
                "`, try running `cargo doc -p pyo3-ffi` first."
            )))
            .into();
        }
    }

    let pyo3_ffi_fields = get_fields_from_file(&pyo3_ffi_struct_file);
    let bindgen_fields = get_fields_from_file(&bindgen_struct_file);

    if pyo3_ffi_fields.is_empty() {
        // probably an opaque type on PyO3 side, skip
        return TokenStream::new().into();
    }

    let mut all_fields: HashSet<_> = pyo3_ffi_fields.into_iter().chain(bindgen_fields).collect();

    if struct_name == "PyMemberDef" {
        // bindgen picked `type_` as the field name to avoid the `type` keyword, but PyO3 uses `type_code`
        all_fields.remove("type_");
    } else if struct_name == "PyObject"
        && pyo3_build_config::get().abi.version >= PythonVersion::PY312
    {
        // bindgen picked `__bindgen_anon_1` as the field name for the anonymous union containing ob_refcnt,
        // PyO3 uses ob_refcnt directly
        all_fields.remove("__bindgen_anon_1");
    }

    let mut output = TokenStream::new();

    for field_name in all_fields {
        if field_name.starts_with("_") {
            // a private field - pyo3-ffi might have it, but it'll be inaccessible, can't do
            // offset of or similar checks on it, skip for now
            continue;
        }

        let field_ident = Ident::new(&field_name, Span::call_site());

        let bindgen_field_ident = if (pyo3_build_config::get().abi.version >= PythonVersion::PY312)
            && struct_name == "PyObject"
            && field_name == "ob_refcnt"
        {
            // PyObject since 3.12 implements ob_refcnt as a union; bindgen creates
            // an anonymous name for the field
            Ident::new("__bindgen_anon_1", Span::call_site())
        } else if struct_name == "PyMemberDef" && field_name == "type_code" {
            // the field name in the C API is `type`, but that's a keyword in Rust
            // so PyO3 picked type_code, bindgen picked type_
            Ident::new("type_", Span::call_site())
        } else {
            field_ident.clone()
        };

        output.extend(quote!(#macro_name!(#struct_name, #field_ident, #bindgen_field_ident);));
    }

    output.into()
}

fn get_fields_from_file(path: &Path) -> Vec<String> {
    let html = fs::read_to_string(path).unwrap();
    let html = scraper::Html::parse_document(&html);
    let selector = scraper::Selector::parse("span.structfield").unwrap();

    html.select(&selector)
        .map(|el| {
            el.value()
                .id()
                .unwrap()
                .strip_prefix("structfield.")
                .unwrap()
                .to_string()
        })
        .collect()
}
