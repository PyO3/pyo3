#![cfg(feature = "macros")]

use pyo3::prelude::*;
use pyo3::inspect::classes::{ClassInfo, ClassStructInfo, InspectClass};
use pyo3::inspect::fields::{ArgumentInfo, ArgumentKind, FieldInfo, FieldKind};
use pyo3::inspect::interface::InterfaceGenerator;
use pyo3::inspect::types::TypeInfo;

mod common;

#[test]
fn types() {
    assert_eq!("bool", format!("{}", <bool>::type_output()));
    assert_eq!("bool", format!("{}", <bool as IntoPy<_>>::type_output()));
    assert_eq!("bytes", format!("{}", <&[u8]>::type_output()));
    assert_eq!("str", format!("{}", <String>::type_output()));
    assert_eq!("str", format!("{}", <char>::type_output()));
    assert_eq!("Optional[str]", format!("{}", <Option<String>>::type_output()));
    assert_eq!("Simple", format!("{}", <&PyCell<Simple>>::type_input()));
}

//region Empty class

const EXPECTED_EMPTY: &str = "class Empty: ...\n";

#[test]
fn empty_manual() {
    let class = ClassInfo {
        class: &ClassStructInfo {
            name: "Empty",
            base: None,
            fields: &[],
        },
        fields: &[],
    };

    assert_eq!(EXPECTED_EMPTY, format!("{}", InterfaceGenerator::new(class)))
}

#[pyclass]
struct Empty {}

#[pymethods]
impl Empty {}

#[test]
fn empty_derived() {
    assert_eq!(EXPECTED_EMPTY, format!("{}", InterfaceGenerator::new(Empty::inspect())))
}

//endregion
//region Constructor

const EXPECTED_SIMPLE: &str = r#"class Simple:
    def __new__(cls) -> None: ...
    def plus_one(self, /, a: int) -> int: ...
"#;

#[test]
fn simple_manual() {
    let class = ClassInfo {
        class: &ClassStructInfo {
            name: "Simple",
            base: None,
            fields: &[],
        },
        fields: &[
            &FieldInfo {
                name: "__new__",
                kind: FieldKind::New,
                py_type: None,
                arguments: &[],
            },
            &FieldInfo {
                name: "plus_one",
                kind: FieldKind::Function,
                py_type: Some(|| TypeInfo::Builtin("int")),
                arguments: &[
                    ArgumentInfo {
                        name: "a",
                        kind: ArgumentKind::PositionOrKeyword,
                        py_type: Some(|| TypeInfo::Builtin("int")),
                        default_value: false,
                        is_modified: false,
                    }
                ],
            }
        ],
    };

    assert_eq!(EXPECTED_SIMPLE, format!("{}", InterfaceGenerator::new(class)))
}

#[pyclass]
#[derive(Clone)]
struct Simple {}

#[pymethods]
impl Simple {
    #[new]
    fn new() -> Self {
        Self {}
    }

    fn plus_one(&self, a: usize) -> usize {
        a + 1
    }
}

#[test]
fn simple_derived() {
    assert_eq!(EXPECTED_SIMPLE, format!("{}", InterfaceGenerator::new(Simple::inspect())))
}

//endregion
//region Complicated

const EXPECTED_COMPLICATED: &str = r#"class Complicated(Simple):
    @property
    def value(self) -> int: ...
    @value.setter
    def value(self, value: int) -> None: ...
    def __new__(cls, /, foo: str = ..., **parent: Any) -> None: ...
    @staticmethod
    def static(input: Complicated) -> Complicated: ...
    @classmethod
    def classmeth(cls, /, input: Union[Complicated, str, int]) -> Complicated: ...
    counter: int = ...
"#;

#[test]
fn complicated_manual() {
    let class = ClassInfo {
        class: &ClassStructInfo {
            name: "Complicated",
            base: Some("Simple"),
            fields: &[
                &FieldInfo {
                    name: "value",
                    kind: FieldKind::Getter,
                    py_type: Some(|| TypeInfo::Builtin("int")),
                    arguments: &[],
                },
                &FieldInfo {
                    name: "value",
                    kind: FieldKind::Setter,
                    py_type: Some(|| TypeInfo::None),
                    arguments: &[
                        ArgumentInfo {
                            name: "value",
                            kind: ArgumentKind::Position,
                            py_type: Some(|| TypeInfo::Builtin("int")),
                            default_value: false,
                            is_modified: false
                        }
                    ],
                }
            ],
        },
        fields: &[
            &FieldInfo {
                name: "__new__",
                kind: FieldKind::New,
                py_type: None,
                arguments: &[
                    ArgumentInfo {
                        name: "foo",
                        kind: ArgumentKind::PositionOrKeyword,
                        py_type: Some(|| TypeInfo::Builtin("str")),
                        default_value: true,
                        is_modified: false,
                    },
                    ArgumentInfo {
                        name: "parent",
                        kind: ArgumentKind::KeywordArg,
                        py_type: Some(|| TypeInfo::Any),
                        default_value: false,
                        is_modified: false,
                    }
                ],
            },
            &FieldInfo {
                name: "static",
                kind: FieldKind::StaticMethod,
                py_type: Some(|| TypeInfo::Class { module: None, name: "Complicated" }),
                arguments: &[
                    ArgumentInfo {
                        name: "input",
                        kind: ArgumentKind::PositionOrKeyword,
                        py_type: Some(|| TypeInfo::Class { module: None, name: "Complicated" }),
                        default_value: false,
                        is_modified: false,
                    }
                ],
            },
            &FieldInfo {
                name: "classmeth",
                kind: FieldKind::ClassMethod,
                py_type: Some(|| TypeInfo::Class { module: None, name: "Complicated" }),
                arguments: &[
                    ArgumentInfo {
                        name: "input",
                        kind: ArgumentKind::PositionOrKeyword,
                        py_type: Some(|| TypeInfo::Union(vec![TypeInfo::Class { module: None, name: "Complicated" }, TypeInfo::Builtin("str"), TypeInfo::Builtin("int")])),
                        default_value: false,
                        is_modified: false
                    }
                ]
            },
            &FieldInfo {
                name: "counter",
                kind: FieldKind::ClassAttribute,
                py_type: Some(|| TypeInfo::Builtin("int")),
                arguments: &[]
            }
        ],
    };

    assert_eq!(EXPECTED_COMPLICATED, format!("{}", InterfaceGenerator::new(class)))
}

//endregion
