
use std::ops::AddAssign;

use proc_macro2::TokenStream;
use syn::{parse::{ParseStream, Parse}, Generics};

const SINGLE_COL: [&str; 6] = ["BTreeSet", "BinaryHeap", "Vec", "HashSet", "LinkedList", "VecDeque"];

#[derive(Debug, Clone)]
enum Pyo3Type {
    Primitive,
    NonPrimitive,
    CollectionSing(Box<crate::intopydict::Pyo3Type>),
    Map(Box<crate::intopydict::Pyo3Type>, Box<crate::intopydict::Pyo3Type>),
}


#[derive(Debug, Clone)]
pub struct Pyo3DictField {
    name: String,
    attr_type: Pyo3Type
}

impl Pyo3DictField {
    pub fn new(name: String, type_: &str) -> Self { Self { name, attr_type: Self::check_primitive(&type_) } }

    fn check_primitive(attr_type: &str) -> Pyo3Type{
        for collection in SINGLE_COL {
            if attr_type.starts_with(collection) {
                let attr_list: Vec<&str> = attr_type.split(['<', '>']).into_iter().collect();
                let out = Self::handle_collection(&attr_list);

                return out;
            }
        }

        match attr_type {
            "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32" | "u64" | "u128" | "usize" | "f32" | "f64" | "char" | "bool" | "&str" | "String" => return Pyo3Type::Primitive,
            _ => return  Pyo3Type::NonPrimitive,
        }
    }

    fn handle_collection(attr_type: &[&str]) -> Pyo3Type {
        match attr_type[0] {
            "BTreeSet" | "BinaryHeap" | "Vec" | "HashSet" | "LinkedList" | "VecDeque" => return Pyo3Type::CollectionSing(Box::new(Self::handle_collection(&attr_type[1..]))), 
            "BTreeMap" | "HashMap" => {
                let join = &attr_type.join("<");
                let types: Vec<&str> = join.split(",").collect();
                let key: Vec<&str> = types[0].split("<").collect();
                let val: Vec<&str> = types[1].split("<").collect();
                let map = Pyo3Type::Map(Box::new(Self::handle_collection(&key)), Box::new(Self::handle_collection(&val)));
                panic!("{:?}", map);
                return map;
            },
            "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32" | "u64" | "u128" | "usize" | "f32" | "f64" | "char" | "bool" | "&str" | "String" => return Pyo3Type::Primitive,
            _ => return Pyo3Type::NonPrimitive
        }
    }
}

impl Parse for Pyo3Collection {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let tok_stream: TokenStream = input.parse()?;
        let binding = tok_stream.to_string().as_str().replace(" ", "").replace("{", "").replace("}", "");

        let tok_split: Vec<&str> = binding.split(",").collect();

        if tok_split.len() <= 1{
            return Ok(Pyo3Collection(Vec::new()))
        }

        let mut field_collection: Vec<Pyo3DictField> = Vec::new();
        
        for i in tok_split.iter() {
            let tok_params_unparsed = &i.to_string();
            let tok_bind: Vec<&str> = tok_params_unparsed.split(":").collect();
            if tok_bind.len() == 2 {
                field_collection.push(Pyo3DictField::new(tok_bind[0].to_string(), tok_bind[1]));
            }
        }

        return Ok(Pyo3Collection(field_collection));
    }
}

#[derive(Debug, Clone)]
pub struct Pyo3Collection(pub Vec<Pyo3DictField>);

impl AddAssign for Pyo3Collection {
    fn add_assign(&mut self, rhs: Self) {
        self.0.extend(rhs.0.into_iter());
    }
}

pub fn build_derive_into_pydict(dict_fields: Pyo3Collection) -> TokenStream  {
    let mut body: String = String::from("let mut pydict = PyDict::new(py);\n");

    for field in dict_fields.0.iter() {
        let ident = &field.name;
        match field.attr_type {
            Pyo3Type::Primitive => {
                body += &format!("pydict.set_item(\"{}\", self.{}).expect(\"Bad element in set_item\");", ident, ident);
            },
            Pyo3Type::NonPrimitive => {
                body += &format!("pydict.set_item(\"{}\", self.{}.into_py_dict(py)).expect(\"Bad element in set_item\");\n", ident, ident);
            },
            Pyo3Type::CollectionSing(ref collection) => {
                let non_class_ident = ident.replace(".", "_");
                body += &handle_single_collection_code_gen(collection, &format!("self.{}", ident), &non_class_ident, 0);
                body += &format!("pydict.set_item(\"{}\", pylist0{}).expect(\"Bad element in set_item\");\n", ident, ident)
            },
            Pyo3Type::Map(ref key, ref val) => {
                if let Pyo3Type::NonPrimitive = key.as_ref() {
                    panic!("Key must be a primitive type to be derived into a dict. If you want to use non primitive as a dict key, use a custom implementation");
                }

                match val.as_ref() {
                    Pyo3Type::Primitive => todo!(),
                    Pyo3Type::NonPrimitive => todo!(),
                    Pyo3Type::CollectionSing(_) => todo!(),
                    Pyo3Type::Map(_, _) => todo!(),
                }
            }
        };
    }
    body += "return pydict;";

    return body.parse().unwrap();
}

fn handle_single_collection_code_gen(py_type: &Pyo3Type, ident: &str, non_class_ident: &str, counter: usize) -> String {
    match py_type {
        Pyo3Type::Primitive => return format!("
            let mut pylist{}{} = pyo3::types::PyList::empty(py);
            for i in {}.into_iter() {{
                pylist{}{}.append(i).expect(\"Bad element in set_item\");
            }};
        ", counter, non_class_ident, ident, counter, non_class_ident),
        Pyo3Type::NonPrimitive => return format!("
        let mut pylist{}{} = pyo3::types::PyList::empty(py);
        for i in {}.into_iter() {{
            pylist{}{}.append(i.into_py_dict(py)).expect(\"Bad element in set_item\");
        }};
    ", counter, non_class_ident, ident, counter, non_class_ident),
        Pyo3Type::CollectionSing(coll) => {
            let out = format!("
                let mut pylist{}{} = pyo3::types::PyList::empty(py);
                for i in {} .into_iter(){{
                    {}
                    pylist{}{}.append(pylist{}{}).expect(\"Bad element in set_item\");
                }};
            ", counter, non_class_ident, ident, handle_single_collection_code_gen(coll.as_ref(), "i", non_class_ident, counter + 1), counter, non_class_ident, counter + 1, non_class_ident);
            return out;
        },
        Pyo3Type::Map(_, _) => todo!(),
    }
}

pub fn parse_generics(generics: &Generics) -> String {
    if generics.params.len() > 0 {
        let mut generics_parsed = "<".to_string();

        for param in &generics.params {
            match param {
                syn::GenericParam::Lifetime(lt) => generics_parsed += ("'".to_string() + &lt.lifetime.ident.to_string()).as_str(),
                syn::GenericParam::Type(generic_type) => generics_parsed += generic_type.ident.to_string().as_str(),
                syn::GenericParam::Const(const_type) => generics_parsed += ("const".to_string() + const_type.ident.to_string().as_str()).as_str(),
            }

            generics_parsed += ",";
        }

        generics_parsed = generics_parsed[0..generics_parsed.len() - 1].to_string();
        generics_parsed += ">";
        return generics_parsed;
    } else {
        return String::new();
    }
}