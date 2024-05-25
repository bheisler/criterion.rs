extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenTree};
use quote::quote_spanned;

#[proc_macro_attribute]
pub fn criterion(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = proc_macro2::TokenStream::from(attr);
    let item = proc_macro2::TokenStream::from(item);

    let span = proc_macro2::Span::call_site();

    let init = if stream_length(attr.clone()) != 0 {
        attr
    }
    else {
        quote_spanned!(span=> criterion::Criterion::default())
    };

    let function_name = find_name(item.clone());
    let wrapped_name = Ident::new(&format!("criterion_wrapped_{}", function_name.to_string()), span);

    let output = quote_spanned!(span=>
        #[test_case]
        pub fn #wrapped_name() {
            #item

            let mut c = #init.configure_from_args();
            #function_name(&mut c);
        }
    );

    output.into()
}

fn stream_length(stream: proc_macro2::TokenStream) -> usize {
    stream.into_iter().count()
}

fn find_name(stream: proc_macro2::TokenStream) -> Ident {
    let mut iter = stream.into_iter();
    while let Some(tok) = iter.next() {
        if let TokenTree::Ident(ident) = tok {
            if ident == "fn" {
                break;
            }
        }
    }
    
    if let Some(TokenTree::Ident(name)) = iter.next() {
        name
    }
    else {
        panic!("Unable to find function name")
    }
}