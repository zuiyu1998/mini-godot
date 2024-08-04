use darling::*;
use proc_macro2::TokenStream as TokenStream2;
use quote::*;
use syn::*;

#[derive(FromDeriveInput)]
#[darling(supports(enum_any))]
pub struct TypeArgs {
    pub ident: Ident,
    pub data: ast::Data<Variant, ()>,
    pub generics: Generics,
}

pub fn impl_enum_variant_meta(ast: DeriveInput) -> TokenStream2 {
    let ty_args = TypeArgs::from_derive_input(&ast).unwrap();
    let ty_ident = &ty_args.ident;

    let (impl_generics, ty_generics, where_clause) = ty_args.generics.split_for_impl();

    let variants = ty_args.data.take_enum().unwrap();

    let idents = variants.iter().map(|v| &v.ident);
    let names = variants.iter().map(|v| v.ident.to_string());
    let indices = 0..names.len();

    quote! {

        impl #impl_generics #ty_ident #ty_generics #where_clause {
            pub fn enum_variant_index(&self) -> usize {
                match self {
                    #(#ty_ident::#idents {..} => #indices,)*
                }
            }
            pub fn enum_variant_name(&self) -> &'static str {
                static variants: &[&str] = &[
                    #(#names,)*
                ];
                let index = self.enum_variant_index();
                variants[index]
            }
        }

    }
}
