use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse2,
    token::Colon,
    Attribute, Ident, ItemFn, Signature, Stmt,
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
#[allow(non_snake_case)] // follow python exception naming for error messages
#[rustfmt::skip] // for import in testcase.imports ... want consistent formatting for each line
fn wrap_testcase(testcase: Pyo3TestCase) -> TokenStream2 {
    let mut o3_moduleidents = Vec::<Ident>::new();
    let mut py_moduleidents = Vec::<Ident>::new();
    let mut pyo3_modulenames = Vec::<String>::new();
    let mut ModuleNotFoundErrormsgs = Vec::<String>::new();
    let mut pyo3_functionnames = Vec::<String>::new();
    let mut AttributeErrormsgs = Vec::<String>::new();
    let mut py_functionidents = Vec::<Ident>::new();
    
    for import in testcase.imports {
        o3_moduleidents.push(
            import.moduleidentifier
        );
        py_moduleidents.push(
            Ident::new(&import.modulename, Span::mixed_site())
        );
        pyo3_modulenames.push(
            import.modulename
        );
        ModuleNotFoundErrormsgs.push(
            "Failed to import ".to_string() + &pyo3_modulenames.iter().last().unwrap()
        );
        pyo3_functionnames.push(
            import.functionname
        );
        AttributeErrormsgs.push(
            "Failed to get ".to_string() + &pyo3_functionnames.iter().last().unwrap() + " function",
        );
        py_functionidents.push(
            Ident::new(pyo3_functionnames.iter().last().unwrap(), Span::mixed_site())
        );
    }

    let testfn_signature = testcase.signature;
    let testfn_statements = testcase.statements;

    quote!(
        #testfn_signature {
            #(pyo3::append_to_inittab!(#o3_moduleidents);)* // allow python to import from each wrapped module
            pyo3::prepare_freethreaded_python();
            Python::with_gil(|py| {
                #(let #py_moduleidents = py
                    .import_bound(#pyo3_modulenames) // import the wrapped module
                    .expect(#ModuleNotFoundErrormsgs);)*
                #(let #py_functionidents = #py_moduleidents
                    .getattr(#pyo3_functionnames) // import the wrapped function
                    .expect(#AttributeErrormsgs);)*
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

/// A pyo3 test case consisting of zero or more imports and an ItemFn which should be wrapped to
/// execute in Python::with_gil
struct Pyo3TestCase {
    imports: Vec<Pyo3Import>,
    signature: Signature,
    statements: Vec<Stmt>,
}

impl From<ItemFn> for Pyo3TestCase {
    fn from(testcase: ItemFn) -> Pyo3TestCase {
        Pyo3TestCase {
            imports: testcase
                .attrs
                .into_iter()
                .map(|attr| { parsepyo3import(&attr) }.unwrap())
                .collect(),
            signature: testcase.sig,
            statements: testcase.block.stmts,
        }
    }
}

#[allow(dead_code)] // Not yet fully implemented
fn impl_pyo3test(_attr: TokenStream2, input: TokenStream2) -> TokenStream2 {
    let testcase: Pyo3TestCase = parse2::<ItemFn>(input).unwrap().into();
    wrap_testcase(testcase)
}

#[cfg(test)]
mod tests {
    use syn::{parse_quote, Attribute};

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
            moduleidentifier: py_fizzbuzzo3,
            modulename: "fizzbuzzo3".to_string(),
            functionname: "fizzbuzz".to_string(),
        };

        let imports = vec![import];

        let testcase: Pyo3TestCase = Pyo3TestCase {
            imports,
            signature: testcase.sig,
            statements: testcase.block.stmts,
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

        let output = wrap_testcase(testcase);

        assert_eq!(output.to_string(), expected.to_string());
    }

    #[test]
    fn test_wrap_testcase_multiline_block() {
        let testcase: ItemFn = parse_quote! {
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

        let imports = vec![import];

        let testcase: Pyo3TestCase = Pyo3TestCase {
            imports,
            signature: testcase.sig,
            statements: testcase.block.stmts,
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

        let output = wrap_testcase(testcase);

        assert_eq!(output.to_string(), expected.to_string());
    }

    #[test]
    fn test_wrap_testcase_multiple_imports() {
        let testcase: ItemFn = parse_quote! {
            fn test_fizzbuzz() {
                assert!(true)
            }
        };

        let py_fizzbuzzo3 = Ident::new("py_fizzbuzzo3", Span::call_site());
        let py_foo_o3 = Ident::new("py_foo_o3", Span::call_site());

        let import1 = Pyo3Import {
            moduleidentifier: py_fizzbuzzo3,
            modulename: "fizzbuzzo3".to_string(),
            functionname: "fizzbuzz".to_string(),
        };

        let import2 = Pyo3Import {
            moduleidentifier: py_foo_o3,
            modulename: "foo_o3".to_string(),
            functionname: "bar".to_string(),
        };

        let imports = vec![import1, import2];

        let testcase: Pyo3TestCase = Pyo3TestCase {
            imports,
            signature: testcase.sig,
            statements: testcase.block.stmts,
        };

        let expected = quote! {
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
