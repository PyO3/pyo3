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
    let macro_name = match get_macro_name_from_input("for_all_structs", input) {
        Ok(name) => name,
        Err(err) => return err.into(),
    };

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

        if pyo3_build_config::get().version < PythonVersion::PY315 && struct_name == "PyBytesWriter"
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
    } else if struct_name == "PyObject" && pyo3_build_config::get().version >= PythonVersion::PY312
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

        let bindgen_field_ident = if (pyo3_build_config::get().version >= PythonVersion::PY312)
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

// C Macros are re-exported in pyo3-ffi as functions with the same name to get roughly equivalent semantics,
// these are excluded here.
const MACRO_EXCLUSIONS: &[&str] = &[
    "PyAnySet_Check",
    "PyAnySet_CheckExact",
    "PyAsyncGen_CheckExact",
    "PyBool_Check",
    "PyByteArray_Check",
    "PyByteArray_CheckExact",
    "PyBytes_AS_STRING",
    "PyBytes_Check",
    "PyBytes_CheckExact",
    "PyCFunction_Check",
    "PyCFunction_CheckExact",
    "PyCFunction_GET_CLASS",
    "PyCFunction_GET_FLAGS",
    "PyCFunction_GET_FUNCTION",
    "PyCFunction_GET_SELF",
    "PyCMethod_Check",
    "PyCMethod_CheckExact",
    "PyCallIter_Check",
    "PyCapsule_CheckExact",
    "PyCode_Check",
    "PyComplex_Check",
    "PyComplex_CheckExact",
    "PyContext_CheckExact",
    "PyContextToken_CheckExact",
    "PyContextVar_CheckExact",
    "PyCoro_CheckExact",
    "PyDate_Check",
    "PyDate_CheckExact",
    "PyDateTimeAPI",
    "PyDateTime_Check",
    "PyDateTime_CheckExact",
    "PyDateTime_DATE_GET_FOLD",
    "PyDateTime_DATE_GET_HOUR",
    "PyDateTime_DATE_GET_MICROSECOND",
    "PyDateTime_DATE_GET_MINUTE",
    "PyDateTime_DATE_GET_SECOND",
    "PyDateTime_DATE_GET_TZINFO",
    "PyDateTime_DELTA_GET_DAYS",
    "PyDateTime_DELTA_GET_MICROSECONDS",
    "PyDateTime_DELTA_GET_SECONDS",
    "PyDateTime_FromTimestamp",
    "PyDateTime_GET_DAY",
    "PyDateTime_GET_MONTH",
    "PyDateTime_GET_YEAR",
    "PyDateTime_IMPORT",
    "PyDateTime_TIME_GET_FOLD",
    "PyDateTime_TIME_GET_HOUR",
    "PyDateTime_TIME_GET_MICROSECOND",
    "PyDateTime_TIME_GET_MINUTE",
    "PyDateTime_TIME_GET_SECOND",
    "PyDateTime_TIME_GET_TZINFO",
    "PyDateTime_TimeZone_UTC",
    "PyDate_FromTimestamp",
    "PyDelta_Check",
    "PyDelta_CheckExact",
    "PyDict_Check",
    "PyDict_CheckExact",
    "PyDictItems_Check",
    "PyDictKeys_Check",
    "PyDictValues_Check",
    "PyDictViewSet_Check",
    "PyExceptionClass_Check",
    "PyExceptionInstance_Check",
    "PyExceptionInstance_Class",
    "PyFloat_AS_DOUBLE",
    "PyFloat_Check",
    "PyFloat_CheckExact",
    "PyFrame_BlockSetup",
    "PyFrame_Check",
    "PyFrameLocalsProxy_Check",
    "PyFrozenSet_Check",
    "PyFrozenSet_CheckExact",
    "PyFunction_Check",
    "PyGen_Check",
    "PyGen_CheckExact",
    "PyImport_ImportModuleEx",
    "PyList_Check",
    "PyList_CheckExact",
    "PyList_GET_ITEM",
    "PyList_GET_SIZE",
    "PyList_SET_ITEM",
    "PyLong_Check",
    "PyLong_CheckExact",
    "PyMapping_DelItem",
    "PyMapping_DelItemString",
    "PyMarshal_ReadLastObjectFromFile",
    "PyMarshal_ReadLongFromFile",
    "PyMarshal_ReadObjectFromFile",
    "PyMarshal_ReadObjectFromString",
    "PyMarshal_ReadShortFromFile",
    "PyMarshal_WriteLongToFile",
    "PyMarshal_WriteObjectToFile",
    "PyMarshal_WriteObjectToString",
    "PyMemoryView_Check",
    "PyModule_Check",
    "PyModule_CheckExact",
    "PyModule_Create",
    "PyModule_FromDefAndSpec",
    "PyObject_CallMethodNoArgs",
    "PyObject_CallMethodOneArg",
    "PyObject_GC_New",
    "PyObject_GC_NewVar",
    "PyObject_GC_Resize",
    "PyObject_New",
    "PyObject_NewVar",
    "PyObject_TypeCheck",
    "PyRange_Check",
    "PySeqIter_Check",
    "PySet_Check",
    "PySet_CheckExact",
    "PySet_GET_SIZE",
    "PySlice_Check",
    "PyStructSequence_GET_ITEM",
    "PyStructSequence_SET_ITEM",
    "PySys_AddWarnOption",
    "PySys_AddWarnOptionUnicode",
    "PySys_AddXOption",
    "PySys_HasWarnOptions",
    "PySys_SetPath",
    "PyTZInfo_Check",
    "PyTZInfo_CheckExact",
    "PyThreadState_GET",
    "PyTime_Check",
    "PyTime_CheckExact",
    "PyTimeZone_FromOffset",
    "PyTimeZone_FromOffsetAndName",
    "PyTraceBack_Check",
    "PyTuple_Check",
    "PyTuple_CheckExact",
    "PyTuple_GET_ITEM",
    "PyTuple_GET_SIZE",
    "PyTuple_SET_ITEM",
    "PyType_Check",
    "PyType_CheckExact",
    "PyType_FastSubclass",
    "PyType_HasFeature",
    "PyType_IS_GC",
    "PyUnicode_1BYTE_DATA",
    "PyUnicode_2BYTE_DATA",
    "PyUnicode_4BYTE_DATA",
    "PyUnicode_Check",
    "PyUnicode_CheckExact",
    "PyUnicode_ClearFreeList",
    "PyUnicode_Encode",
    "PyUnicode_EncodeASCII",
    "PyUnicode_EncodeCharmap",
    "PyUnicode_EncodeDecimal",
    "PyUnicode_EncodeLatin1",
    "PyUnicode_EncodeRawUnicodeEscape",
    "PyUnicode_EncodeUTF16",
    "PyUnicode_EncodeUTF32",
    "PyUnicode_EncodeUTF7",
    "PyUnicode_EncodeUTF8",
    "PyUnicode_EncodeUnicodeEscape",
    "PyUnicode_GET_LENGTH",
    "PyUnicode_IS_READY",
    "PyUnicode_READY",
    "PyUnicode_TransformDecimalToASCII",
    "PyUnicode_TranslateCharmap",
    "PyWeakref_Check",
    "PyWeakref_CheckProxy",
    "PyWeakref_CheckRef",
    "PyWeakref_CheckRefExact",
    "Py_CLEAR",
    "Py_CompileStringFlags",
    "Py_DECREF",
    "Py_Ellipsis",
    "Py_False",
    "Py_INCREF",
    "Py_IS_TYPE",
    "Py_None",
    "Py_NotImplemented",
    "Py_SIZE",
    "Py_True",
    "Py_XDECREF",
    "Py_XINCREF",
    // CPython deprecated these but the symbols still exist, pyo3-ffi will probably clean them up anyway
    "_PyCode_GetExtra",
    "_PyCode_SetExtra",
    "_PyEval_RequestCodeExtraIndex",
    // FIXME: probably outdated definitions that fail to build, need investigation,
    // temporarily here to make the build pass to get CI running
    "_PyFloat_CAST",
    "_PyObject_CallNoArg",
    "_PyObject_FastCall",
    "_PyObject_FastCallTstate",
    "_PyObject_MakeTpCall",
    "_PyObject_VectorcallTstate",
    "_PyRun_AnyFileObject",
    "_PyRun_InteractiveLoopObject",
    "_PyRun_SimpleFileObject",
    "_PySequence_IterSearch",
    "_PySet_NextEntry",
    "_PyUnicode_CheckConsistency",
    "_Py_CheckFunctionResult",
    "PyCode_New",
    "PyCode_NewWithPosOnlyArgs",
];

