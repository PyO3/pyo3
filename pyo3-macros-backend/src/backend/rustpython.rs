use proc_macro2::TokenStream;
use quote::quote;

pub(crate) fn rustpython_cfg_item(item: TokenStream) -> TokenStream {
    quote!(#[allow(unexpected_cfgs)] #[cfg(PyRustPython)] #item)
}
