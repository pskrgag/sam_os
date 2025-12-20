extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;

fn impl_kernel_object_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;

    let _struct_type = ast.ident.to_string();

    let gen = quote! {
        impl crate::object::KernelObject for #name {
            fn as_any(&self) -> &dyn core::any::Any {
                self
            }

            fn signal(&self, signals: crate::object::signals::Signals) {
                self.base.signal(signals)
            }

            fn wait_event(&self, obs: crate::object::ObserverHandler) {
                self.base.add_observer(obs)
            }
        }

        unsafe impl Send for #name { }
        unsafe impl Sync for #name { }
    };

    gen.into()
}

#[proc_macro_derive(object)]
pub fn kernel_object_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_kernel_object_macro(&ast)
}
