use std::fmt::{Display, Formatter};

/// The various Python types handled by PyO3.
///
/// The current implementation is not able to generate custom generics.
/// All generic types can only be hardcoded in PyO3.
#[derive(Debug)]
pub enum TypeInfo {
    /// The type `typing.Any`, which represents a dynamically-typed value (unknown type).
    Any,

    /// The type `typing.None`.
    None,

    /// The type `typing.Callable`, which represents a function-like object.
    Callable(Option<Vec<TypeInfo>>, Box<TypeInfo>),

    /// The type `typing.NoReturn`, which represents a function that never returns.
    NoReturn,

    /// A tuple of any value and size: `typing.Tuple[...]`.
    AnyTuple,

    /// A tuple of the specified types.
    Tuple(Vec<TypeInfo>),

    /// A tuple of unknown size, in which all elements have the same type.
    UnsizedTuple(Box<TypeInfo>),

    /// A union of multiple types.
    Union(Vec<TypeInfo>),

    /// An optional value.
    Optional(Box<TypeInfo>),

    Dict(Box<TypeInfo>, Box<TypeInfo>),

    Mapping(Box<TypeInfo>, Box<TypeInfo>),

    List(Box<TypeInfo>),

    Set(Box<TypeInfo>),

    FrozenSet(Box<TypeInfo>),

    Sequence(Box<TypeInfo>),

    Iterable(Box<TypeInfo>),

    Iterator(Box<TypeInfo>),

    Builtin(&'static str),

    /// Any type that doesn't receive special treatment from PyO3.
    Class {
        module: Option<&'static str>,
        name: &'static str,
    },
}

impl TypeInfo {
    pub fn module_name(&self) -> Option<&'static str> {
        match self {
            TypeInfo::Class { module, .. } => { *module }
            TypeInfo::Builtin(_) => None,
            _ => Some("typing"),
        }
    }
}

impl Display for TypeInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeInfo::Any => write!(f, "Any"),
            TypeInfo::None => write!(f, "None"),
            TypeInfo::Callable(args, output) => {
                if let Some(args) = args {
                    write!(f, "Callable[[")?;
                    let mut is_first = true;
                    for arg in args {
                        if !is_first {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", arg)?;
                        is_first = false
                    }
                    write!(f, "], {}]", output)
                } else {
                    write!(f, "Callable[..., {}]", output)
                }
            }
            TypeInfo::NoReturn => write!(f, "NoReturn"),
            TypeInfo::AnyTuple => write!(f, "Tuple[...]"),
            TypeInfo::Tuple(args) => {
                write!(f, "Tuple[")?;
                let mut is_first = true;
                for arg in args {
                    if !is_first {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                    is_first = false;
                }
                if args.is_empty() {
                    write!(f, "()")?;
                }
                write!(f, "]")
            }
            TypeInfo::UnsizedTuple(arg) => write!(f, "Tuple[{arg}, ...]"),
            TypeInfo::Union(args) => {
                write!(f, "Union[")?;
                let mut is_first = true;
                for arg in args {
                    if !is_first {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                    is_first = false;
                }
                write!(f, "]")
            }
            TypeInfo::Optional(arg) => write!(f, "Optional[{arg}]"),
            TypeInfo::Dict(key, value) => write!(f, "Dict[{key}, {value}]"),
            TypeInfo::Mapping(key, value) => write!(f, "Mapping[{key}, {value}]"),
            TypeInfo::List(arg) => write!(f, "List[{arg}]"),
            TypeInfo::Set(arg) => write!(f, "Set[{arg}]"),
            TypeInfo::FrozenSet(arg) => write!(f, "FrozenSet[{arg}]"),
            TypeInfo::Sequence(arg) => write!(f, "Sequence[{arg}]"),
            TypeInfo::Iterator(arg) => write!(f, "Iterator[{arg}]"),
            TypeInfo::Iterable(arg) => write!(f, "Iterable[{arg}]"),
            TypeInfo::Builtin(name) => write!(f, "{}", name),
            TypeInfo::Class { name, .. } => write!(f, "{}", name),
        }
    }
}

