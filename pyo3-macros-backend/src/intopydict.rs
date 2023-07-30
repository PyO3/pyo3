use quote::{quote, TokenStreamExt};
use std::{collections::HashMap, ops::AddAssign};

use proc_macro2::{Span, TokenStream};
use syn::{
    parse::{Parse, ParseStream},
    DeriveInput, Error, Generics,
};

const COL_NAMES: [&str; 8] = [
    "BTreeSet",
    "BinaryHeap",
    "Vec",
    "HashSet",
    "LinkedList",
    "VecDeque",
    "BTreeMap",
    "HashMap",
];

#[derive(Debug, Clone)]
enum Pyo3Type {
    Primitive,
    NonPrimitive,
    CollectionSing(Box<crate::intopydict::Pyo3Type>),
    // Map(
    //     Box<crate::intopydict::Pyo3Type>,
    //     Box<crate::intopydict::Pyo3Type>,
    // ),
}

#[derive(Debug, Clone)]
pub struct Pyo3DictField {
    name: String,
    attr_type: Pyo3Type,
    attr_name: Option<String>,
}

impl Pyo3DictField {
    pub fn new(name: String, type_: &str, span: Span, attr_name: Option<String>) -> Self {
        Self {
            name,
            attr_type: Self::check_primitive(type_, span),
            attr_name,
        }
    }

    fn check_primitive(attr_type: &str, span: Span) -> Pyo3Type {
        for collection in COL_NAMES {
            if attr_type.starts_with(collection) {
                let attr_type = attr_type.replace('>', "");
                let attr_list: Vec<&str> = attr_type.split('<').collect();
                let out = Self::handle_collection(&attr_list, span);

                return out.unwrap();
            }
        }

        match attr_type {
            "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32" | "u64"
            | "u128" | "usize" | "f32" | "f64" | "char" | "bool" | "&str" | "String" => {
                Pyo3Type::Primitive
            }
            _ => Pyo3Type::NonPrimitive,
        }
    }

    fn handle_collection(attr_type: &[&str], span: Span) -> syn::Result<Pyo3Type> {
        match attr_type[0] {
            "BTreeSet" | "BinaryHeap" | "Vec" | "HashSet" | "LinkedList" | "VecDeque" => {
                Ok(Pyo3Type::CollectionSing(Box::new(
                    Self::handle_collection(&attr_type[1..], span).unwrap(),
                )))
            }
            "BTreeMap" | "HashMap" => {
                Err(Error::new(span, "Derive currently doesn't support map types. Please use a custom implementation for structs using a map type like HashMap or BTreeMap"))
            }
            "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32" | "u64"
            | "u128" | "usize" | "f32" | "f64" | "char" | "bool" | "&str" | "String" => {
                Ok(Pyo3Type::Primitive)
            }
            _ => Ok(Pyo3Type::NonPrimitive),
        }
    }
}

impl Parse for Pyo3Collection {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let tok_stream: TokenStream = input.parse()?;
        let binding = tok_stream
            .to_string()
            .as_str()
            .replace(|c| c == ' ' || c == '{' || c == '}', "");

        if !binding.contains(':') {
            return Ok(Pyo3Collection(Vec::new()));
        }

        let (name_map, tok_split) = split_struct(binding);

        let mut field_collection: Vec<Pyo3DictField> = Vec::new();

        for i in &tok_split {
            let tok_params_unparsed = &i.to_string();
            let tok_bind: Vec<&str> = tok_params_unparsed.split(':').collect();
            if tok_bind.len() == 2 {
                if let Some(val) = name_map.get(tok_bind[0]) {
                    field_collection.push(Pyo3DictField::new(
                        tok_bind[0].to_string(),
                        tok_bind[1],
                        input.span(),
                        Some(val.to_string()),
                    ));
                } else {
                    field_collection.push(Pyo3DictField::new(
                        tok_bind[0].to_string(),
                        tok_bind[1],
                        input.span(),
                        None,
                    ));
                }
            }
        }

        Ok(Pyo3Collection(field_collection))
    }
}

fn split_struct(binding: String) -> (HashMap<String, String>, Vec<String>) {
    let mut stack: Vec<char> = Vec::new();
    let mut tok_split: Vec<String> = Vec::new();
    let mut start = 0;
    let binding = binding.replace('\n', "");
    let mut name_map: HashMap<String, String> = HashMap::new();

    for (i, char_val) in binding.chars().enumerate() {
        if char_val == ',' && stack.is_empty() {
            if binding[start..i].starts_with('#') {
                let new_name = get_new_name(binding.clone(), start, i);
                let var_string = &binding[start..i].split(']').collect::<Vec<&str>>()[1];
                name_map.insert(
                    var_string.split(':').collect::<Vec<&str>>()[0].to_string(),
                    new_name,
                );
                tok_split.push(var_string.to_string());
            } else {
                tok_split.push(binding[start..i].to_string());
            }
            start = i + 1;
        } else if i == binding.len() - 1 {
            tok_split.push(binding[start..].to_string());
        }

        if char_val == '<' || char_val == '(' {
            stack.push(char_val);
        }

        if char_val == '>' || char_val == ')' {
            stack.pop();
        }
    }

    if !tok_split.is_empty() {
        let mut last = tok_split.last().unwrap().clone();
        for i in stack {
            last.push(i)
        }
        let len = tok_split.len();
        tok_split[len - 1] = last;
    }

    (name_map, tok_split)
}

fn get_new_name(binding: String, start: usize, i: usize) -> String {
    let fragments: Vec<&str> = binding[start..i].split("name=").collect();
    let mut quote_count = 0;
    let mut start = 0;
    for (j, char_val_inner) in fragments[1].chars().enumerate() {
        if char_val_inner == '"' {
            quote_count += 1;

            if quote_count == 1 {
                start = j + 1;
            }
        }

        if quote_count == 2 {
            return fragments[1][start..j].to_string();
        }
    }

    String::new()
}

