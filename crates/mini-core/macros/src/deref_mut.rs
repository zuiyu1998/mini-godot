use darling::*;
use proc_macro2::TokenStream as TokenStream2;
use quote::*;
use syn::*;

#[derive(FromDeriveInput)]
#[darling(supports(struct_newtype))]
pub struct TypeArgs {
    pub ident: Ident,
    pub generics: Generics,
}

pub fn impl_deref_mut(ast: DeriveInput) -> TokenStream2 {
    let ty_args = TypeArgs::from_derive_input(&ast).unwrap();
    let ty_ident = &ty_args.ident;

    let (impl_generics, ty_generics, where_clause) = ty_args.generics.split_for_impl();

    quote! {

        impl #impl_generics std::ops::DerefMut for #ty_ident #ty_generics #where_clause {

            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

    }
}
