use proc_macro2::{Span, TokenStream};
use quote::{TokenStreamExt, format_ident, quote};
use syn::{
    DeriveInput, Expr, Field, Fields, Ident, Token, braced, parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    spanned::Spanned,
    token,
};

struct DrawInstructionVariant {
    variant_ident: Ident,
    paren_token: syn::token::Paren,
    type_ident: Ident,
}

impl Parse for DrawInstructionVariant {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;

        Ok(DrawInstructionVariant {
            variant_ident: input.parse()?,
            paren_token: parenthesized!(content in input),
            type_ident: content.parse()?,
        })
    }
}

struct DrawInstructionInput {
    ident: Ident,
    brace_token: token::Brace,
    variants: Punctuated<DrawInstructionVariant, Token![,]>,
}

impl Parse for DrawInstructionInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;

        Ok(DrawInstructionInput {
            ident: input.parse()?,
            brace_token: braced!(content in input),
            variants: content.parse_terminated(DrawInstructionVariant::parse, Token![,])?,
        })
    }
}

#[proc_macro]
pub fn draw_instruction(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(item as DrawInstructionInput);

    let struct_identifier = input.ident;
    let enum_identifier = format_ident!("{}Discriminant", struct_identifier);
    let properties_identifier = format_ident!("{}Properties", struct_identifier);

    let variant_identifiers = input.variants.iter().map(|variant| &variant.variant_ident);
    let draw_match_identifiers = variant_identifiers.clone();
    let field_identifiers = variant_identifiers
        .clone()
        .map(|identifier| format_ident!("properties_{}", identifier));
    let field_type_identifiers = input.variants.iter().map(|variant| &variant.type_ident);

    let new_field_identifiers = field_identifiers.clone();
    let draw_field_identifiers = field_identifiers.clone();
    let new_variant_identifiers = variant_identifiers.clone();
    let new_function_identifiers = variant_identifiers
        .clone()
        .map(|variant| format_ident!("new_{}", variant));
    let new_type_identifiers = field_type_identifiers.clone();
    let output = quote! {
        #[allow(non_snake_case)]
        struct #properties_identifier {
            #(#field_identifiers: Vec<#field_type_identifiers>), *
        }

        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        enum #enum_identifier {
            #(#variant_identifiers),*
        }

        #[derive(Clone, Debug, PartialEq, Eq)]
        struct #struct_identifier {
            discriminant: #enum_identifier,
            pointer_offset: u16,
        }

        impl #struct_identifier {
            #(
            #[allow(non_snake_case)]
            fn #new_function_identifiers(element_properties: #new_type_identifiers) -> Self {
                let boxed_properties = ::alloc::boxed::Box::new(element_properties);

                Self {
                    discriminant: #enum_identifier::#new_variant_identifiers,
                    properties_index: index
                }
            }
            )*

            fn draw(&self, properties: #properties_identifier) {
                match self.discriminant {
                    #(#enum_identifier::#draw_match_identifiers => {
                        let element_properties = &properties.#draw_field_identifiers[self.properties_index as usize];

                        ::embedded_gui::element::Draw::draw(element_properties);
                    }),*
                }
            }
        }


    };

    proc_macro::TokenStream::from(output)
}

// #[proc_macro]
// pub fn new(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
//     quote! {
//         todo!("Not yet implemented")
//     }
//     .into()
// }