#[proc_macro]
pub fn for_all_functions(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let macro_name = match get_macro_name_from_input("for_all_functions", _input) {
        Ok(name) => name,
        Err(err) => return err.into(),
    };

    let doc_dir = get_doc_dir();
    let functions_glob = format!("{}/pyo3_ffi/fn.*.html", doc_dir.display());

    let mut output = TokenStream::new();

    for entry in glob::glob(&functions_glob).expect("Failed to read glob pattern") {
        let entry = entry.unwrap();

        let file_name = entry.file_name().unwrap().to_string_lossy().into_owned();
        let function_name = file_name
            .strip_prefix("fn.")
            .unwrap()
            .strip_suffix(".html")
            .unwrap();

        if MACRO_EXCLUSIONS.contains(&function_name) {
            continue;
        }

        if pyo3_build_config::get().implementation == pyo3_build_config::PythonImplementation::PyPy
        {
            // If the function doesn't exist in PyPy, for now we don't care:
            // - For PyO3 inline functions it's probably fine to include anyway
            // - For extern symbols - PyPy may add them in a future release
            let bingen_path = doc_dir.join(format!("bindgen/fn.{}.html", function_name));
            if !bingen_path.exists() {
                continue;
            }
        }

        let FunctionInfo {
            modifiers,
            arg_count,
            variadic,
        } = get_function_info(&function_name, &entry);

        let function_ident = Ident::new(function_name, Span::call_site());

        let arg_types = std::iter::repeat_n(quote!(_), arg_count);

        let vararg = if variadic { Some(quote!(, ...)) } else { None };

        if !modifiers.to_string().contains(r#"extern "C""#) {
            // if the function is not extern "C", it's a static inline function, pyo3-ffi uses the Rust abi,
            // bindgen uses the C abi still
            output
                .extend(quote!(#macro_name!(@inline #function_ident, (#(#arg_types),*) #vararg);));
        } else {
            output.extend(
                quote!(#macro_name!(#function_ident, [#modifiers] (#(#arg_types),* #vararg));),
            );
        }
    }

    output.into()
}

struct FunctionInfo {
    modifiers: TokenStream, // e.g. `unsafe extern "C"`, empty for no modifiers
    arg_count: usize,       // not including the "..." for variadic functions
    variadic: bool,
}

fn get_function_info(name: &str, path: &Path) -> FunctionInfo {
    let html = fs::read_to_string(path).unwrap();
    let html = scraper::Html::parse_document(&html);
    let selector = scraper::Selector::parse("pre.item-decl code").unwrap();

    let code_el = html.select(&selector).next().unwrap();
    let text = code_el.text().collect::<String>();

    // skip "pub " prefix
    let text = text.strip_prefix("pub ").unwrap();

    // find modifiers, e.g. `unsafe extern "C"`
    let left_paren = text.find('(').unwrap();
    let modifiers = text[..left_paren]
        .strip_suffix(name)
        .unwrap()
        .strip_suffix(" fn ")
        .unwrap()
        .parse()
        .unwrap();

    // Extract text between parens
    let start = left_paren + 1;

    // some functions might have function pointer arguments with their own parens,
    // so find the matching closing paren
    let mut depth = 1;
    let mut end = 0;
    let mut last_arg_start = 0;
    let mut arg_count = 0;
    let args_begin = text[start..].trim_start();
    for (i, c) in args_begin.char_indices() {
        match c {
            '(' => depth += 1,
            ')' if depth == 1 => {
                end = i;
                break;
            }
            ')' => {
                depth -= 1;
            }
            ',' if depth == 1 => {
                arg_count += 1;
                last_arg_start = i + 1;
            }
            _ => (),
        }
    }

    let args = &args_begin[..end].trim();
    let variadic = args.ends_with("...");

    if last_arg_start < end && !variadic && !args_begin[last_arg_start..end].trim_end().is_empty() {
        // additional argument after the last comma (i.e. no trailing comma)
        arg_count += 1;
    }

    FunctionInfo {
        modifiers,
        arg_count,
        variadic,
    }
}

fn get_macro_name_from_input(
    proc_macro: &str,
    input: proc_macro::TokenStream,
) -> Result<Ident, TokenStream> {
    let input: TokenStream = input.into();
    let mut input = input.into_iter();

    let macro_name = match input.next() {
        Some(TokenTree::Ident(i)) => i,
        _ => {
            let error_message = format!("{}!() takes only a single ident as input", proc_macro);
            return Err(quote!(compile_error!(#error_message)));
        }
    };

    if input.next().is_some() {
        let error_message = format!("{}!() takes only a single ident as input", proc_macro);
        return Err(quote!(compile_error!(#error_message)));
    }

    Ok(macro_name)
}
