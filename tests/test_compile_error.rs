#![cfg(feature = "macros")]

#[cfg(not(target_arch = "wasm32"))] // Not possible to invoke compiler from wasm
#[test]
fn test_compile_errors() {
    let t = trybuild::TestCases::new();

    t.compile_fail("tests/ui/invalid_need_module_arg_position.rs");
    t.compile_fail("tests/ui/invalid_property_args.rs");
    t.compile_fail("tests/ui/invalid_proto_pymethods.rs");
    t.compile_fail("tests/ui/invalid_pyclass_args.rs");
    t.compile_fail("tests/ui/invalid_pyclass_enum.rs");
    t.compile_fail("tests/ui/invalid_pyclass_item.rs");
    t.compile_fail("tests/ui/invalid_pyfunction_signatures.rs");
    #[cfg(not(Py_LIMITED_API))]
    t.compile_fail("tests/ui/invalid_pymethods_buffer.rs");
    t.compile_fail("tests/ui/invalid_pymethod_names.rs");
    t.compile_fail("tests/ui/invalid_pymodule_args.rs");
    t.compile_fail("tests/ui/reject_generics.rs");
    t.compile_fail("tests/ui/deprecations.rs");
    t.compile_fail("tests/ui/invalid_closure.rs");
    t.compile_fail("tests/ui/pyclass_send.rs");
    t.compile_fail("tests/ui/invalid_argument_attributes.rs");
    t.compile_fail("tests/ui/invalid_frompy_derive.rs");
    t.compile_fail("tests/ui/static_ref.rs");
    t.compile_fail("tests/ui/wrong_aspyref_lifetimes.rs");
    t.compile_fail("tests/ui/invalid_pyfunctions.rs");
    t.compile_fail("tests/ui/invalid_pymethods.rs");
    #[cfg(Py_LIMITED_API)]
    t.compile_fail("tests/ui/abi3_nativetype_inheritance.rs");
    t.compile_fail("tests/ui/invalid_intern_arg.rs");
    t.compile_fail("tests/ui/invalid_frozen_pyclass_borrow.rs");
    t.compile_fail("tests/ui/invalid_pymethod_receiver.rs");
    t.compile_fail("tests/ui/missing_intopy.rs");
    t.compile_fail("tests/ui/invalid_result_conversion.rs");
    t.compile_fail("tests/ui/not_send.rs");
    t.compile_fail("tests/ui/not_send2.rs");
    t.compile_fail("tests/ui/get_set_all.rs");
    t.compile_fail("tests/ui/traverse.rs");
    t.compile_fail("tests/ui/invalid_intopydict.rs");
    t.compile_fail("tests/ui/invalid_intopydict.rs");
}
