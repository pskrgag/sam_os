use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

extern crate quote;

extern crate proc_macro;
extern crate syn;

#[proc_macro_attribute]
pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let mut old_func = input_fn.clone();
    let new_ident = syn::Ident::new("real_main", fn_name.span());

    old_func.sig.ident = new_ident;

    if input_fn.sig.asyncness.is_none() {
        return syn::Error::new(
            fn_name.span(),
            format!("Function '{}' should be async", fn_name),
        )
        .to_compile_error()
        .into();
    }

    if fn_name != "main" {
        panic!("Function must be called main!")
    }

    // Build the output, possibly using quasi-quotation
    let expanded = quote! {
        #[libc::main]
        fn main(handle: Option<Handle>) {
            rokio::executor::block_on(real_main(handle));
            panic!("return from main");
        }

        #old_func
    };

    // Hand the output tokens back to the compiler
    TokenStream::from(expanded)
}
