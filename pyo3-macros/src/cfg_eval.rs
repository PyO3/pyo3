//! Eagerly expand cfg/cfg_attr attributes.
//!
//! This works by duplicating code and then re-emitting that. If there are multiple attributes with
//! different predicates this process will be repeated again until there are no more cfg/cfg_attr
//! attributes left. See the tests at the end of this file for examples.

use quote::quote;
use quote::ToTokens;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::parse_quote;
use syn::punctuated::Punctuated;
use syn::visit;
use syn::visit::Visit;
use syn::visit_mut;
use syn::visit_mut::VisitMut;
use syn::{
    AttrStyle, Attribute, Fields, FieldsNamed, FieldsUnnamed, Item, ItemEnum, ItemStruct, LitBool,
    Meta, Token, Variant,
};

#[allow(clippy::large_enum_variant)]
#[derive(Eq, PartialEq, Clone, Debug)]
enum Predicate {
    Meta(Meta),
    Bool(LitBool),
}

impl TryFrom<&Attribute> for Predicate {
    type Error = ();
    fn try_from(attr: &Attribute) -> Result<Self, Self::Error> {
        if let Attribute {
            style: AttrStyle::Outer,
            meta: Meta::List(ml),
            ..
        } = &attr
        {
            if ml.path.is_ident("cfg") {
                if let Ok(meta) = ml.parse_args::<Meta>() {
                    return Ok(Predicate::Meta(meta));
                } else if let Ok(lb) = ml.parse_args::<LitBool>() {
                    return Ok(Predicate::Bool(lb));
                }
            }
        }
        Err(())
    }
}

impl ToTokens for Predicate {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Predicate::Meta(m) => m.to_tokens(tokens),
            Predicate::Bool(b) => b.to_tokens(tokens),
        }
    }
}

struct CfgAttr {
    predicate: Predicate,
    attrs: Punctuated<Meta, Token![,]>,
}

impl TryFrom<&Attribute> for CfgAttr {
    type Error = ();
    fn try_from(attr: &Attribute) -> Result<Self, Self::Error> {
        struct CfgAttrImpl {
            predicate: Predicate,
            attrs: Punctuated<Meta, Token![,]>,
        }

        impl Parse for CfgAttrImpl {
            fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
                Ok(Self {
                    predicate: if let Ok(meta) = input.parse::<Meta>() {
                        Predicate::Meta(meta)
                    } else if let Ok(lb) = input.parse::<LitBool>() {
                        Predicate::Bool(lb)
                    } else {
                        return Err(input.error("invalid cfg predicate"));
                    },
                    attrs: {
                        input.parse::<Token![,]>()?;
                        input.parse_terminated(Meta::parse, Token![,])?
                    },
                })
            }
        }

        if let Attribute {
            style: AttrStyle::Outer,
            meta: Meta::List(ml),
            ..
        } = &attr
        {
            if ml.path.is_ident("cfg_attr") {
                if let Ok(CfgAttrImpl { predicate, attrs }) = ml.parse_args::<CfgAttrImpl>() {
                    return Ok(CfgAttr { predicate, attrs });
                }
            }
        }
        Err(())
    }
}

/// Finds the first cfg/cfg_attr predicate.
///
/// The order is `syn::Visit` implementation detail, we should not depend on it.
#[derive(Default)]
struct PredicateFinder {
    predicate: Option<Predicate>,
}

impl PredicateFinder {
    fn find_first_in_attributes(&mut self, attrs: &[Attribute]) {
        for attr in attrs {
            if self.predicate.is_some() {
                break;
            }
            if let Ok(cfg) = TryInto::<Predicate>::try_into(attr) {
                self.predicate = Some(cfg);
            } else if let Ok(cfg_attr) = TryInto::<CfgAttr>::try_into(attr) {
                self.predicate = Some(cfg_attr.predicate);
            }
        }
    }
}

