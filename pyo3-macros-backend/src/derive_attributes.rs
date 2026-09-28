use crate::attributes::{
    self, get_pyo3_options, CrateAttribute, DefaultAttribute, FromPyWithAttribute,
    IntoPyWithAttribute, RenameAllAttribute,
};
use proc_macro2::Span;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{parenthesized, Attribute, LitStr, Result, Token};

/// Attributes for deriving `FromPyObject`/`IntoPyObject` scoped on containers.
pub enum ContainerAttribute {
    /// Treat the Container as a Wrapper, operate directly on its field
    Transparent(attributes::kw::transparent),
    /// Force every field to be extracted from item of source Python object.
    ItemAll(attributes::kw::from_item_all),
    /// Change the name of an enum variant in the generated error message.
    ErrorAnnotation(LitStr),
    /// Change the path for the pyo3 crate
    Crate(CrateAttribute),
    /// Converts the field idents according to the [RenamingRule](attributes::RenamingRule) before extraction
    RenameAll(RenameAllAttribute),
}

impl Parse for ContainerAttribute {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(attributes::kw::transparent) {
            let kw: attributes::kw::transparent = input.parse()?;
            Ok(ContainerAttribute::Transparent(kw))
        } else if lookahead.peek(attributes::kw::from_item_all) {
            let kw: attributes::kw::from_item_all = input.parse()?;
            Ok(ContainerAttribute::ItemAll(kw))
        } else if lookahead.peek(attributes::kw::annotation) {
            let _: attributes::kw::annotation = input.parse()?;
            let _: Token![=] = input.parse()?;
            input.parse().map(ContainerAttribute::ErrorAnnotation)
        } else if lookahead.peek(Token![crate]) {
            input.parse().map(ContainerAttribute::Crate)
        } else if lookahead.peek(attributes::kw::rename_all) {
            input.parse().map(ContainerAttribute::RenameAll)
        } else {
            Err(lookahead.error())
        }
    }
}

#[derive(Default, Clone)]
pub struct ContainerAttributes {
    /// Treat the Container as a Wrapper, operate directly on its field
    pub transparent: Option<attributes::kw::transparent>,
    /// Force every field to be extracted from item of source Python object.
    pub from_item_all: Option<attributes::kw::from_item_all>,
    /// Change the name of an enum variant in the generated error message.
    pub annotation: Option<syn::LitStr>,
    /// Change the path for the pyo3 crate
    pub krate: Option<CrateAttribute>,
    /// Converts the field idents according to the [RenamingRule](attributes::RenamingRule) before extraction
    pub rename_all: Option<RenameAllAttribute>,
}

impl ContainerAttributes {
    pub fn from_attrs(attrs: &[Attribute]) -> Result<Self> {
        let mut options = ContainerAttributes::default();

        for attr in attrs {
            if let Some(pyo3_attrs) = get_pyo3_options(attr)? {
                pyo3_attrs
                    .into_iter()
                    .try_for_each(|opt| options.set_option(opt))?;
            }
        }
        Ok(options)
    }

    fn set_option(&mut self, option: ContainerAttribute) -> syn::Result<()> {
        macro_rules! set_option {
            ($key:ident) => {
                {
                    ensure_spanned!(
                        self.$key.is_none(),
                        $key.span() => concat!("`", stringify!($key), "` may only be specified once")
                    );
                    self.$key = Some($key);
                }
            };
        }

        match option {
            ContainerAttribute::Transparent(transparent) => set_option!(transparent),
            ContainerAttribute::ItemAll(from_item_all) => set_option!(from_item_all),
            ContainerAttribute::ErrorAnnotation(annotation) => set_option!(annotation),
            ContainerAttribute::Crate(krate) => set_option!(krate),
            ContainerAttribute::RenameAll(rename_all) => set_option!(rename_all),
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub enum FieldGetter {
    GetItem(attributes::kw::item, Option<syn::Lit>),
    GetAttr(attributes::kw::attribute, Option<syn::LitStr>),
}

impl FieldGetter {
    pub fn span(&self) -> Span {
        match self {
            FieldGetter::GetItem(item, _) => item.span,
            FieldGetter::GetAttr(attribute, _) => attribute.span,
        }
    }
}

pub enum FieldAttribute {
    Getter(FieldGetter),
    FromPyWith(FromPyWithAttribute),
    IntoPyWith(IntoPyWithAttribute),
    Default(DefaultAttribute),
}

impl Parse for FieldAttribute {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(attributes::kw::attribute) {
            let attr_kw: attributes::kw::attribute = input.parse()?;
            if input.peek(syn::token::Paren) {
                let content;
                let _ = parenthesized!(content in input);
                let attr_name: LitStr = content.parse()?;
                if !content.is_empty() {
                    return Err(content.error(
                        "expected at most one argument: `attribute` or `attribute(\"name\")`",
                    ));
                }
                ensure_spanned!(
                    !attr_name.value().is_empty(),
                    attr_name.span() => "attribute name cannot be empty"
                );
                Ok(Self::Getter(FieldGetter::GetAttr(attr_kw, Some(attr_name))))
            } else {
                Ok(Self::Getter(FieldGetter::GetAttr(attr_kw, None)))
            }
        } else if lookahead.peek(attributes::kw::item) {
            let item_kw: attributes::kw::item = input.parse()?;
            if input.peek(syn::token::Paren) {
                let content;
                let _ = parenthesized!(content in input);
                let key = content.parse()?;
                if !content.is_empty() {
                    return Err(
                        content.error("expected at most one argument: `item` or `item(key)`")
                    );
                }
                Ok(Self::Getter(FieldGetter::GetItem(item_kw, Some(key))))
            } else {
                Ok(Self::Getter(FieldGetter::GetItem(item_kw, None)))
            }
        } else if lookahead.peek(attributes::kw::from_py_with) {
            input.parse().map(Self::FromPyWith)
        } else if lookahead.peek(attributes::kw::into_py_with) {
            input.parse().map(FieldAttribute::IntoPyWith)
        } else if lookahead.peek(Token![default]) {
            input.parse().map(Self::Default)
        } else {
            Err(lookahead.error())
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct FieldAttributes {
    pub getter: Option<FieldGetter>,
    pub from_py_with: Option<FromPyWithAttribute>,
    pub into_py_with: Option<IntoPyWithAttribute>,
    pub default: Option<DefaultAttribute>,
}

impl FieldAttributes {
    /// Extract the field attributes.
    pub fn from_attrs(attrs: &[Attribute]) -> Result<Self> {
        let mut options = FieldAttributes::default();

        for attr in attrs {
            if let Some(pyo3_attrs) = get_pyo3_options(attr)? {
                pyo3_attrs
                    .into_iter()
                    .try_for_each(|opt| options.set_option(opt))?;
            }
        }
        Ok(options)
    }

    fn set_option(&mut self, option: FieldAttribute) -> syn::Result<()> {
        macro_rules! set_option {
            ($key:ident) => {
                set_option!($key, concat!("`", stringify!($key), "` may only be specified once"))
            };
            ($key:ident, $msg: expr) => {{
                ensure_spanned!(
                    self.$key.is_none(),
                    $key.span() => $msg
                );
                self.$key = Some($key);
            }}
        }

        match option {
            FieldAttribute::Getter(getter) => {
                set_option!(getter, "only one of `attribute` or `item` can be provided")
            }
            FieldAttribute::FromPyWith(from_py_with) => set_option!(from_py_with),
            FieldAttribute::IntoPyWith(into_py_with) => set_option!(into_py_with),
            FieldAttribute::Default(default) => set_option!(default),
        }
        Ok(())
    }
}
