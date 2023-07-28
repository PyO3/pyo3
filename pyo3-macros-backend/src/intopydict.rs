
use std::ops::AddAssign;

use syn::{parse::{ParseStream, Parse}, parse_macro_input, __private::TokenStream, Generics};

#[derive(Debug, Clone)]
enum Pyo3Type {
    Primitive,
    NonPrimitive
}


#[derive(Debug, Clone)]
struct Pyo3DictField {
    name: String,
    attr_type: Pyo3Type
}

impl Pyo3DictField {
    fn new(name: String, type_: &str) -> Self { Self { name, attr_type: Self::check_primitive(&type_) } }

    fn check_primitive(attr_type: &str) -> Pyo3Type{
        match attr_type {
            "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32" | "u64" | "u128" | "usize" | "f32" | "f64" | "char" | "bool" | "&str" | "String" => return Pyo3Type::Primitive,
            _ => return  Pyo3Type::NonPrimitive,
        }
    }
}

impl Parse for Pyo3Collection {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let tok_stream: proc_macro2::TokenStream = input.parse()?;
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
struct Pyo3Collection(Vec<Pyo3DictField>);

impl AddAssign for Pyo3Collection {
    fn add_assign(&mut self, rhs: Self) {
        self.0.extend(rhs.0.into_iter());
    }
}

pub fn build_derive_into_pydict(tokens: TokenStream) -> TokenStream  {
    let mut body: String = String::from("let mut pydict = PyDict::new(py);\n");
    let mut dict_fields: Pyo3Collection = Pyo3Collection(Vec::new());
    for token in tokens {
        let token_stream: syn::__private::TokenStream = token.into();
        dict_fields += parse_macro_input!(token_stream as Pyo3Collection);
    }

    for field in dict_fields.0.iter() {
        let ident = &field.name;
        match field.attr_type {
            Pyo3Type::Primitive => {
                body += &format!("pydict.set_item(\"{}\", self.{}).expect(\"Bad element in set_item\");\n", ident, ident);
            },
            Pyo3Type::NonPrimitive => {
                body += &format!("pydict.set_item(\"{}\", self.{}.into_py_dict(py)).expect(\"Bad element in set_item\");\n", ident, ident);
            },
        };
    }
    body += "return pydict;";

    return body.parse().unwrap();
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