mod resource_data;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(ResourceData, attributes(type_uuid))]
pub fn type_resource_data(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    TokenStream::from(resource_data::impl_type_resource_data(ast))
}