#[cfg(test)]
#[test]
fn type_info_display() {
    // Ensure the type infos follow the syntax defined in PEP484
    assert_eq!("Any", format!("{}", TypeInfo::Any));
    assert_eq!("None", format!("{}", TypeInfo::None));
    assert_eq!("int", format!("{}", TypeInfo::Builtin("int")));
    assert_eq!("Callable[..., int]", format!("{}", TypeInfo::Callable(None, Box::new(TypeInfo::Builtin("int")))));
    assert_eq!("Callable[[int, bool], int]", format!("{}", TypeInfo::Callable(Some(vec![TypeInfo::Builtin("int"), TypeInfo::Builtin("bool")]), Box::new(TypeInfo::Builtin("int")))));
    assert_eq!("NoReturn", format!("{}", TypeInfo::NoReturn));
    assert_eq!("Tuple[...]", format!("{}", TypeInfo::AnyTuple));
    assert_eq!("Tuple[int, bool, int]", format!("{}", TypeInfo::Tuple(vec![TypeInfo::Builtin("int"), TypeInfo::Builtin("bool"), TypeInfo::Builtin("int")])));
    assert_eq!("Tuple[()]", format!("{}", TypeInfo::Tuple(vec![])));
    assert_eq!("Tuple[int, ...]", format!("{}", TypeInfo::UnsizedTuple(Box::new(TypeInfo::Builtin("int")))));
    assert_eq!("Union[int, bool]", format!("{}", TypeInfo::Union(vec![TypeInfo::Builtin("int"), TypeInfo::Builtin("bool")])));
    assert_eq!("Optional[int]", format!("{}", TypeInfo::Optional(Box::new(TypeInfo::Builtin("int")))));
    assert_eq!("Optional[Any]", format!("{}", TypeInfo::Optional(Box::new(TypeInfo::Any))));
    assert_eq!("Dict[str, int]", format!("{}", TypeInfo::Dict(Box::new(TypeInfo::Builtin("str")), Box::new(TypeInfo::Builtin("int")))));
    assert_eq!("Mapping[str, int]", format!("{}", TypeInfo::Mapping(Box::new(TypeInfo::Builtin("str")), Box::new(TypeInfo::Builtin("int")))));
    assert_eq!("List[str]", format!("{}", TypeInfo::List(Box::new(TypeInfo::Builtin("str")))));
    assert_eq!("Set[str]", format!("{}", TypeInfo::Set(Box::new(TypeInfo::Builtin("str")))));
    assert_eq!("FrozenSet[str]", format!("{}", TypeInfo::FrozenSet(Box::new(TypeInfo::Builtin("str")))));
    assert_eq!("Sequence[str]", format!("{}", TypeInfo::Sequence(Box::new(TypeInfo::Builtin("str")))));
    assert_eq!("Iterable[str]", format!("{}", TypeInfo::Iterable(Box::new(TypeInfo::Builtin("str")))));
    assert_eq!("Iterator[str]", format!("{}", TypeInfo::Iterator(Box::new(TypeInfo::Builtin("str")))));
    assert_eq!("MyClass", format!("{}", TypeInfo::Class { module: Some("whatever"), name: "MyClass" }));

    // Just to be sure, a complicated (real life!) example
    assert_eq!(
        "List[Callable[[Common, Common], List[List[Tuple[Common, Common]]]]]",
        format!("{}", TypeInfo::List(
            &TypeInfo::Callable(
                Some(vec![TypeInfo::Class { module: None, name: "Common" }, TypeInfo::Class { module: None, name: "Common" }]),
                &TypeInfo::List(
                    &TypeInfo::List(
                        &TypeInfo::Tuple(vec![TypeInfo::Class { module: None, name: "Common" }, TypeInfo::Class { module: None, name: "Common" }])
                    )
                ),
            ),
        )),
    );
}
