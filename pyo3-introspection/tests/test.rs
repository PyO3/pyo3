use anyhow::Result;
use pyo3_introspection::introspect_cdylib;
use pyo3_introspection::model::{Class, Module};
use std::env;

#[test]
fn introspect_pytests() -> Result<()> {
    let binary = env::var_os("PYO3_PYTEST_LIB_PATH")
        .expect("The PYO3_PYTEST_LIB_PATH constant must be set and target the pyo3-pytests cdylib");
    let module = introspect_cdylib(binary, "pyo3_pytests")?;
    assert_eq!(
        module,
        Module {
            name: "pyo3_pytests".into(),
            modules: vec![
                Module {
                    name: "pyclasses".into(),
                    modules: vec![],
                    classes: vec![
                        Class {
                            name: "AssertingBaseClass".into()
                        },
                        Class {
                            name: "AssertingBaseClassGilRef".into()
                        },
                        Class {
                            name: "ClassWithoutConstructor".into()
                        },
                        Class {
                            name: "EmptyClass".into()
                        },
                        Class {
                            name: "PyClassIter".into()
                        }
                    ],
                    functions: vec![],
                },
                Module {
                    name: "pyfunctions".into(),
                    modules: vec![],
                    classes: vec![],
                    functions: vec![],
                }
            ],
            classes: vec![],
            functions: vec![],
        }
    );
    Ok(())
}
