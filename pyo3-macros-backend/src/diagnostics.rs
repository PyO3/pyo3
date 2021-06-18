use quote::ToTokens;
use syn::spanned::Spanned;

/// Returns a `syn::Error` if the struct has any generic parameters,
/// or its fields have any non-static lifetime parameters
pub fn check_pyclass_generics_error(class: &syn::ItemStruct) -> Result<(), syn::Error> {
    if !class.generics.params.is_empty() {
        let mut base_error = syn::Error::new(
            class.generics.span(),
            "#[pyclass] cannot have generic parameters",
        );
        if let Err(another) = check_pyclass_field_lifetimes(class) {
            base_error.combine(another);
        }

        Err(base_error)
    } else {
        check_pyclass_field_lifetimes(class)
    }
}

/// Returns a `syn::Error` if any field has any non-static lifetime parameters.
fn check_pyclass_field_lifetimes(class: &syn::ItemStruct) -> Result<(), syn::Error> {
    for field in &class.fields {
        let lifetime = if let syn::Type::Reference(typeref) = &(field.ty) {
            typeref
                .lifetime
                .as_ref()
                .map(|lifetime| lifetime.ident.to_string())
        } else {
            None
        };

        let type_name = if let syn::Type::Reference(typeref) = &(field.ty) {
            if let syn::Type::Path(typepath) = &*(typeref.elem) {
                let mut s = typepath.path.segments.to_token_stream().to_string();
                s.retain(|c| !c.is_whitespace());
                Some(s)
            } else {
                None
            }
        } else {
            None
        };

        if let (Some(lifetime), Some(type_name)) = (lifetime, type_name) {
            // a naive attempt to detect python types
            if lifetime != "static" && (type_name.starts_with("Py") || type_name.starts_with("pyo3::types::Py"))
                // avoid making this message for something already wrapped in `Py<...>` for some reason
                && !(type_name.starts_with("Py<") || type_name.starts_with("pyo3::types::Py<"))
            {
                // at this point is is known that the pyclass contains a non-static reference,
                // which is always a compile error because this is never allowed,
                // therefore it is fine to error ourselves
                let message = format!(
                    "#[pyclass] cannot contain borrowed references to other Python objects. 

The lifetime `'{lifetime}` represents the scope during which the GIL is held, therefore
the reference `&'{lifetime} {type_name}` is only valid until the GIL is released.
Consider storing an owned reference using the `Py<...>` container: `Py<{type_name}>`.

See https://pyo3.rs/main/doc/pyo3/struct.Py.html for more information.",
                    lifetime = lifetime,
                    type_name = type_name,
                );
                let e = syn::Error::new(field.span(), message);
                return Err(e);
            };
        }
    }
    Ok(())
}
