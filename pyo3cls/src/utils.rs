use quote::{Tokens, ToTokens};


pub fn print_err(msg: String, t: Tokens) {
    println!("Error: {} in '{}'", msg, t.to_string());
}

pub fn for_err_msg(i: &ToTokens) -> String {
    let mut tokens = Tokens::new();

    i.to_tokens(&mut tokens);
    tokens.as_str().to_string()
}
