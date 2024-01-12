extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;

use std::string::ToString;

fn impl_kernel_object_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;

    let _struct_type = ast.ident.to_string();

    let gen = quote! {
        impl crate::ir::IrObject for #name {
            fn as_any(&self) -> &dyn core::any::Any {
                self
            }
        }
    };

    gen.into()
}

#[proc_macro_derive(ir)]
pub fn kernel_object_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_kernel_object_macro(&ast)
}
