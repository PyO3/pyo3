#![cfg(feature = "macros")]

#[cfg(not(target_arch = "wasm32"))] // Not possible to invoke compiler from wasm
#[test]
fn test_compile_errors() {
    let t = trybuild::TestCases::new();

    t.compile_fail("tests/ui/invalid_property_args.rs");
    t.compile_fail("tests/ui/invalid_proto_pymethods.rs");
    t.compile_fail("tests/ui/invalid_pyclass_args.rs");
    t.compile_fail("tests/ui/invalid_pyclass_enum.rs");
    t.compile_fail("tests/ui/invalid_pyclass_item.rs");
    t.compile_fail("tests/ui/invalid_pyfunction_signatures.rs");
    t.compile_fail("tests/ui/invalid_pyfunction_definition.rs");
    #[cfg(any(not(Py_LIMITED_API), Py_3_11))]
    t.compile_fail("tests/ui/invalid_pymethods_buffer.rs");
    // The output is not stable across abi3 / not abi3 and features
    #[cfg(all(not(Py_LIMITED_API), feature = "full"))]
    t.compile_fail("tests/ui/invalid_pymethods_duplicates.rs");
    t.compile_fail("tests/ui/invalid_pymethod_enum.rs");
    t.compile_fail("tests/ui/invalid_pymethod_names.rs");
    t.compile_fail("tests/ui/invalid_pymodule_args.rs");
    t.compile_fail("tests/ui/invalid_pycallargs.rs");
    t.compile_fail("tests/ui/reject_generics.rs");
    t.compile_fail("tests/ui/invalid_closure.rs");
    t.compile_fail("tests/ui/pyclass_send.rs");
    t.compile_fail("tests/ui/invalid_argument_attributes.rs");
    t.compile_fail("tests/ui/invalid_intopy_derive.rs");
    #[cfg(not(windows))]
    t.compile_fail("tests/ui/invalid_intopy_with.rs");
    t.compile_fail("tests/ui/invalid_frompy_derive.rs");
    t.compile_fail("tests/ui/static_ref.rs");
    t.compile_fail("tests/ui/wrong_aspyref_lifetimes.rs");
    #[cfg(not(any(feature = "uuid")))]
    t.compile_fail("tests/ui/invalid_pyfunctions.rs");
    #[cfg(not(any(feature = "hashbrown", feature = "indexmap")))]
    t.compile_fail("tests/ui/invalid_pymethods.rs");
    // output changes with async feature
    #[cfg(all(Py_LIMITED_API, feature = "experimental-async"))]
    t.compile_fail("tests/ui/abi3_nativetype_inheritance.rs");
    t.compile_fail("tests/ui/invalid_intern_arg.rs");
    t.compile_fail("tests/ui/invalid_frozen_pyclass_borrow.rs");
    #[cfg(not(any(feature = "hashbrown", feature = "indexmap")))]
    t.compile_fail("tests/ui/invalid_pymethod_receiver.rs");
    t.compile_fail("tests/ui/missing_intopy.rs");
    // adding extra error conversion impls changes the output
    #[cfg(not(any(windows, feature = "eyre", feature = "anyhow", Py_LIMITED_API)))]
    t.compile_fail("tests/ui/invalid_result_conversion.rs");
    t.compile_fail("tests/ui/not_send.rs");
    t.compile_fail("tests/ui/not_send2.rs");
    t.compile_fail("tests/ui/get_set_all.rs");
    t.compile_fail("tests/ui/traverse.rs");
    t.compile_fail("tests/ui/invalid_pymodule_in_root.rs");
    t.compile_fail("tests/ui/invalid_pymodule_glob.rs");
    t.compile_fail("tests/ui/invalid_pymodule_trait.rs");
    t.compile_fail("tests/ui/invalid_pymodule_two_pymodule_init.rs");
    #[cfg(feature = "experimental-async")]
    #[cfg(any(not(Py_LIMITED_API), Py_3_10))] // to avoid PyFunctionArgument for &str
    t.compile_fail("tests/ui/invalid_cancel_handle.rs");
    t.pass("tests/ui/pymodule_missing_docs.rs");
    #[cfg(not(any(Py_LIMITED_API, feature = "experimental-inspect")))]
    t.pass("tests/ui/forbid_unsafe.rs");
    #[cfg(all(Py_LIMITED_API, not(feature = "experimental-async")))]
    // output changes with async feature
    t.compile_fail("tests/ui/abi3_inheritance.rs");
    #[cfg(all(Py_LIMITED_API, not(Py_3_9)))]
    t.compile_fail("tests/ui/abi3_weakref.rs");
    #[cfg(all(Py_LIMITED_API, not(Py_3_9)))]
    t.compile_fail("tests/ui/abi3_dict.rs");
    #[cfg(not(feature = "experimental-inspect"))]
    t.compile_fail("tests/ui/duplicate_pymodule_submodule.rs");
    #[cfg(all(not(Py_LIMITED_API), Py_3_11))]
    t.compile_fail("tests/ui/invalid_base_class.rs");
    t.pass("tests/ui/ambiguous_associated_items.rs");
    t.pass("tests/ui/pyclass_probe.rs");
    t.compile_fail("tests/ui/deprecated.rs");
}
