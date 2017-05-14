use syn;
use quote;


enum ImplType {
    Buffer,
}

pub fn build_py_impl(ast: &syn::Item) -> quote::Tokens {
    match ast.node {
        syn::ItemKind::Impl(_, _, _, ref path, ref ty, ref impl_items) => {
            if let &Some(ref path) = path {
                    match process_path(path) {
                        ImplType::Buffer => {
                            impl_protocol("PyBufferProtocolImpl", path.clone(), ty, impl_items)
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
                "PyBufferProtocol" => ImplType::Buffer,
                _ => panic!("#[py_impl] can not be used with this block"),
            }
    } else {
        panic!("#[py_impl] can not be used with this block");
    }
}

fn impl_protocol(name: &'static str,
                 path: syn::Path, ty: &Box<syn::Ty>,
                 impls: &Vec<syn::ImplItem>) -> quote::Tokens {
    // get method names in impl block
    let mut meth = Vec::new();
    for iimpl in impls.iter() {
        meth.push(String::from(iimpl.ident.as_ref()))
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
