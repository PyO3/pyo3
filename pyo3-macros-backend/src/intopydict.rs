use quote::{quote, TokenStreamExt};
use std::ops::AddAssign;

use proc_macro2::TokenStream;
use syn::{
    parse::{Parse, ParseStream},
    Generics,
};

const SINGLE_COL: [&str; 6] = [
    "BTreeSet",
    "BinaryHeap",
    "Vec",
    "HashSet",
    "LinkedList",
    "VecDeque",
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
}

impl Pyo3DictField {
    pub fn new(name: String, type_: &str) -> Self {
        Self {
            name,
            attr_type: Self::check_primitive(type_),
        }
    }

    fn check_primitive(attr_type: &str) -> Pyo3Type {
        for collection in SINGLE_COL {
            if attr_type.starts_with(collection) {
                let attr_type = attr_type.replace('>', "");
                let attr_list: Vec<&str> = attr_type.split('<').collect();
                let out = Self::handle_collection(&attr_list);

                return out;
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

    fn handle_collection(attr_type: &[&str]) -> Pyo3Type {
        match attr_type[0] {
            "BTreeSet" | "BinaryHeap" | "Vec" | "HashSet" | "LinkedList" | "VecDeque" => {
                Pyo3Type::CollectionSing(Box::new(Self::handle_collection(&attr_type[1..])))
            }
            // "BTreeMap" | "HashMap" => {
            //     let join = &attr_type.join("<");
            //     let types: Vec<&str> = join.split(',').collect();
            //     let key: Vec<&str> = types[0].split('<').collect();
            //     let val: Vec<&str> = types[1].split('<').collect();

            //     Pyo3Type::Map(
            //         Box::new(Self::handle_collection(&key)),
            //         Box::new(Self::handle_collection(&val)),
            //     )
            // }
            "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32" | "u64"
            | "u128" | "usize" | "f32" | "f64" | "char" | "bool" | "&str" | "String" => {
                Pyo3Type::Primitive
            }
            _ => Pyo3Type::NonPrimitive,
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

        let tok_split: Vec<&str> = binding.split(',').collect();

        let mut field_collection: Vec<Pyo3DictField> = Vec::new();

        for i in &tok_split {
            let tok_params_unparsed = &i.to_string();
            let tok_bind: Vec<&str> = tok_params_unparsed.split(':').collect();
            if tok_bind.len() == 2 {
                field_collection.push(Pyo3DictField::new(tok_bind[0].to_string(), tok_bind[1]));
            }
        }

        Ok(Pyo3Collection(field_collection))
    }
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
        let mut pydict = PyDict::new(py);
    };

    for field in &dict_fields.0 {
        let ident = &field.name;
        let ident_tok: TokenStream = field.name.parse().unwrap();
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
                    &format!("self.{}", ident),
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
    body.append_all(quote! {
        return pydict;
    });

    body
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
                for i in #ident.into_iter() {
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
        } // Pyo3Type::Map(_, _) => todo!(),
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
