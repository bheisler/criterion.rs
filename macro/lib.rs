//! `#[criterion]` macro

extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;

use proc_macro2::{Ident, TokenStream, TokenTree};

#[proc_macro_attribute]
pub fn criterion(
    attr: proc_macro::TokenStream, item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let tokens = TokenStream::from(attr).into_iter().collect::<Vec<_>>();

    if tokens.len() != 0 {
        panic!("expected #[criterion]");
    }

    let item = TokenStream::from(item);
    let name = find_fn_name(item.clone());

    let name: TokenStream = name
        .to_string()
        .parse()
        .expect(&format!("failed to parse name: {}", name.to_string()));

    let mut bench_const: String = name.to_string();
    bench_const += "_const";
    let bench_const: TokenStream = bench_const.parse().expect(&format!("failed to
    parse name: {}", bench_const.to_string()));

    let ret: TokenStream = quote_spanned! {
        proc_macro2::Span::call_site() =>
            #item
            #[test_case]
            const #bench_const: ::criterion::CtfBenchmark = ::criterion::CtfBenchmark {
                name: stringify!(#name),
                fun: #name,
            };
    }.into();
    ret.into()
}

/// Find function name
fn find_fn_name(item: TokenStream) -> Ident {
    let mut tokens = item.into_iter();
    while let Some(tok) = tokens.next() {
        if let TokenTree::Ident(word) = tok {
            if word == "fn" {
                break;
            }
        }
    }

    match tokens.next() {
        Some(TokenTree::Ident(word)) => word,
        _ => panic!("failed to find function name"),
    }
}