/// This doesn't override `visit_attribute` because we don't want to visit all attributes,
/// only the ones in places we explicitly support:
/// - struct fields
/// - enum variants
/// - enum variant fields
impl Visit<'_> for PredicateFinder {
    fn visit_item_struct(&mut self, i: &ItemStruct) {
        for field in &i.fields {
            self.find_first_in_attributes(&field.attrs);
        }
        visit::visit_item_struct(self, i);
    }
    fn visit_item_enum(&mut self, i: &ItemEnum) {
        for variant in &i.variants {
            self.find_first_in_attributes(&variant.attrs);
        }
        visit::visit_item_enum(self, i);
    }
    fn visit_variant(&mut self, i: &Variant) {
        for field in &i.fields {
            self.find_first_in_attributes(&field.attrs);
        }
        visit::visit_variant(self, i);
    }
}

struct CfgHoist<'p> {
    predicate: &'p Predicate,
    direction: Direction,
    progress: bool,
}

impl CfgHoist<'_> {
    fn edit_attributes(&mut self, attrs: &mut Vec<Attribute>) -> OwnerStatus {
        let old_attrs = std::mem::take(attrs).into_iter();
        for attr in old_attrs {
            if let Ok(predicate) = TryInto::<Predicate>::try_into(&attr) {
                if predicate == *self.predicate {
                    // We've found `#[cfg($predicate)]` on a field or variant.
                    self.progress = true;
                    let _: MustDiverge = match self.direction {
                        Direction::Forward => {
                            // #[cfg($predicate)] is hoisted on top of the item,
                            // so we don't preserve the attribute
                            continue;
                        }
                        Direction::Reverse => {
                            // #[cfg(not($predicate))] is hoisted on top of the item,
                            // so we remove the owner (and its attributes with it -
                            // we don't care about those).
                            return OwnerStatus::Remove;
                        }
                    };
                }
            } else if let Ok(cfg_attr) = TryInto::<CfgAttr>::try_into(&attr) {
                if cfg_attr.predicate == *self.predicate {
                    // We've found `#[cfg_attr($predicate, a, b, c, etc..)]` on a field or variant.
                    self.progress = true;
                    let _: MustDiverge = match self.direction {
                        Direction::Forward => {
                            // #[cfg($predicate)] is hoisted on top of the item,
                            // so we have to put `#[a] #[b] #[c] /* etc.. */` on the owner.
                            for meta in cfg_attr.attrs {
                                attrs.push(parse_quote!(#[#meta]));
                            }
                            continue;
                        }
                        Direction::Reverse => {
                            // #[cfg(not($predicate))] is hoisted on top of the item,
                            // so we just don't keep the cfg_attr.
                            continue;
                        }
                    };
                }
            }
            // Nothing we care about, preserve it.
            attrs.push(attr);
        }
        OwnerStatus::Keep
    }
}

impl VisitMut for CfgHoist<'_> {
    fn visit_item_struct_mut(&mut self, i: &mut ItemStruct) {
        if let Fields::Named(FieldsNamed { named: fields, .. })
        | Fields::Unnamed(FieldsUnnamed {
            unnamed: fields, ..
        }) = &mut i.fields
        {
            let mut old_fields = std::mem::take(fields);
            let trailing = old_fields.pop_punct();
            for mut field in old_fields {
                let keep = self.edit_attributes(&mut field.attrs);
                if matches!(keep, OwnerStatus::Keep) {
                    fields.push(field);
                }
            }
            if !fields.empty_or_trailing() {
                if let Some(punct) = trailing {
                    fields.push_punct(punct);
                }
            }
        }

        visit_mut::visit_item_struct_mut(self, i);
    }

    fn visit_item_enum_mut(&mut self, i: &mut ItemEnum) {
        let mut old_variants = std::mem::take(&mut i.variants);
        let trailing = old_variants.pop_punct();

        for mut variant in old_variants {
            let keep = self.edit_attributes(&mut variant.attrs);
            if matches!(keep, OwnerStatus::Keep) {
                i.variants.push(variant);
            }
        }
        if !i.variants.empty_or_trailing() {
            if let Some(punct) = trailing {
                i.variants.push_punct(punct);
            }
        }

        visit_mut::visit_item_enum_mut(self, i);
    }

    fn visit_variant_mut(&mut self, i: &mut Variant) {
        if let Fields::Named(FieldsNamed { named: fields, .. })
        | Fields::Unnamed(FieldsUnnamed {
            unnamed: fields, ..
        }) = &mut i.fields
        {
            let mut old_fields = std::mem::take(fields);
            let trailing = old_fields.pop_punct();
            for mut field in old_fields {
                let keep = self.edit_attributes(&mut field.attrs);
                if matches!(keep, OwnerStatus::Keep) {
                    fields.push(field);
                }
            }
            if !fields.empty_or_trailing() {
                if let Some(punct) = trailing {
                    fields.push_punct(punct);
                }
            }
        }

        visit_mut::visit_variant_mut(self, i);
    }
}

