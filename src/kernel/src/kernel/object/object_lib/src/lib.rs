extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;

fn impl_kernel_object_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;

    let _struct_type = ast.ident.to_string();

    let gen = quote! {
        impl crate::kernel::object::KernelObject for #name {
            fn as_any(&self) -> &dyn core::any::Any {
                self
            }

            fn invoke(&self, args: &[usize]) -> Result<usize, rtl::error::ErrorType> {
                self.do_invoke(args)
            }
        }

        unsafe impl Send for #name { }
        unsafe impl Sync for #name { }

        impl Drop for #name {
            fn drop(&mut self) {
                // panic!();
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
