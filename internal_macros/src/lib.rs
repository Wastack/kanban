extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(PresenterHolder)]
pub fn derive_presenter_holder(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            pub fn presenter(&self) -> &Presenter {
                &self.presenter
            }
        }
    };

    TokenStream::from(expanded)
}
