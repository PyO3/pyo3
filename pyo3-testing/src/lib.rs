use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    token::Colon,
    Attribute, Ident,
};

/// Takes a code block which should be executed using the python gil, after importing a pyo3-wrapped
/// function and adds the required `import` and `with_gil` statements.
/// 
/// Technically this is the equivalent to the python statements:
/// ```python
/// import module
/// function = module.function
/// ```
/// and not `from module import function`
#[allow(dead_code)] // Not yet fully implemented
fn import_pyo3_from(import: Pyo3Import, input: TokenStream2) -> TokenStream2 {
    let moduleident = import.moduleidentifier;
    let pymoduleident = Ident::new(&import.modulename, Span::mixed_site());
    let modulename = import.modulename;
    let modulerror = "Failed to import ".to_string() + &modulename;
    let functionname = import.functionname;
    let functionerror = "Failed to get ".to_string() + &functionname + " function";

    quote!(
        pyo3::append_to_inittab!(#moduleident); // allow python to import from this wrapped module
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let #pymoduleident = py
                .import_bound(#modulename) // import the wrapped module
                .expect(#modulerror);
            let fizzbuzz = fizzbuzzo3
                .getattr(#functionname) // import the wrapped function
                .expect(#functionerror);
            #input
        });
    )
}

/// A python `import` statement for a pyo3-wrapped function.
#[derive(Debug, PartialEq)]
struct Pyo3Import {
    /// The *rust* `ident` of the wrapped module
    moduleidentifier: Ident,
    /// The *python* module name
    modulename: String,
    /// The *python* function name
    functionname: String,
}

impl Parse for Pyo3Import {
    /// Attributes parsing to Pyo3Imports should have the format:
    /// `moduleidentifier: from modulename import functionname`
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        // Written by a rust newbie, if there is a better option than all these assignments; please
        // feel free to change this code...
        let moduleidentifier = input.parse()?;
        let _colon: Colon = input.parse()?;
        let _from: Ident = input.parse()?;
        let modulename: Ident = input.parse()?;
        let _import: Ident = input.parse()?;
        let functionname: Ident = input.parse()?;
        Ok(Pyo3Import {
            moduleidentifier,
            modulename: modulename.to_string(),
            functionname: functionname.to_string(),
        })
    }
}

/// Parse an `Attribute` as a `pyo3import`, including path validation.
///
/// Return:
/// - `Some(Pyo3Import)` for Attributes with the path `pyo3import` e.g.:
/// `#[pyo3import(foo: from foo import bar)]`
/// - `None` for Attributes with other paths.
#[allow(dead_code)] // Not yet fully implemented
fn parsepyo3import(import: Attribute) -> Option<Pyo3Import> {
    if import.path().is_ident("pyo3import") {
        Some(import.parse_args().unwrap())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use syn::{parse_quote, Attribute};

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

    #[test]
    fn test_parseimport() {
        let import: Attribute = parse_quote! {
            #[pyo3import(o3module: from module import function)]
        };

        let o3module = Ident::new("o3module", Span::call_site());

        let expected = Pyo3Import {
            moduleidentifier: o3module,
            modulename: "module".to_string(),
            functionname: "function".to_string(),
        };

        let parsed = parsepyo3import(import);

        assert_eq!(parsed.unwrap(), expected)
    }
}
