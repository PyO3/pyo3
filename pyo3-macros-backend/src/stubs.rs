use syn::{GenericArgument, PathArguments, Type};

pub fn map_rust_type_to_python(rust: &Type) -> String {
    let rust_str = match rust {
        Type::Array(_) => todo!("Unknown how to map to Python: {:?}", rust),
        Type::BareFn(_) => todo!("Unknown how to map to Python: {:?}", rust),
        Type::Group(_) => todo!("Unknown how to map to Python: {:?}", rust),
        Type::ImplTrait(_) => todo!("Unknown how to map to Python: {:?}", rust),
        Type::Infer(_) => "None".to_string(),
        Type::Macro(_) => todo!("Unknown how to map to Python: {:?}", rust),
        Type::Never(_) => todo!("Unknown how to map to Python: {:?}", rust),
        Type::Paren(_) => todo!("Unknown how to map to Python: {:?}", rust),
        Type::Path(ref path) => {
            match &path.path.segments.last() {
                Some(ref segment) => {
                    let str = segment.ident.to_string();

                    match str.as_str() {
                        "Vec" => {
                            if let PathArguments::AngleBracketed(angle) = &segment.arguments {
                                let mut res = None;
                                for arg in &angle.args {
                                    if let GenericArgument::Type(tp) = arg {
                                        res = Some(format!("List[{}]", map_rust_type_to_python(tp)))
                                    }
                                }
                                res.unwrap_or("List[Any]".to_string())
                            } else {
                                "List[Any]".to_string()
                            }
                        }
                        "Option" => {
                            if let PathArguments::AngleBracketed(angle) = &segment.arguments {
                                let mut res = None;
                                for arg in &angle.args {
                                    if let GenericArgument::Type(tp) = arg {
                                        res = Some(format!("Optional[{}]", map_rust_type_to_python(tp)))
                                    }
                                }
                                res.unwrap_or("Optional[Any]".to_string())
                            } else {
                                "Optional[Any]".to_string()
                            }
                        }
                        "usize" | "isize" | "u32" | "u64" | "i32" | "i64" => "int".to_string(),
                        "f64" | "f32" => "float".to_string(),
                        "bool" => "bool".to_string(),
                        _ => format!("rust_{}", str),
                    }
                },
                _ => "Any".to_string()
            }
        },
        Type::Reference(_) => todo!("Unknown how to map to Python: {:?}", rust),
        Type::Slice(_) => todo!("Unknown how to map to Python: {:?}", rust),
        Type::TraitObject(_) => todo!("Unknown how to map to Python: {:?}", rust),
        Type::Tuple(_) => todo!("Unknown how to map to Python: {:?}", rust),
        Type::Verbatim(_) => todo!("Unknown how to map to Python: {:?}", rust),

        #[cfg_attr(test, deny(non_exhaustive_omitted_patterns))]
        _ => {
            "Any".to_string()
        }
    };

    rust_str
}
