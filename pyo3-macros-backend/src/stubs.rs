use syn::Type;

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
                Some(ref segment) => segment.ident.to_string(),
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

    match rust_str.as_str() {
        "usize" | "isize" | "u32" | "u64" | "i32" | "i64" => "int".to_string(),
        "f64" | "f32" => "float".to_string(),
        "bool" => "bool".to_string(),
        "None" | "Any" => rust_str,
        _ => format!("rust_{}", rust_str)
    }
}