#[derive(Debug, Clone)]
pub struct Pyo3Collection(pub Vec<Pyo3DictField>);

impl AddAssign for Pyo3Collection {
    fn add_assign(&mut self, rhs: Self) {
        self.0.extend(rhs.0);
    }
}

pub fn build_derive_into_pydict(dict_fields: Pyo3Collection) -> TokenStream {
    let mut body = quote! {
        let mut pydict = pyo3::types::PyDict::new(py);
    };

    for field in &dict_fields.0 {
        let ident: &String;
        if let Some(ref val) = field.attr_name {
            ident = val;
        } else {
            ident = &field.name;
        }
        let ident_tok: TokenStream = field.name.parse().unwrap();
        if !ident.is_empty() {
            match_tok(field, &mut body, ident, ident_tok);
        }
    }
    body.append_all(quote! {
        return pydict;
    });

    body
}

fn match_tok(
    field: &Pyo3DictField,
    body: &mut TokenStream,
    ident: &String,
    ident_tok: TokenStream,
) {
    match field.attr_type {
        Pyo3Type::Primitive => {
            body.append_all(quote! {
                pydict.set_item(#ident, self.#ident_tok).expect("Bad element in set_item");
            });
        }
        Pyo3Type::NonPrimitive => {
            body.append_all(quote! {
                pydict.set_item(#ident, self.#ident_tok.into_py_dict(py)).expect("Bad element in set_item");
            });
        }
        Pyo3Type::CollectionSing(ref collection) => {
            let non_class_ident = ident.replace('.', "_");
            body.append_all(handle_single_collection_code_gen(
                collection,
                &format!("self.{}", ident_tok),
                &non_class_ident,
                0,
            ));

            let ls_name: TokenStream = format!("pylist0{}", ident).parse().unwrap();
            body.append_all(quote! {
                pydict.set_item(#ident, #ls_name).expect("Bad element in set_item");
            });
        } // Pyo3Type::Map(ref key, ref val) => {
          //     if let Pyo3Type::NonPrimitive = key.as_ref() {
          //         panic!("Key must be a primitive type to be derived into a dict. If you want to use non primitive as a dict key, use a custom implementation");
          //     }

          //     match val.as_ref() {
          //         Pyo3Type::Primitive => todo!(),
          //         Pyo3Type::NonPrimitive => todo!(),
          //         Pyo3Type::CollectionSing(_) => todo!(),
          //         Pyo3Type::Map(_, _) => todo!(),
          //     }
          // }
    };
}

fn handle_single_collection_code_gen(
    py_type: &Pyo3Type,
    ident: &str,
    non_class_ident: &str,
    counter: usize,
) -> TokenStream {
    let curr_pylist: TokenStream = format!("pylist{}{}", counter, non_class_ident)
        .parse()
        .unwrap();
    let next_pylist: TokenStream = format!("pylist{}{}", counter + 1, non_class_ident)
        .parse()
        .unwrap();
    let ident_tok: TokenStream = ident.parse().unwrap();
    match py_type {
        Pyo3Type::Primitive => {
            quote! {
                let mut #curr_pylist = pyo3::types::PyList::empty(py);
                for i in #ident_tok.into_iter() {
                    #curr_pylist.append(i).expect("Bad element in set_item");
                };
            }
        }
        Pyo3Type::NonPrimitive => {
            quote! {
                let mut #curr_pylist = pyo3::types::PyList::empty(py);
                for i in #ident_tok.into_iter() {
                    #curr_pylist.append(i.into_py_dict(py)).expect("Bad element in set_item");
                };
            }
        }
        Pyo3Type::CollectionSing(coll) => {
            let body =
                handle_single_collection_code_gen(coll.as_ref(), "i", non_class_ident, counter + 1);
            quote! {
                let mut #curr_pylist = pyo3::types::PyList::empty(py);
                for i in #ident_tok.into_iter(){
                    #body
                    #curr_pylist.append(#next_pylist).expect("Bad element in set_item");
                };
            }
        }
    }
}

pub fn parse_generics(generics: &Generics) -> String {
    if !generics.params.is_empty() {
        let mut generics_parsed = "<".to_string();

        for param in &generics.params {
            match param {
                syn::GenericParam::Lifetime(lt) => {
                    generics_parsed += ("'".to_string() + &lt.lifetime.ident.to_string()).as_str()
                }
                syn::GenericParam::Type(generic_type) => {
                    generics_parsed += generic_type.ident.to_string().as_str()
                }
                syn::GenericParam::Const(const_type) => {
                    generics_parsed +=
                        ("const".to_string() + const_type.ident.to_string().as_str()).as_str()
                }
            }

            generics_parsed += ",";
        }

        generics_parsed = generics_parsed[0..generics_parsed.len() - 1].to_string();
        generics_parsed += ">";
        generics_parsed
    } else {
        String::new()
    }
}

pub fn check_type(input: &DeriveInput) -> syn::Result<()> {
    match input.data {
        syn::Data::Struct(ref info) => {
            if let syn::Fields::Unnamed(_) = info.fields {
                return Err(syn::Error::new(
                    info.struct_token.span,
                    "No support for tuple structs currently. Please write your own implementation for the struct.",
                ));
            }

            Ok(())
        }
        syn::Data::Enum(ref info) => Err(syn::Error::new(
            info.brace_token.span.close(),
            "No support for enums currently. Please write your own implementation for the enum.",
        )),
        syn::Data::Union(ref info) => Err(syn::Error::new(
            info.union_token.span,
            "No support for unions currently. Please write your own implementation for the union.",
        )),
    }
}
