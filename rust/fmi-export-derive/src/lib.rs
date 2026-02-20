use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(ValueReference)]
pub fn derive_value_reference(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let variants = match &input.data {
        Data::Enum(data_enum) => &data_enum.variants,
        _ => panic!("ValueReference can only be derived for enums"),
    };

    let match_arms = variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let discriminant = match &variant.discriminant {
            Some((_, expr)) => quote! { #expr },
            None => panic!("All variants must have explicit discriminants"),
        };
        
        quote! {
            #discriminant => Ok(#name::#variant_name)
        }
    });

    let expanded = quote! {
        impl TryFrom<u32> for #name {
            type Error = ();

            fn try_from(value: u32) -> Result<Self, Self::Error> {
                match value {
                    #(#match_arms,)*
                    _ => Err(()),
                }
            }
        }
    };

    TokenStream::from(expanded)
}
