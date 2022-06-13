//! Generates static structures for runtime inspection of the Python objects
//!
//! The goal is to enable Rust code to implement features similar to Python's `dict(obj)`.
//! The generated structures are read-only.

use proc_macro2::{Ident, Literal, TokenStream, TokenTree};
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::Type;
use crate::method::FnType;
use crate::pyclass::{FieldPyO3Options, get_class_python_name};
use crate::PyClassArgs;
use crate::pymethod::PyMethod;

/// Extracts inspection information from the `#[pyclass]` macro.
///
/// Extracted information:
/// - Name of the class
pub(crate) fn generate_class_inspection(
    cls: &Ident,
    args: &PyClassArgs,
    field_options: &Vec<(&syn::Field, FieldPyO3Options)>,
) -> TokenStream {
    let ident_prefix = format_ident!("_path_{}", cls);
    let class_field_info = format_ident!("{}_struct_field_info", ident_prefix);
    let class_info = format_ident!("{}_struct_info", ident_prefix);

    let name = Literal::string(&*get_class_python_name(cls, args).to_string());

    quote! {
        const #class_field_info: [&'static _pyo3::inspect::fields::FieldInfo<'static>; 0] = [
            //TODO
        ];

        const #class_info: _pyo3::inspect::classes::ClassStructInfo<'static> = _pyo3::inspect::classes::ClassStructInfo {
            name: #name,
            base: ::std::option::Option::None, //TODO
            fields: &#class_field_info,
        };

        impl _pyo3::inspect::classes::InspectStruct<'static> for #cls {
            fn inspect_struct() -> &'static _pyo3::inspect::classes::ClassStructInfo<'static> {
                &#class_info
            }
        }
    }
}

/// Extracts information from an impl block annotated with `#[pymethods]`.
///
/// Currently, generating information from multiple impl blocks for the same class is not possible
/// (name collision in the generated structures and trait implementation), which makes inspection incompatible
/// with `multiple-pymethods`.
pub(crate) fn generate_impl_inspection(
    cls: &Type,
    fields: Vec<Ident>
) -> TokenStream {
    let ident_prefix = generate_unique_ident(cls, None);
    let fields_info = format_ident!("{}_fields_info", ident_prefix);

    let field_size = Literal::usize_suffixed(fields.len());

    let fields = fields.iter()
        .map(|field| quote!(&#field));

    quote! {
        const #fields_info: [&'static _pyo3::inspect::fields::FieldInfo<'static>; #field_size] = [
            #(#fields),*
        ];

        impl _pyo3::inspect::classes::InspectImpl<'static> for #cls {
            fn inspect_impl() -> &'static [&'static _pyo3::inspect::fields::FieldInfo<'static>] {
                &#fields_info
            }
        }
    }
}

/// Generates information from a field in a `#[pymethods]` block.
///
/// Extracted information:
/// - Field name
/// - Field kind (getter / setter / constructor / function / static method / …)
pub(crate) fn generate_fields_inspection(
    cls: &Type,
    field: &PyMethod<'_>
) -> (TokenStream, Ident) {
    let ident_prefix = generate_unique_ident(cls, Some(field.spec.name));

    let field_info_name = format_ident!("{}_info", ident_prefix);
    let field_args_name = format_ident!("{}_args", ident_prefix);

    let field_name = TokenTree::Literal(Literal::string(&*field.method_name));
    let field_kind = match &field.spec.tp {
        FnType::Getter(_) => quote!(_pyo3::inspect::fields::FieldKind::Getter),
        FnType::Setter(_) => quote!(_pyo3::inspect::fields::FieldKind::Setter),
        FnType::Fn(_) => quote!(_pyo3::inspect::fields::FieldKind::Function),
        FnType::FnNew => quote!(_pyo3::inspect::fields::FieldKind::New),
        FnType::FnClass => quote!(_pyo3::inspect::fields::FieldKind::ClassMethod),
        FnType::FnStatic => quote!(_pyo3::inspect::fields::FieldKind::StaticMethod),
        FnType::FnModule => todo!("FnModule is not currently supported"),
        FnType::ClassAttribute => quote!(_pyo3::inspect::fields::FieldKind::ClassAttribute),
    };

    let output = quote! {
        const #field_args_name: [_pyo3::inspect::fields::ArgumentInfo<'static>; 0] = []; //TODO

        const #field_info_name: _pyo3::inspect::fields::FieldInfo<'static> = _pyo3::inspect::fields::FieldInfo {
            name: #field_name,
            kind: #field_kind,
            py_type: ::std::option::Option::None, //TODO
            arguments: &#field_args_name,
        };
    };

    (output, field_info_name)
}

/// Generates a unique identifier based on a type and (optionally) a field.
///
/// For the same input values, the result should be the same output, and for different input values,
/// the output should be different. No other guarantees are made (do not try to parse it).
fn generate_unique_ident(class: &Type, field: Option<&Ident>) -> Ident {
    let span = if let Some(field) = field {
        field.span()
    } else {
        class.span()
    };

    let mut result = "".to_string();

    // Attempt to generate something unique for each type
    // Types that cannot be annotated with #[pyclass] are ignored
    match class {
        Type::Array(_) => unreachable!("Cannot generate a unique name for an array: {:?}", class),
        Type::BareFn(_) => unreachable!("Cannot generate a unique name for a function: {:?}", class),
        Type::Group(_) => unreachable!("Cannot generate a unique name for a group: {:?}", class),
        Type::ImplTrait(_) => unreachable!("Cannot generate a unique name for an impl trait: {:?}", class),
        Type::Infer(_) => unreachable!("Cannot generate a unique name for an inferred type: {:?}", class),
        Type::Macro(_) => unreachable!("Cannot generate a unique name for a macro: {:?}", class),
        Type::Never(_) => {
            result += "_never";
        },
        Type::Paren(_) => unreachable!("Cannot generate a unique name for a type in parenthesis: {:?}", class),
        Type::Path(path) => {
            result += "_path";
            for segment in &path.path.segments {
                result += "_";
                result += &*segment.ident.to_string();
            }
        }
        Type::Ptr(_) => unreachable!("Cannot generate a unique name for a pointer: {:?}", class),
        Type::Reference(_) => unreachable!("Cannot generate a unique name for a reference: {:?}", class),
        Type::Slice(_) => unreachable!("Cannot generate a unique name for a slice: {:?}", class),
        Type::TraitObject(_) => unreachable!("Cannot generate a unique name for a trait object: {:?}", class),
        Type::Tuple(_) => unreachable!("Cannot generate a unique name for a tuple: {:?}", class),
        _ => unreachable!("Cannot generate a unique name for an unknown type: {:?}", class),
    }

    if let Some(field) = field {
        result += "_";
        result += &*field.to_string()
    }

    Ident::new(&*result, span)
}
