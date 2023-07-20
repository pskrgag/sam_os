#![no_std]

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn;

fn impl_kernel_object_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let ref_name = format_ident!("{}Ref", name);
    let weak_ref_name = format_ident!("{}WeakRef", name);

    let gen = quote! {
        use alloc::sync::Arc;
        use qrwlock::RwLock;

        impl crate::kernel::object::KernelObject for #name {
            fn as_any(&self) -> &dyn core::any::Any {
                self
            }
        }

        pub type #ref_name = alloc::sync::Arc<qrwlock::RwLock<#name>>;
        pub type #weak_ref_name = alloc::sync::Weak<qrwlock::RwLock<#name>>;

        impl #name {
            fn construct(s: Self) -> #ref_name {
                alloc::sync::Arc::new(qrwlock::RwLock::new(s))
            }
        }
    };

    gen.into()
}

#[proc_macro_derive(object)]
pub fn kernel_object_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_kernel_object_macro(&ast)
}
