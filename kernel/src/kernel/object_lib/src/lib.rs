#![no_std]

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn;

fn impl_kernel_object_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let global_slab_name = format_ident!("{}_SLAB", name);
    let wrapper_name = format_ident!("{}Alloc", name);
    let ref_name = format_ident!("{}Ref", name);

    let gen = quote! {
        impl crate::kernel::object::KernelObject for #name {
            fn as_any(&self) -> &dyn core::any::Any {
                self
            }
        }

        crate::slab_allocator!(#global_slab_name, #wrapper_name, #name);
        pub type #ref_name = alloc::sync::Arc<alloc::boxed::Box<qrwlock::RwLock<#name>, #wrapper_name>>;

        impl #name {
            fn construct(new: Self) -> #ref_name {
                use alloc::sync::Arc;
                use alloc::boxed::Box;

                Arc::new(Box::new_in(qrwlock::RwLock::new(new), #wrapper_name::new()))
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
