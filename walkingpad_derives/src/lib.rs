extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::DeriveInput;
use syn::Data;
use quote::quote;

#[proc_macro_derive(EnumVariants)]
pub fn enum_variants(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);
    variants(&ast).unwrap_or_else(|err| err.to_compile_error()).into()
}

fn variants(ast: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = &ast.ident;

    let variants = match &ast.data {
        Data::Enum(v) => &v.variants,
        _ => return Err(syn::Error::new(Span::call_site(), "This macro only supports enums."))
    };
    let variants_len = variants.len();

    Ok(quote! {
        impl #name {
            const VARIANTS: [#name; #variants_len] = [ #variants ]; 
        }
    })
}
