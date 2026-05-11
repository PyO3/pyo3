use std::{
    collections::HashSet,
    env, fs,
    path::{Path, PathBuf},
    sync::LazyLock,
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

    let structs_glob = format!("{}/pyo3_ffi/struct.*.html", DOC_DIR.display());

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

static DOC_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| PathBuf::from(env::var_os("PYO3_FFI_CHECK_DOC_DIR").unwrap()));

static BINDGEN_FUNCTION_NAMES: LazyLock<HashSet<String>> = LazyLock::new(|| {
    // parse all the function names from the bindgen index file
    let index_file = DOC_DIR.join("bindgen/index.html");

    // the functions are in `a` elements with class "fn", and the full path is in the
    // `title` attribute
    let html = fs::read_to_string(index_file).unwrap();
    let html = scraper::Html::parse_document(&html);
    let selector = scraper::Selector::parse("a.fn").unwrap();

    html.select(&selector)
        .map(|el| {
            el.value()
                .attr("title")
                .unwrap()
                .rsplit_once("::")
                .unwrap()
                .1
                .to_string()
        })
        .collect()
});

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

    let pyo3_ffi_struct_file = DOC_DIR.join(format!("pyo3_ffi/struct.{}.html", struct_name));
    let mut bindgen_struct_file = DOC_DIR.join(format!("bindgen/struct.{}.html", struct_name));

    // might be a type alias
    if !bindgen_struct_file.exists() {
        let type_alias_file = DOC_DIR.join(format!("bindgen/type.{}.html", struct_name));
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
// bindgen doesn't generate symbols for C macros, we note their exclusions here.
//
// Each entry is `(name, cfg)` where `cfg` is a Rust `cfg` predicate (e.g. `"Py_3_8"`,
// `"not(Py_3_11)"`, `"all(PyPy, not(Py_3_10))"`) describing the configurations in which the
// symbol exists as a macro in C. An empty string means "all known configurations".
const MACRO_EXCLUSIONS: &[(&str, &str)] = &[
    // FIXME: for many of these `not(PyPy)` cases,
    // it seems that PyPy might actually offer symbols which PyO3
    // should be using rather than implementing inline functions
    ("PyAnySet_Check", "not(PyPy)"),
    ("PyAnySet_CheckExact", "not(PyPy)"),
    ("PyAsyncGen_CheckExact", ""),
    ("PyBool_Check", ""),
    ("PyByteArray_AS_STRING", ""),
    ("PyByteArray_GET_SIZE", ""),
    ("PyByteArray_Check", "not(PyPy)"),
    ("PyByteArray_CheckExact", "not(PyPy)"),
    ("PyBytes_AS_STRING", "not(PyPy)"),
    ("PyBytes_Check", ""),
    ("PyBytes_CheckExact", ""),
    ("PyCFunction_Check", "not(PyPy)"),
    ("PyCFunction_CheckExact", ""),
    ("PyCFunction_New", ""),
    ("PyCFunction_GET_CLASS", ""),
    ("PyCFunction_GET_FLAGS", ""),
    ("PyCFunction_GET_FUNCTION", ""),
    ("PyCFunction_GET_SELF", ""),
    ("PyCMethod_Check", ""),
    ("PyCMethod_CheckExact", ""),
    ("PyCallIter_Check", ""),
    ("PyCapsule_CheckExact", ""),
    ("PyCell_Check", ""),
    ("PyCode_Check", "not(PyPy)"),
    ("PyComplex_Check", "not(PyPy)"),
    ("PyComplex_CheckExact", "not(PyPy)"),
    ("PyContext_CheckExact", ""),
    ("PyContextToken_CheckExact", ""),
    ("PyContextVar_CheckExact", ""),
    ("PyCoro_CheckExact", "not(PyPy)"),
    ("PyDate_Check", "not(PyPy)"),
    ("PyDate_CheckExact", "not(PyPy)"),
    ("PyDate_FromDate", ""),
    ("PyDateTimeAPI", ""),
    ("PyDateTime_Check", "not(PyPy)"),
    ("PyDateTime_CheckExact", "not(PyPy)"),
    ("PyDateTime_DATE_GET_FOLD", ""),
    ("PyDateTime_DATE_GET_HOUR", "not(PyPy)"),
    ("PyDateTime_DATE_GET_MICROSECOND", "not(PyPy)"),
    ("PyDateTime_DATE_GET_MINUTE", "not(PyPy)"),
    ("PyDateTime_DATE_GET_SECOND", "not(PyPy)"),
    ("PyDateTime_DATE_GET_TZINFO", "not(PyPy)"),
    ("PyDateTime_DELTA_GET_DAYS", "not(PyPy)"),
    ("PyDateTime_DELTA_GET_MICROSECONDS", "not(PyPy)"),
    ("PyDateTime_DELTA_GET_SECONDS", "not(PyPy)"),
    ("PyDateTime_FromTimestamp", "not(PyPy)"),
    ("PyDateTime_FromDateAndTime", ""),
    ("PyDateTime_FromDateAndTimeAndFold", ""),
    ("PyDateTime_GET_DAY", "not(PyPy)"),
    ("PyDateTime_GET_MONTH", "not(PyPy)"),
    ("PyDateTime_GET_YEAR", "not(PyPy)"),
    ("PyDateTime_IMPORT", ""),
    ("PyDateTime_TIME_GET_FOLD", "not(PyPy)"),
    ("PyDateTime_TIME_GET_HOUR", "not(PyPy)"),
    ("PyDateTime_TIME_GET_MICROSECOND", "not(PyPy)"),
    ("PyDateTime_TIME_GET_MINUTE", "not(PyPy)"),
    ("PyDateTime_TIME_GET_SECOND", "not(PyPy)"),
    ("PyDateTime_TIME_GET_TZINFO", "not(PyPy)"),
    ("PyDateTime_TimeZone_UTC", ""),
    ("PyDate_FromTimestamp", "not(PyPy)"),
    ("PyDelta_Check", "not(PyPy)"),
    ("PyDelta_CheckExact", "not(PyPy)"),
    ("PyDelta_FromDSU", ""),
    ("PyDict_Check", ""),
    ("PyDict_CheckExact", ""),
    ("PyDictItems_Check", ""),
    ("PyDictKeys_Check", ""),
    ("PyDictValues_Check", ""),
    ("PyDictViewSet_Check", ""),
    ("PyExceptionClass_Check", ""),
    ("PyExceptionInstance_Check", ""),
    ("PyExceptionInstance_Class", "not(PyPy)"),
    ("PyEval_CallObject", "not(Py_3_13)"),
    ("PyFloat_AS_DOUBLE", "not(PyPy)"),
    ("PyFloat_Check", "not(PyPy)"),
    ("PyFloat_CheckExact", "not(PyPy)"),
    ("PyFrame_BlockSetup", ""),
    ("PyFrame_Check", ""),
    ("PyFrameLocalsProxy_Check", ""),
    ("PyFrozenSet_Check", "not(PyPy)"),
    ("PyFrozenSet_CheckExact", "not(PyPy)"),
    ("PyFunction_Check", "not(PyPy)"),
    ("PyGen_Check", "not(PyPy)"),
    ("PyGen_CheckExact", "not(PyPy)"),
    ("PyHeapType_GET_MEMBERS", "not(Py_3_11)"),
    ("PyImport_ImportModuleEx", ""),
    ("PyList_Check", ""),
    ("PyList_CheckExact", ""),
    ("PyList_GET_ITEM", "not(PyPy)"),
    ("PyList_GET_SIZE", "not(PyPy)"),
    ("PyList_SET_ITEM", "not(PyPy)"),
    ("PyLong_Check", ""),
    ("PyLong_CheckExact", ""),
    ("PyMapping_DelItem", ""),
    ("PyMapping_DelItemString", ""),
    ("PyMemoryView_Check", "not(PyPy)"),
    ("PyModule_Check", "not(PyPy)"),
    ("PyModule_CheckExact", "not(PyPy)"),
    ("PyModule_Create", ""),
    ("PyModule_FromDefAndSpec", "not(PyPy)"),
    ("PyObject_CallMethodNoArgs", ""),
    ("PyObject_CallMethodOneArg", ""),
    ("PyObject_CheckBuffer", "not(Py_3_9)"),
    (
        "PyObject_DelAttr",
        "any(all(not(PyPy), not(Py_3_13)), all(PyPy, Py_3_11))",
    ),
    (
        "PyObject_DelAttrString",
        "any(all(not(PyPy), not(Py_3_13)), all(PyPy, Py_3_11))",
    ),
    ("PyObject_GC_New", ""),
    ("PyObject_GC_NewVar", ""),
    ("PyObject_GC_Resize", ""),
    ("PyObject_IS_GC", "not(Py_3_9)"),
    ("PyObject_New", ""),
    ("PyObject_NewVar", ""),
    ("PyObject_TypeCheck", ""),
    ("PyParser_SimpleParseFile", ""),
    ("PyParser_SimpleParseString", ""),
    ("PyRange_Check", ""),
    ("PySeqIter_Check", ""),
    ("PySequence_Fast_GET_ITEM", ""),
    ("PySequence_Fast_GET_SIZE", ""),
    ("PySequence_Fast_ITEMS", ""),
    ("PySequence_ITEM", "not(PyPy)"),
    ("PySet_Check", "not(PyPy)"),
    ("PySet_CheckExact", "not(PyPy)"),
    ("PySet_GET_SIZE", ""),
    ("PySlice_Check", ""),
    ("PyStructSequence_GET_ITEM", ""),
    ("PyStructSequence_SET_ITEM", ""),
    ("PySys_AddWarnOption", ""),
    ("PySys_AddWarnOptionUnicode", ""),
    ("PySys_AddXOption", ""),
    ("PySys_HasWarnOptions", ""),
    ("PySys_SetPath", ""),
    ("PyTZInfo_Check", "not(PyPy)"),
    ("PyTZInfo_CheckExact", "not(PyPy)"),
    ("PyThreadState_GET", ""),
    ("PyTime_Check", "not(PyPy)"),
    ("PyTime_CheckExact", "not(PyPy)"),
    ("PyTime_FromTime", ""),
    ("PyTime_FromTimeAndFold", ""),
    ("PyTimeZone_FromOffset", ""),
    ("PyTimeZone_FromOffsetAndName", ""),
    ("PyTraceBack_Check", "not(PyPy)"),
    ("PyTuple_Check", ""),
    ("PyTuple_CheckExact", ""),
    ("PyTuple_GET_ITEM", ""),
    ("PyTuple_GET_SIZE", ""),
    ("PyTuple_SET_ITEM", ""),
    ("PyType_Check", ""),
    ("PyType_CheckExact", ""),
    ("PyType_FastSubclass", ""),
    ("PyType_HasFeature", ""),
    ("PyType_IS_GC", ""),
    ("PyType_SUPPORTS_WEAKREFS", "not(Py_3_11)"),
    ("PyUnicode_1BYTE_DATA", ""),
    ("PyUnicode_2BYTE_DATA", ""),
    ("PyUnicode_4BYTE_DATA", ""),
    ("PyUnicode_Check", "not(PyPy)"),
    ("PyUnicode_CheckExact", "not(PyPy)"),
    ("PyUnicode_ClearFreeList", ""),
    ("PyUnicode_DATA", "not(Py_3_14)"),
    ("PyUnicode_Encode", ""),
    ("PyUnicode_EncodeASCII", ""),
    ("PyUnicode_EncodeCharmap", ""),
    ("PyUnicode_EncodeDecimal", ""),
    ("PyUnicode_EncodeLatin1", ""),
    ("PyUnicode_EncodeRawUnicodeEscape", ""),
    ("PyUnicode_EncodeUTF16", ""),
    ("PyUnicode_EncodeUTF32", ""),
    ("PyUnicode_EncodeUTF7", ""),
    ("PyUnicode_EncodeUTF8", ""),
    ("PyUnicode_EncodeUnicodeEscape", ""),
    ("PyUnicode_GET_LENGTH", ""),
    ("PyUnicode_IS_ASCII", ""),
    ("PyUnicode_IS_COMPACT", ""),
    ("PyUnicode_IS_COMPACT_ASCII", ""),
    ("PyUnicode_IS_READY", ""),
    ("PyUnicode_KIND", "not(Py_3_14)"),
    ("PyUnicode_READY", ""),
    ("PyUnicode_TransformDecimalToASCII", ""),
    ("PyUnicode_TranslateCharmap", ""),
    ("PyWeakref_Check", "not(PyPy)"),
    ("PyWeakref_CheckProxy", "not(PyPy)"),
    ("PyWeakref_CheckRef", "not(PyPy)"),
    ("PyWeakref_CheckRefExact", "not(PyPy)"),
    ("PyVectorcall_NARGS", "not(Py_3_12)"),
    ("Py_CLEAR", ""),
    ("Py_CompileString", "not(Py_3_10)"),
    ("Py_CompileStringFlags", "not(PyPy)"),
    ("Py_DECREF", ""),
    ("Py_Ellipsis", ""),
    ("Py_False", ""),
    ("Py_GETENV", "not(Py_3_11)"),
    ("Py_INCREF", ""),
    ("Py_IS_TYPE", ""),
    ("Py_None", ""),
    ("Py_NotImplemented", ""),
    ("Py_REFCNT", "not(Py_3_14)"),
    ("Py_SIZE", ""),
    ("Py_True", ""),
    ("Py_TYPE", "not(Py_3_14)"),
    ("Py_XDECREF", ""),
    ("Py_XINCREF", ""),
    // These functions were only added in 3.10, but pyo3-ffi defines them for
    // all versions. Technically not macros but the machinery happens to work
    // the same way.
    ("Py_Is", "not(Py_3_10)"),
    ("Py_IsFalse", "not(Py_3_10)"),
    ("Py_IsTrue", "not(Py_3_10)"),
    ("Py_IsNone", "not(Py_3_10)"),
];

// TODO: probably need to clean these up
const EXCLUDED_SYMBOLS: &[&str] = &[
    // CPython deprecated these but the symbols still exist, pyo3-ffi will probably clean them up anyway
    "_PyCode_GetExtra",
    "_PyCode_SetExtra",
    "_PyEval_RequestCodeExtraIndex",
    // FIXME: probably outdated definitions that fail to build, need investigation,
    // temporarily here to make the build pass to get CI running
    "_PyFloat_CAST",
    "_PyObject_MakeTpCall",
    "_PyRun_AnyFileObject",
    "_PyRun_InteractiveLoopObject",
    "_PyRun_SimpleFileObject",
    "_PySequence_IterSearch",
    "_PySet_NextEntry",
    "_PyUnicode_CheckConsistency",
    "_Py_CheckFunctionResult",
    "PyCode_New",
    "PyCode_NewWithPosOnlyArgs",
    "PyCFunction_New",
    "PyObject_GET_WEAKREFS_LISTPTR",
    "PyFrame_BlockSetup",
    "PySys_AddWarnOption",
    "PySys_AddWarnOptionUnicode",
    "PySys_AddXOption",
    "PySys_HasWarnOptions",
    "PySys_SetPath",
    "PyUnicode_ClearFreeList",
    "PyUnicode_Encode",
    "PyUnicode_EncodeASCII",
    "PyUnicode_EncodeCharmap",
    "PyUnicode_EncodeDecimal",
    "PyUnicode_EncodeLatin1",
    "PyUnicode_EncodeRawUnicodeEscape",
    "PyUnicode_EncodeUTF7",
    "PyUnicode_EncodeUTF8",
    "PyUnicode_EncodeUTF16",
    "PyUnicode_EncodeUTF32",
    "PyUnicode_EncodeUnicodeEscape",
    "PyUnicode_TransformDecimalToASCII",
    "PyUnicode_TranslateCharmap",
    "_Py_HashBytes",
    // This symbol was not in headers but still public until Python 3.10,
    // should be able to remove this exclusion once support for 3.9 dropped
    "Py_GetArgcArgv",
    // pyo3-ffi defined these functions for 3.8 but they only exist for 3.9+
    "PyObject_CallOneArg",
    "PyObject_Vectorcall",
    "PyVectorcall_Function",
    "PyObject_VectorcallDict",
    // Needs fixing: since 3.9 it takes thread state as first argument
    "_PyEval_EvalFrameDefault",
    // Needs fixing: argument count is wrong on 3.13 against headers
    "_PyLong_AsByteArray",
    // CPython gates these on a HAVE_FORK macro, pyo3-ffi needs to replicate this?
    "PyOS_BeforeFork",
    "PyOS_AfterFork_Parent",
    "PyOS_AfterFork_Child",
    // Private symbols that pyo3-ffi should stop exporting
    "_PyUnicode_COMPACT_DATA",
    "_PyUnicode_NONCOMPACT_DATA",
    "_PyUnicode_Ready",
    // See https://github.com/python/cpython/pull/139166/changes#r3214904694
    "Py_IS_TYPE",
    "Py_SIZE",
];

#[proc_macro]
pub fn for_all_functions(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let macro_name = match get_macro_name_from_input("for_all_functions", _input) {
        Ok(name) => name,
        Err(err) => return err.into(),
    };

    let functions_glob = format!("{}/pyo3_ffi/fn.*.html", DOC_DIR.display());

    let mut output = TokenStream::new();

    for entry in glob::glob(&functions_glob).expect("Failed to read glob pattern") {
        let entry = entry.unwrap();

        let file_name = entry.file_name().unwrap().to_string_lossy().into_owned();

        let function_name = file_name
            .strip_prefix("fn.")
            .unwrap()
            .strip_suffix(".html")
            .unwrap();

        if EXCLUDED_SYMBOLS.contains(&function_name) {
            continue;
        }

        if pyo3_build_config::get().implementation == pyo3_build_config::PythonImplementation::PyPy
        {
            // If the function doesn't exist in PyPy, for now we don't care:
            // - For PyO3 inline functions it's probably fine to include anyway
            // - For extern symbols - PyPy may add them in a future release
            if !BINDGEN_FUNCTION_NAMES.contains(function_name) {
                continue;
            }
        }

        let FunctionInfo {
            modifiers,
            arg_count,
            variadic,
        } = match (function_name, get_function_info(function_name, &entry)) {
            (_, Ok(info)) => info,
            // In some cases symbols and macros differ only by case, which is a problem for case-insensitive filesystems.
            //
            // Hard-code workarounds for these cases here.
            //
            // Maybe one day the rustdoc json output can be used to avoid this problem
            ("Py_INCREF", Err(FunctionNameMismatch(e))) if e == "Py_IncRef" => FunctionInfo {
                modifiers: quote!(),
                arg_count: 1,
                variadic: false,
            },
            ("Py_IncRef", Err(FunctionNameMismatch(e))) if e == "Py_INCREF" => FunctionInfo {
                modifiers: quote!(extern "C"),
                arg_count: 1,
                variadic: false,
            },
            ("Py_DECREF", Err(FunctionNameMismatch(e))) if e == "Py_DecRef" => FunctionInfo {
                modifiers: quote!(),
                arg_count: 1,
                variadic: false,
            },
            ("Py_DecRef", Err(FunctionNameMismatch(e))) if e == "Py_DECREF" => FunctionInfo {
                modifiers: quote!(extern "C"),
                arg_count: 1,
                variadic: false,
            },
            ("PyThreadState_GET", Err(FunctionNameMismatch(e))) if e == "PyThreadState_Get" => {
                FunctionInfo {
                    modifiers: quote!(),
                    arg_count: 0,
                    variadic: false,
                }
            }
            ("PyThreadState_Get", Err(FunctionNameMismatch(e))) if e == "PyThreadState_GET" => {
                FunctionInfo {
                    modifiers: quote!(extern "C"),
                    arg_count: 0,
                    variadic: false,
                }
            }
            (function_name, Err(FunctionNameMismatch(unexpected))) => {
                let error_message = format!(
                    "parsed unexpected function declaration for `{function_name}`: {unexpected}",
                );
                output.extend(quote!(compile_error!(#error_message);));
                continue;
            }
        };

        let function_ident = Ident::new(function_name, Span::call_site());

        let arg_types = std::iter::repeat_n(quote!(_), arg_count);

        let vararg = if variadic { Some(quote!(, ...)) } else { None };

        // if the function is not extern "C":
        // - could be a Rust reimplementation of a C macro, or
        // - a static inline function in the C headers, pyo3-ffi uses the Rust abi, bindgen uses the C abi still
        //
        // The for_all_functions macro has two forms accordingly
        let (inline, modifiers) = if !modifiers.to_string().contains(r#"extern "C""#) {
            // inline form takes @inline at front, no modifiers
            (quote!( @inline ), quote!())
        } else {
            // regular form takes the modifiers
            (quote!(), quote!([#modifiers]))
        };

        // If a macro, then there will be no symbol from bindgen at all. To avoid
        // exclusions overreaching we have use cfg to document the expected macro range.
        let macro_exclusion_cfg: Option<TokenStream> = MACRO_EXCLUSIONS
            .iter()
            .find(|(n, _)| *n == function_name)
            .map(|(_, cfg)| if cfg.is_empty() { "all()" } else { *cfg })
            .map(|cfg| cfg.parse().expect("failed to parse macro exclusion cfg"));

        let has_symbol = BINDGEN_FUNCTION_NAMES.contains(function_name);

        match (macro_exclusion_cfg, has_symbol) {
            (Some(cfg), true) => {
                // emit an error if checking within the cfgs where a macro is expected
                let error_message = format!(
                    "`{function_name}` is in MACRO_EXCLUSIONS but a symbol was found in bindgen bindings, this likely means the exclusion cfg `{cfg}` is incorrect",
                );
                output.extend(quote!(#[cfg(#cfg)] compile_error!(#error_message);));
                // if not within the macro range, we found a symbol, this should be good
                output.extend(
                    quote!(#[cfg(not(#cfg))] #macro_name!(#inline #function_ident, #modifiers (#(#arg_types),* #vararg));),
                );
            }
            (Some(cfg), false) => {
                // emit an error if outside the cfgs where a macro is expected - should
                // be a static inline function in both pyo3-ffi and bindgen
                let error_message = format!(
                    "`{function_name}` is only expected to be a macro for versions `{cfg}`, but no symbol was found in bindgen bindings (means the symbol is probably still a macro in the headers)",
                );
                output.extend(quote!(#[cfg(not(#cfg))] compile_error!(#error_message);));
            }
            (None, true) => {
                // emit the comparison macro to check that the argument count matches
                output.extend(
                    quote!(#macro_name!(#inline #function_ident, #modifiers (#(#arg_types),* #vararg));),
                );
            }
            (None, false) => {
                // Not in MACRO_EXCLUSIONS, should have a symbol from bindgen
                let error_message = format!(
                    "`{function_name}` is not in MACRO_EXCLUSIONS and no symbol was found in bindgen bindings",
                );
                output.extend(quote!(compile_error!(#error_message);));
            }
        }
    }

    output.into()
}

struct FunctionInfo {
    modifiers: TokenStream, // e.g. `unsafe extern "C"`, empty for no modifiers
    arg_count: usize,       // not including the "..." for variadic functions
    variadic: bool,
}

// Error returned when the function definition does not match the expected name of the file
struct FunctionNameMismatch(String);

fn get_function_info(
    function_name: &str,
    path: &Path,
) -> Result<FunctionInfo, FunctionNameMismatch> {
    let html = fs::read_to_string(path).expect("file not found");
    let html = scraper::Html::parse_document(&html);
    let selector = scraper::Selector::parse("pre.item-decl code").unwrap();

    let code_el = html
        .select(&selector)
        .next()
        .expect("failed to find code element in function doc");
    let text = code_el.text().collect::<String>();

    static FUNCTION_DECL_REGEX: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r"^pub\s+(.*?)\sfn\s+([^(<]*)").unwrap());

    let captures = FUNCTION_DECL_REGEX
        .captures(&text)
        .expect("failed to parse function declaration with regex");

    // find modifiers, e.g. `unsafe extern "C"`
    let modifiers = captures.get(1).unwrap().as_str().parse().unwrap();

    // find function name
    let parsed_name = captures.get(2).unwrap().as_str().to_string();

    if parsed_name != function_name {
        return Err(FunctionNameMismatch(parsed_name));
    }

    let left_paren = text
        .find('(')
        .expect("function declaration should have opening paren");

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

    Ok(FunctionInfo {
        modifiers,
        arg_count,
        variadic,
    })
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
