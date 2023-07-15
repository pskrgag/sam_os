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

    if fn_name != "main" {
        panic!("Function must be called main!")
    }

    // Build the output, possibly using quasi-quotation
    let expanded = quote! {
        #input_fn

        #[no_mangle]
        pub extern "C" fn _start() {
            main();

            loop { }
        }

        #[panic_handler]
        fn panic(info: &core::panic::PanicInfo) -> ! {
            println!("PANIC!!! {}", info);
            loop {}
        }

        #[macro_use]
        extern crate libc;
    };

    // Hand the output tokens back to the compiler
    TokenStream::from(expanded)}
