use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse2,
    token::Colon,
    Attribute, Ident, ItemFn,
};

/// Takes a code block which should be executed using Python::with_gil, and after importing a pyo3-wrapped
/// function and adds the required `import` and `with_gil` statements.
///
/// Technically this is the equivalent to the python statements:
/// ```python
/// import module
/// function = module.function
/// ```
/// and not `from module import function`
fn wrap_testcase(import: Pyo3Import, testcase: ItemFn) -> TokenStream2 {
    let o3_moduleident = import.moduleidentifier;
    let py_moduleident = Ident::new(&import.modulename, Span::mixed_site());
    let pyo3_modulename = import.modulename;
    #[allow(non_snake_case)] // "follow python exception naming
    let ModuleNotFoundErrormsg = "Failed to import ".to_string() + &pyo3_modulename;
    let pyo3_functionname = import.functionname;
    #[allow(non_snake_case)] // "follow python exception naming
    let AttributeErrormsg = "Failed to get ".to_string() + &pyo3_functionname + " function";
    let py_functionident = Ident::new(&pyo3_functionname, Span::mixed_site());
    let testfn_signature = &testcase.sig;
    let testfn_statements = &testcase.block.stmts;

    quote!(
        #testfn_signature {
            pyo3::append_to_inittab!(#o3_moduleident); // allow python to import from this wrapped module
            pyo3::prepare_freethreaded_python();
            Python::with_gil(|py| {
                let #py_moduleident = py
                    .import_bound(#pyo3_modulename) // import the wrapped module
                    .expect(#ModuleNotFoundErrormsg);
                let #py_functionident = #py_moduleident
                    .getattr(#pyo3_functionname) // import the wrapped function
                    .expect(#AttributeErrormsg);
                #(#testfn_statements)*
            });
        }
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
fn parsepyo3import(import: &Attribute) -> Option<Pyo3Import> {
    if import.path().is_ident("pyo3import") {
        Some(import.parse_args().unwrap())
    } else {
        None
    }
}

#[allow(dead_code)] // Not yet fully implemented
fn impl_pyo3test(_attr: TokenStream2, input: TokenStream2) -> TokenStream2 {
    let input: ItemFn = parse2(input).unwrap();
    let import = parsepyo3import(&input.attrs[0]).unwrap();
    wrap_testcase(import, input)
}

#[cfg(test)]
mod tests {
    use syn::{parse_quote, Attribute};

    use super::*;

    #[test]
    fn test_wrap_testcase() {
        let testcase = parse_quote! {
            fn test_fizzbuzz() {
                assert!(true)
            }
        };

        let py_fizzbuzzo3 = Ident::new("py_fizzbuzzo3", Span::call_site());

        let import = Pyo3Import {
            moduleidentifier: py_fizzbuzzo3,
            modulename: "fizzbuzzo3".to_string(),
            functionname: "fizzbuzz".to_string(),
        };

        let expected = quote! {
            fn test_fizzbuzz() {
                pyo3::append_to_inittab!(py_fizzbuzzo3);
                pyo3::prepare_freethreaded_python();
                Python::with_gil(|py| {
                    let fizzbuzzo3 = py
                    .import_bound("fizzbuzzo3")
                    .expect("Failed to import fizzbuzzo3");
                    let fizzbuzz = fizzbuzzo3
                    .getattr("fizzbuzz")
                    .expect("Failed to get fizzbuzz function");
                    assert!(true)
                });
            }
        };

        let output = wrap_testcase(import, testcase);

        assert_eq!(output.to_string(), expected.to_string());
    }

    #[test]
    fn test_wrap_testcase_multiline_block() {
        let testcase = parse_quote! {
            fn test_fizzbuzz() {
                let x = 1;
                let y = 1;
                assert_eq!(x, y)
            }
        };

        let py_fizzbuzzo3 = Ident::new("py_fizzbuzzo3", Span::call_site());

        let import = Pyo3Import {
            moduleidentifier: py_fizzbuzzo3,
            modulename: "fizzbuzzo3".to_string(),
            functionname: "fizzbuzz".to_string(),
        };

        let expected = quote! {
            fn test_fizzbuzz() {
                pyo3::append_to_inittab!(py_fizzbuzzo3);
                pyo3::prepare_freethreaded_python();
                Python::with_gil(|py| {
                    let fizzbuzzo3 = py
                    .import_bound("fizzbuzzo3")
                    .expect("Failed to import fizzbuzzo3");
                    let fizzbuzz = fizzbuzzo3
                    .getattr("fizzbuzz")
                    .expect("Failed to get fizzbuzz function");
                    let x = 1;
                    let y = 1;
                    assert_eq!(x, y)
                });
            }
        };

        let output = wrap_testcase(import, testcase);

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

        let parsed = parsepyo3import(&import);

        assert_eq!(parsed.unwrap(), expected)
    }

    #[test]
    fn test_macro() {
        let attr = quote! {};

        let input = quote! {
            #[pyo3import(foo_o3: from pyfoo import pybar)]
            fn pytest() {
                assert!(true)
            }
        };

        let expected = quote! {
            fn pytest() {
                pyo3::append_to_inittab!(foo_o3);
                pyo3::prepare_freethreaded_python();
                Python::with_gil(|py| {
                    let pyfoo = py
                    .import_bound("pyfoo")
                    .expect("Failed to import pyfoo");
                    let pybar = pyfoo
                    .getattr("pybar")
                    .expect("Failed to get pybar function");
                    assert!(true)
                });
            }
        };

        let result = impl_pyo3test(attr, input);

        assert_eq!(result.to_string(), expected.to_string())
    }
}
