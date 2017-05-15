use syn;
use quote;


enum ImplType {
    Async,
    Buffer,
}

pub fn build_py_impl(ast: &mut syn::Item) -> quote::Tokens {
    match ast.node {
        syn::ItemKind::Impl(_, _, _, ref path, ref ty, ref mut impl_items) => {
            if let &Some(ref path) = path {
                    match process_path(path) {
                        ImplType::Async => {
                            impl_protocol("PyAsyncProtocolImpl",
                                          path.clone(), ty, impl_items, true)
                        }
                        ImplType::Buffer => {
                            impl_protocol("PyBufferProtocolImpl",
                                          path.clone(), ty, impl_items, false)
                        }
                    }
            } else {
                //ImplType::Impl
                unimplemented!()
            }
        },
        _ => panic!("#[py_impl] can only be used with Impl blocks"),
    }
}

fn process_path(path: &syn::Path) -> ImplType {
    if let Some(segment) = path.segments.last() {
            match segment.ident.as_ref() {
                "PyAsyncProtocol" => ImplType::Async,
                "PyBufferProtocol" => ImplType::Buffer,
                _ => panic!("#[py_impl] can not be used with this block"),
            }
    } else {
        panic!("#[py_impl] can not be used with this block");
    }
}

fn impl_protocol(name: &'static str,
                 path: syn::Path, ty: &Box<syn::Ty>,
                 impls: &mut Vec<syn::ImplItem>, adjust_result: bool) -> quote::Tokens {
    // get method names in impl block
    let mut meth = Vec::new();
    for iimpl in impls.iter_mut() {
        match iimpl.node {
            syn::ImplItemKind::Method(ref mut sig, ref mut block) => {
                meth.push(String::from(iimpl.ident.as_ref()));

                // adjust return type
                if adjust_result {
                    impl_adjust_result(sig, block);
                }
            },
            _ => (),
        }
    }

    // set trait name
    let mut path = path;
    {
        let mut last = path.segments.last_mut().unwrap();
        last.ident = syn::Ident::from(name);
    }

    quote! {
        impl #path for #ty {
            fn methods() -> &'static [&'static str] {
                static METHODS: &'static [&'static str] = &[#(#meth,),*];
                METHODS
            }
        }
    }
}

fn impl_adjust_result(sig: &mut syn::MethodSig, block: &mut syn::Block) {
    match sig.decl.output {
        syn::FunctionRetTy::Ty(ref mut ty) => match *ty {
            syn::Ty::Path(_, ref mut path) => {
                // check if function returns PyResult
                if let Some(segment) = path.segments.last_mut() {
                    match segment.ident.as_ref() {
                        // check result type
                        "PyResult" => match segment.parameters {
                            syn::PathParameters::AngleBracketed(ref mut data) => {
                                if rewrite_pyobject(&mut data.types) {
                                    let expr = {
                                        let s = block as &quote::ToTokens;
                                        quote! {
                                            match #s {
                                                Ok(res) => Ok(res.to_py_object(py)),
                                                Err(err) => Err(err)
                                            }
                                        }
                                    };
                                    let expr = syn::parse_expr(&expr.as_str()).unwrap();
                                    let expr = syn::Stmt::Expr(Box::new(expr));
                                    block.stmts = vec![expr];
                                }
                            },
                            _ => (),
                        },
                        _ => (),
                    }
                }
            }
            _ => (),
        },
        syn::FunctionRetTy::Default => (),
    }
}

fn rewrite_pyobject(path: &mut Vec<syn::Ty>) -> bool {
    if path.len() != 1 {
        false
    } else {
        if let &mut syn::Ty::Path(_, ref mut path) = path.first_mut().unwrap() {
            if let Some(segment) = path.segments.last_mut() {
                if segment.ident.as_ref() == "PyObject" {
                    return false
                } else {
                    segment.ident = syn::Ident::from("PyObject");
                    return true
                }
            }
        }
        let ty = syn::Ty::Path(
            None, syn::Path{
                global: false,
                segments: vec![
                    syn::PathSegment {
                        ident: syn::Ident::from("PyObject"),
                        parameters: syn::PathParameters::AngleBracketed(
                            syn::AngleBracketedParameterData {
                                lifetimes: vec![], types: vec![], bindings: vec![] }) }]});
        let _ = path.pop();
        let _ = path.push(ty);
        true
    }
}
