extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Type};

#[proc_macro_derive(PresenterHolder)]
pub fn derive_presenter_holder(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Extract the type of the presenter field
    let mut presenter_type = None;
    if let Data::Struct(data) = &input.data {
        if let Fields::Named(fields) = &data.fields {
            for field in &fields.named {
                if field.ident.as_ref().map(|i| i == "presenter").unwrap_or(false) {
                    if let Type::Path(type_path) = &field.ty {
                        presenter_type = Some(type_path.clone());
                    }
                }
            }
        }
    }

    let presenter_type = match presenter_type {
        Some(t) => t,
        None => panic!("Expected a field named 'presenter'"),
    };

    let expanded = quote! {
        impl #impl_generics HasPresenter<#presenter_type> for #struct_name #ty_generics #where_clause {
            fn presenter_ref(&self) -> &#presenter_type {
                &self.presenter
            }

            fn presenter_mut(&mut self) -> &mut #presenter_type {
                &mut self.presenter
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(StorageHolder)]
pub fn derive_storage_holder(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Extract the type of the presenter field
    let mut storage_type = None;
    if let Data::Struct(data) = &input.data {
        if let Fields::Named(fields) = &data.fields {
            for field in &fields.named {
                if field.ident.as_ref().map(|i| i == "storage").unwrap_or(false) {
                    if let Type::Path(type_path) = &field.ty {
                        storage_type = Some(type_path.clone());
                    }
                }
            }
        }
    }

    let storage_type = match storage_type {
        Some(t) => t,
        None => panic!("Expected a field named 'storage'"),
    };

    let expanded = quote! {
        impl #impl_generics HasStorage<#storage_type> for #struct_name #ty_generics #where_clause {
            fn storage_ref(&self) -> &#storage_type {
                &self.storage
            }

            fn storage_mut(&mut self) -> &mut #storage_type {
                &mut self.storage
            }
        }
    };

    TokenStream::from(expanded)
}
