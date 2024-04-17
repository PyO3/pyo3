#![cfg(not(any(PyPy, GraalPy)))]
use proc_macro::TokenStream as TokenStream1;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse2,
    token::Colon,
    Attribute, Ident, ItemFn, Signature, Stmt,
};

#[proc_macro_attribute]
pub fn pyo3test(attr: TokenStream1, input: TokenStream1) -> TokenStream1 {
    impl_pyo3test(attr.into(), input.into()).into()
}

/// The function which is called by the proc macro `pyo3test`.
/// Takes a TokenStream2 input, parses it as a Pyo3TestCase and returns a wrapped
/// function with the requested imports, run in Python::with_gil
fn impl_pyo3test(_attr: TokenStream2, input: TokenStream2) -> TokenStream2 {
    let testcase: Pyo3TestCase = parse2::<ItemFn>(input).unwrap().into();
    wrap_testcase(testcase)
}

/// A pyo3 test case consisting of zero or more imports and an ItemFn which should be wrapped to
/// execute in Python::with_gil. Don't construct this directly but use .into() on a suitable ItemFn
struct Pyo3TestCase {
    pythonimports: Vec<Pyo3Import>,
    signature: Signature,
    statements: Vec<Stmt>,
}

impl From<ItemFn> for Pyo3TestCase {
    fn from(testcase: ItemFn) -> Pyo3TestCase {
        Pyo3TestCase {
            pythonimports: testcase
                .attrs
                .into_iter()
                .map(|attr| { parsepyo3import(&attr) }.unwrap())
                .collect(),
            signature: testcase.sig,
            statements: testcase.block.stmts,
        }
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

/// A python `import` statement for a pyo3-wrapped function.
#[derive(Debug, PartialEq)]
struct Pyo3Import {
    /// The *rust* `ident` of the wrapped module
    o3_moduleident: Ident,
    /// The *python* module name
    py_modulename: String,
    /// The *python* function name
    py_functionname: Option<String>,
}

impl Parse for Pyo3Import {
    /// Attributes parsing to Pyo3Imports should have the format:
    /// `moduleidentifier: from modulename import functionname`
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        // Written by a rust newbie, if there is a better option than all these assignments; please
        // feel free to change this code...
        let moduleidentifier = input.parse()?;
        let _colon: Colon = input.parse()?;
        let firstkeyword: String = input.parse::<Ident>()?.to_string();
        let modulename: Ident = input.parse()?;
        let functionname: Option<String>;
        match firstkeyword.as_str() {
            "from" => {
                let _import: Ident = input.parse()?;
                functionname = Some(input.parse::<Ident>()?.to_string());
            }
            "import" => {
                functionname = None;
            }
            _ => return Err(syn::Error::new(input.span(), "invalid import statement")),
        }

        Ok(Pyo3Import {
            o3_moduleident: moduleidentifier,
            py_modulename: modulename.to_string(),
            py_functionname: functionname,
        })
    }
}

/// Takes a code block which should be executed using Python::with_gil and adds the required
/// pyo3 equivalent `import` and `with_gil` statements.
///
/// Technically this is the equivalent to the python statements:
/// ```python
/// import module
/// function = module.function
/// ```
/// and not `from module import function`
#[allow(non_snake_case)] // follow python exception naming for error messages
#[rustfmt::skip] // for import in testcase.imports ... want consistent formatting for each line
fn wrap_testcase(testcase: Pyo3TestCase) -> TokenStream2 {
    let mut o3_moduleidents = Vec::<Ident>::new();
    let mut py_moduleidents = Vec::<Ident>::new();
    let mut py_modulenames = Vec::<String>::new();
    let mut py_ModuleNotFoundErrormsgs = Vec::<String>::new();
    let mut py_functionidents = Vec::<Ident>::new();
    let mut py_moduleswithfnsidents = Vec::<Ident>::new();
    let mut py_functionnames = Vec::<String>::new();
    let mut py_AttributeErrormsgs = Vec::<String>::new();

    
    for import in testcase.pythonimports {
        if let Some(functionname) = import.py_functionname {
            py_AttributeErrormsgs.push("Failed to get ".to_string() + &functionname + " function");
            py_functionidents.push(Ident::new(&functionname, Span::call_site()));
            py_moduleswithfnsidents.push(Ident::new(&import.py_modulename, Span::mixed_site()));
            py_functionnames.push(functionname);
        };
        o3_moduleidents.push(
            import.o3_moduleident
        );
        py_moduleidents.push(
            Ident::new(&import.py_modulename, Span::mixed_site())
        );
        py_modulenames.push(
            import.py_modulename
        );   
        py_ModuleNotFoundErrormsgs.push(
            "Failed to import ".to_string() + py_modulenames.iter().last().unwrap()
        );
    }

    let testfn_signature = testcase.signature;
    let testfn_statements = testcase.statements;

    quote!(
        #[test]
        #testfn_signature {
            #(pyo3::append_to_inittab!(#o3_moduleidents);)* // allow python to import from each wrapped module
            pyo3::prepare_freethreaded_python();
            Python::with_gil(|py| {
                #(let #py_moduleidents = py
                    .import_bound(#py_modulenames) // import the wrapped module
                    .expect(#py_ModuleNotFoundErrormsgs);)*
                #(let #py_functionidents = #py_moduleswithfnsidents
                    .getattr(#py_functionnames) // import the wrapped function
                    .expect(#py_AttributeErrormsgs);)*
                #(#testfn_statements)*
            });
        }
    )
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::*;

    #[test]
    fn test_wrap_testcase() {
        let testcase: ItemFn = parse_quote! {
            fn test_fizzbuzz() {
                assert!(true)
            }
        };

        let py_fizzbuzzo3 = Ident::new("py_fizzbuzzo3", Span::call_site());

        let import = Pyo3Import {
            o3_moduleident: py_fizzbuzzo3,
            py_modulename: "fizzbuzzo3".to_string(),
            py_functionname: Some("fizzbuzz".to_string()),
        };

        let imports = vec![import];

        let testcase: Pyo3TestCase = Pyo3TestCase {
            pythonimports: imports,
            signature: testcase.sig,
            statements: testcase.block.stmts,
        };

        let expected = quote! {
            #[test]
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

        let output = wrap_testcase(testcase);

        assert_eq!(output.to_string(), expected.to_string());
    }

    #[test]
    fn test_parseimport() {
        let import: Attribute = parse_quote! {
            #[pyo3import(o3module: from module import function)]
        };

        let o3module = Ident::new("o3module", Span::call_site());

        let expected = Pyo3Import {
            o3_moduleident: o3module,
            py_modulename: "module".to_string(),
            py_functionname: Some("function".to_string()),
        };

        let parsed = parsepyo3import(&import);

        assert_eq!(parsed.unwrap(), expected)
    }

    #[test]
    fn test_simple_case() {
        let attr = quote! {};

        let input = quote! {
            #[pyo3import(foo_o3: from pyfoo import pybar)]
            fn pytest() {
                assert!(true)
            }
        };

        let expected = quote! {
            #[test]
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

    #[test]
    fn test_multiline_block() {
        let testcase: TokenStream2 = quote! {
            #[pyo3import(py_fizzbuzzo3: from fizzbuzzo3 import fizzbuzz)]
            fn test_fizzbuzz() {
                let x = 1;
                let y = 1;
                assert_eq!(x, y)
            }
        };

        let expected: TokenStream2 = quote! {
            #[test]
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

        let output: TokenStream2 = impl_pyo3test(quote! {}, testcase);

        assert_eq!(output.to_string(), expected.to_string());
    }

    #[test]
    fn test_zero_imports() {
        let attr = quote! {};

        let input = quote! {
            fn pytest() {
                assert!(true)
            }
        };

        let expected = quote! {
            #[test]
            fn pytest() {
                pyo3::prepare_freethreaded_python();
                Python::with_gil(|py| {
                    assert!(true)
                });
            }
        };

        let result = impl_pyo3test(attr, input);

        assert_eq!(result.to_string(), expected.to_string())
    }
    #[test]
    fn test_multiple_imports() {
        let testcase: TokenStream2 = quote! {
            #[pyo3import(py_fizzbuzzo3: from fizzbuzzo3 import fizzbuzz)]
            #[pyo3import(py_foo_o3: from foo_o3 import bar)]
            fn test_fizzbuzz() {
                assert!(true)
            }
        };

        let expected: TokenStream2 = quote! {
            #[test]
            fn test_fizzbuzz() {
                pyo3::append_to_inittab!(py_fizzbuzzo3);
                pyo3::append_to_inittab!(py_foo_o3);
                pyo3::prepare_freethreaded_python();
                Python::with_gil(|py| {
                    let fizzbuzzo3 = py
                    .import_bound("fizzbuzzo3")
                    .expect("Failed to import fizzbuzzo3");
                    let foo_o3 = py
                    .import_bound("foo_o3")
                    .expect("Failed to import foo_o3");
                    let fizzbuzz = fizzbuzzo3
                    .getattr("fizzbuzz")
                    .expect("Failed to get fizzbuzz function");
                    let bar = foo_o3
                    .getattr("bar")
                    .expect("Failed to get bar function");
                    assert!(true)
                });
            }
        };

        let output: TokenStream2 = impl_pyo3test(quote! {}, testcase);

        assert_eq!(output.to_string(), expected.to_string());
    }

    #[test]
    fn test_import_module_only() {
        let attr = quote! {};

        let input = quote! {
            #[pyo3import(foo_o3: import pyfoo)]
            fn pytest() {
                assert!(true)
            }
        };

        let expected = quote! {
            #[test]
            fn pytest() {
                pyo3::append_to_inittab!(foo_o3);
                pyo3::prepare_freethreaded_python();
                Python::with_gil(|py| {
                    let pyfoo = py
                    .import_bound("pyfoo")
                    .expect("Failed to import pyfoo");
                    assert!(true)
                });
            }
        };

        let result = impl_pyo3test(attr, input);

        assert_eq!(result.to_string(), expected.to_string())
    }

    #[test]
    fn test_mixed_import_types() {
        let attr = quote! {};

        let input = quote! {
            #[pyo3import(foo_o3: import pyfoo)]
            #[pyo3import(bar_o3: from pybar import bang)]
            fn pytest() {
                assert!(true)
            }
        };

        let expected = quote! {
            #[test]
            fn pytest() {
                pyo3::append_to_inittab!(foo_o3);
                pyo3::append_to_inittab!(bar_o3);
                pyo3::prepare_freethreaded_python();
                Python::with_gil(|py| {
                    let pyfoo = py
                    .import_bound("pyfoo")
                    .expect("Failed to import pyfoo");
                    let pybar = py
                    .import_bound("pybar")
                    .expect("Failed to import pybar");
                    let bang = pybar
                    .getattr("bang")
                    .expect("Failed to get bang function");
                    assert!(true)
                });
            }
        };

        let result = impl_pyo3test(attr, input);

        assert_eq!(result.to_string(), expected.to_string())
    }
}
