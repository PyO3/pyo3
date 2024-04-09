use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::Ident;


#[allow(dead_code)] // Not yet fully implemented
fn import_pyo3_from(import: Pyo3Import, input: TokenStream2) -> TokenStream2 {
    let moduleident = import.moduleidentifier;
    let pymoduleident = Ident::new(&import.modulename, Span::mixed_site());
    let modulename = import.modulename;
    let modulerror = "Failed to import ".to_string() + &modulename;
    let functionname = import.functionname;
    let functionerror = "Failed to get ".to_string() + &functionname + " function";

    quote!(
        pyo3::append_to_inittab!(#moduleident);
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let #pymoduleident = py
                .import_bound(#modulename)
                .expect(#modulerror);
            let fizzbuzz = fizzbuzzo3
                .getattr(#functionname)
                .expect(#functionerror);
            #input
        });
    )
}

struct Pyo3Import {
    moduleidentifier: Ident,
    modulename: String,
    functionname: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_importmodule() {
        let input = quote! {
            assert!(true);
        };

        let py_fizzbuzzo3 = Ident::new("py_fizzbuzzo3", Span::call_site());

        let module = Pyo3Import {
            moduleidentifier: py_fizzbuzzo3,
            modulename: "fizzbuzzo3".to_string(),
            functionname: "fizzbuzz".to_string(),
        };

        let expected = quote! {
            pyo3::append_to_inittab!(py_fizzbuzzo3);
            pyo3::prepare_freethreaded_python();
            Python::with_gil(|py| {
                let fizzbuzzo3 = py
                .import_bound("fizzbuzzo3")
                .expect("Failed to import fizzbuzzo3");
                let fizzbuzz = fizzbuzzo3
                .getattr("fizzbuzz")
                .expect("Failed to get fizzbuzz function");
                assert!(true);
            });
        };

        let output = import_pyo3_from(module, input);

        assert_eq!(output.to_string(), expected.to_string());
    }
}
