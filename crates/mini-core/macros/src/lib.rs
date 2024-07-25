mod uuid;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(TypeUuidProvider, attributes(type_uuid))]
pub fn type_uuid(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    TokenStream::from(uuid::impl_type_uuid_provider(ast))
}
