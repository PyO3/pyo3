use criterion::{criterion_group, criterion_main, Bencher, Criterion};
use pyo3_macros_backend::PyClassArgs;
use quote::quote;
use syn::parse_quote;

fn bench_parse_base(b: &mut Bencher<'_>) {
    let attrs: Vec<syn::Attribute> = vec![parse_quote! {#[pyo3(name = "Foo")]}];
    b.iter(|| {
        let mut args =
            syn::parse::Parser::parse2(PyClassArgs::parse_struct_args, quote! {}).unwrap();
        let mut attrs = attrs.clone();
        args.options.take_pyo3_options(&mut attrs).unwrap();
    });
}

fn bench_parse_cfg_attr(b: &mut Bencher<'_>) {
    let attrs: Vec<syn::Attribute> =
        vec![parse_quote! {#[cfg_attr(feature = "pyo3", (name = "Foo"))]}];
    b.iter(|| {
        let mut args =
            syn::parse::Parser::parse2(PyClassArgs::parse_struct_args, quote! {}).unwrap();
        let mut attrs = attrs.clone();
        args.options.take_pyo3_options(&mut attrs).unwrap();
    });
}

fn bench_parse_null_cfg_attr(b: &mut Bencher<'_>) {
    let attrs: Vec<syn::Attribute> =
        vec![parse_quote! {#[cfg_attr(feature = "something_else", (name = "Foo"))]}];
    b.iter(|| {
        let mut args =
            syn::parse::Parser::parse2(PyClassArgs::parse_struct_args, quote! {}).unwrap();
        let mut attrs = attrs.clone();
        args.options.take_pyo3_options(&mut attrs).unwrap();
    });
}

fn bench_parse_irrelevant_attribute(b: &mut Bencher<'_>) {
    let attrs: Vec<syn::Attribute> = vec![parse_quote! {#[foobar(bar = "baz")]}];
    b.iter(|| {
        let mut args =
            syn::parse::Parser::parse2(PyClassArgs::parse_struct_args, quote! {}).unwrap();
        let mut attrs = attrs.clone();
        args.options.take_pyo3_options(&mut attrs).unwrap();
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("bench_parse_base", bench_parse_base);
    c.bench_function("bench_parse_cfg_attr", bench_parse_cfg_attr);
    c.bench_function("bench_parse_null_cfg_attr", bench_parse_null_cfg_attr);
    c.bench_function(
        "bench_parse_irrelevant_attribute",
        bench_parse_irrelevant_attribute,
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
