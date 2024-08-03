use darling::*;
use proc_macro2::TokenStream as TokenStream2;
use quote::*;
use syn::*;

#[derive(FromDeriveInput)]
#[darling(supports(struct_newtype))]
pub struct TypeArgs {
    pub ident: Ident,
    pub data: ast::Data<(), Field>,
    pub generics: Generics,
}

pub fn impl_deref(ast: DeriveInput) -> TokenStream2 {
    let ty_args = TypeArgs::from_derive_input(&ast).unwrap();
    let ty_ident = &ty_args.ident;

    let ty_inline = ty_args.data.take_struct().unwrap().fields[0].clone().ty;

    let (impl_generics, ty_generics, where_clause) = ty_args.generics.split_for_impl();

    quote! {

        impl #impl_generics std::ops::Deref for #ty_ident #ty_generics #where_clause {
            type Target = #ty_inline;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

    }
}
