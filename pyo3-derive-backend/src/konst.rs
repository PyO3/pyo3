use crate::pyfunction::parse_name_attribute;
use syn::ext::IdentExt;

#[derive(Clone, PartialEq, Debug)]
pub struct ConstSpec {
    pub is_class_attr: bool,
    pub python_name: syn::Ident,
}

impl ConstSpec {
    // For now, the only valid attribute is `#[classattr]`.
    pub fn parse(name: &syn::Ident, attrs: &mut Vec<syn::Attribute>) -> syn::Result<ConstSpec> {
        let mut new_attrs = Vec::new();
        let mut is_class_attr = false;

        for attr in attrs.iter() {
            if let syn::Meta::Path(name) = attr.parse_meta()? {
                if name.is_ident("classattr") {
                    is_class_attr = true;
                    continue;
                }
            }
            new_attrs.push(attr.clone());
        }

        attrs.clear();
        attrs.extend(new_attrs);

        Ok(ConstSpec {
            is_class_attr,
            python_name: parse_name_attribute(attrs)?.unwrap_or_else(|| name.unraw()),
        })
    }
}