enum Direction {
    Forward,
    Reverse,
}

enum OwnerStatus {
    Keep,
    Remove,
}

enum MustDiverge {}

#[allow(clippy::large_enum_variant)]
pub enum CfgEvalResult {
    Ready(Item),
    Retry(proc_macro2::TokenStream),
}

pub fn cfg_eval_impl(this: proc_macro2::TokenStream, input: Item) -> CfgEvalResult {
    let predicate = &{
        let mut finder = PredicateFinder::default();
        finder.visit_item(&input);
        match finder.predicate {
            Some(predicate) => predicate,
            None => return CfgEvalResult::Ready(input),
        }
    };

    let mut forward = input.clone();

    let mut hoist = CfgHoist {
        predicate,
        direction: Direction::Forward,
        progress: false,
    };
    hoist.visit_item_mut(&mut forward);
    assert!(
        hoist.progress,
        "found cfg/cfg_attr predicates but was unable to expand any"
    );

    let mut reverse = input;
    let mut hoist = CfgHoist {
        predicate,
        direction: Direction::Reverse,
        progress: false,
    };
    hoist.visit_item_mut(&mut reverse);
    assert!(
        hoist.progress,
        "found cfg/cfg_attr predicates but was unable to expand any"
    );
    let retry = quote! {
        #[cfg(#predicate)]
        #this
        #forward

        #[cfg(not(#predicate))]
        #this
        #reverse

    };
    CfgEvalResult::Retry(retry)
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::ItemMod;

    #[test]
    fn find_cfg1() {
        let input: Item = parse_quote! {
            pub struct Thing(
                #[cfg_attr(feature = "feature_name", blah)] u8,
            );
        };
        let predicate = {
            let mut finder = PredicateFinder::default();
            finder.visit_item(&input);
            match finder.predicate {
                Some(predicate) => predicate,
                None => panic!(),
            }
        };
        assert_eq!(
            predicate,
            Predicate::Meta(parse_quote!(feature = "feature_name"))
        );
    }

    #[test]
    fn find_cfg2() {
        let input: Item = parse_quote! {
            pub struct Thing(
                #[cfg(true)] u8,
            );
        };
        let predicate = {
            let mut finder = PredicateFinder::default();
            finder.visit_item(&input);
            match finder.predicate {
                Some(predicate) => predicate,
                None => panic!(),
            }
        };
        assert_eq!(predicate, Predicate::Bool(parse_quote!(true)));
    }

    #[test]
    fn dont_find_cfg() {
        let input: Item = parse_quote! {
            #[cfg(what1)]
            pub struct Thing<#[cfg(what2)] GENERIC>(
                u8,
            );
        };

        let mut finder = PredicateFinder::default();
        finder.visit_item(&input);
        assert!(finder.predicate.is_none());
    }

    #[test]
    fn test_struct() {
        let this = quote! { #[macro_name(a,b,c,d)] };
        {
            let first: Item = parse_quote! {
                pub struct MyClass(
                    #[cfg_attr(not(feature = "feature_name"), helper_name(get, name = "raw"))] u8,
                    #[cfg_attr(all(feature = "feature_name", other_cfg), helper_name(get, name = "raw2"))] u8,
                );
            };
            let expected: ItemMod = parse_quote! {
                mod test {
                    #[cfg(not(feature = "feature_name"))]
                    #[macro_name(a, b, c, d)]
                    pub struct MyClass(
                        #[helper_name(get, name = "raw")] u8,
                        #[cfg_attr(all(feature = "feature_name", other_cfg), helper_name(get, name = "raw2"))] u8,
                    );

                    #[cfg(not(not(feature = "feature_name")))]
                    #[macro_name(a, b, c, d)]
                    pub struct MyClass(
                        u8,
                        #[cfg_attr(all(feature = "feature_name", other_cfg), helper_name(get, name = "raw2"))] u8,
                    );
                }
            };

            let CfgEvalResult::Retry(second) = cfg_eval_impl(this.clone(), first) else {
                panic!()
            };
            let second: ItemMod = parse_quote! {
                mod test {
                    #second
                }
            };
            assert_eq!(
                second,
                expected,
                "{} {}",
                second.to_token_stream(),
                expected.to_token_stream()
            );
        }

        {
            let second: Item = parse_quote! {
                pub struct MyClass(
                    u8,
                    #[cfg_attr(all(feature = "feature_name", other_cfg), helper_name(get, name = "raw2"))] u8,
                );
            };
            let expected: ItemMod = parse_quote! {
                mod test {
                    #[cfg(all(feature = "feature_name", other_cfg))]
                    #[macro_name(a, b, c, d)]
                    pub struct MyClass(u8, #[helper_name(get, name = "raw2")] u8,);

                    #[cfg(not(all(feature = "feature_name", other_cfg)))]
                    #[macro_name(a, b, c, d)]
                    pub struct MyClass(u8, u8,);
                }

            };
            let CfgEvalResult::Retry(third) = cfg_eval_impl(this.clone(), second) else {
                panic!("couldnt find cfgs to expand")
            };
            let third: ItemMod = parse_quote! {
                mod test {
                    #third
                }
            };
            assert_eq!(
                third,
                expected,
                "{} {}",
                third.to_token_stream(),
                expected.to_token_stream()
            );
        }

        {
            let third: Item = parse_quote! {
                pub struct MyClass(u8, #[helper_name(get, name = "raw2")] u8,);
            };
            let CfgEvalResult::Ready(fourth) = cfg_eval_impl(this, third.clone()) else {
                panic!("couldnt find cfgs to expand")
            };
            assert_eq!(
                fourth,
                third,
                "{} {}",
                fourth.to_token_stream(),
                third.to_token_stream()
            );
        }
    }

    #[test]
    fn test_enum() {
        let this = quote! { #[macro_name(a,b,c,d)] };
        {
            let first = parse_quote! {
                enum Shape {
                    Circle {
                        #[cfg_attr(cfg_name, helper_name = "what")]
                        radius: f64,
                    },
                    Rectangle {
                        width: f64,
                        #[cfg(cfg_name2)]
                        height: f64,
                    },
                    #[cfg(cfg_name2)]
                    RegularPolygon(u32, f64),
                    #[cfg(cfg_name)]
                    Nothing(),
                }
            };
            let expected: ItemMod = parse_quote! {
                mod test {
                    #[cfg(cfg_name2)]
                    #[macro_name(a, b, c, d)]
                    enum Shape {
                        Circle {
                            #[cfg_attr(cfg_name, helper_name = "what")]
                            radius: f64,
                        },
                        Rectangle {
                            width: f64,
                            height: f64,
                        },
                        RegularPolygon(u32, f64),
                        #[cfg(cfg_name)]
                        Nothing(),
                    }

                    #[cfg(not(cfg_name2))]
                    #[macro_name(a, b, c, d)]
                    enum Shape {
                        Circle {
                            #[cfg_attr(cfg_name, helper_name = "what")]
                            radius: f64,
                        },
                        Rectangle {
                            width: f64,
                        },
                        #[cfg(cfg_name)]
                        Nothing(),
                    }
                }
            };

            let CfgEvalResult::Retry(second) = cfg_eval_impl(this.clone(), first) else {
                panic!("couldnt find cfgs to expand")
            };
            let second: ItemMod = parse_quote! {
                mod test {
                    #second
                }
            };
            assert_eq!(
                second,
                expected,
                "{} {}",
                second.to_token_stream(),
                expected.to_token_stream()
            );
        }
    }
}
