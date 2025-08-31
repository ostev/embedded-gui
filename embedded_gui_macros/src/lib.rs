use proc_macro2::{Span, TokenStream};
use quote::{TokenStreamExt, format_ident, quote};
use syn::{
    DeriveInput, Field, Fields, Ident, Token, braced,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    spanned::Spanned,
    token,
};

struct DrawInstructionDeclaration {
    variants: Punctuated<DrawInstructionVariant, Token![,]>,
}

struct DrawInstructionVariant {
    variant_ident: Ident,
    brace_token: token::Brace,
    fields: Punctuated<Field, Token![,]>,
    arrow_token: Token![=>],
    draw_fn_ident: Ident,
}

impl Parse for DrawInstructionDeclaration {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(DrawInstructionDeclaration {
            variants: input.parse_terminated(DrawInstructionVariant::parse, Token![,])?,
        })
    }
}

impl Parse for DrawInstructionVariant {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;

        Ok(DrawInstructionVariant {
            variant_ident: input.parse()?,
            brace_token: braced!(content in input),
            fields: content.parse_terminated(Field::parse_named, Token![,])?,
            arrow_token: input.parse()?,
            draw_fn_ident: input.parse()?,
        })
    }
}

// fn parse_enum_variants<'ast>(
//     enum_data: &'ast syn::DataEnum,
// ) -> Result<Vec<DrawInstructionVariant<'ast>>, syn::Error> {
//     enum_data.variants.iter().map(|variant| {
//         if variant.attrs.len() > 0 {
//             Err(syn::Error::new(variant.span(), "Attributes are not supported within a draw instruction declaration."))
//         } else if variant.discriminant.is_some() {
//             Err(syn::Error::new(variant.span(), "Explicit discriminants are not supported within a draw instruction declaration."))
//         } else {
//             Ok(DrawInstructionVariant {
//                 span: variant.span(),
//                 identifier: &variant.ident,
//                 fields: &variant.fields
//             })
//         }
//     }).collect()
// }

// #[proc_macro_attribute]
// pub fn draw_instruction(
//     _attribute: proc_macro::TokenStream,
//     item: proc_macro::TokenStream,
// ) -> proc_macro::TokenStream {
//     let input = parse_macro_input!(item as DeriveInput);

//     let result = match input.data {
//         syn::Data::Enum(ref enum_data) => match parse_enum_variants(enum_data) {
//             Ok(variants) => {
//                 let struct_declarations = variants
//                     .iter()
//                     .map(|variant| {
//                         let identifier =
//                             format_ident!("DrawInstruction{}PropertiesContainer", variant.identifier);

//                         let field_types = variant.fields.iter().map(|field| &field.ty);

//                         if variant
//                             .fields
//                             .iter()
//                             .all(|field| field.ident.is_some() == true)
//                         {
//                             let field_identifiers = variant
//                                 .fields
//                                 .iter()
//                                 .map(|field| field.ident.as_ref().unwrap());
//                             quote! {
//                                 struct #identifier {
//                                     #(#field_identifiers: #field_types),*
//                                 }
//                             }
//                         } else if variant
//                             .fields
//                             .iter()
//                             .all(|field| field.ident.is_none() == true)
//                         {
//                             quote! {
//                                 struct #identifier(#(#field_types),*);
//                             }
//                         } else {
//                             syn::Error::new(
//                                 variant.span,
//                                 "Enum variants within a draw instruction declaration are not allowed to mix named and unnamed fields."
//                             ).to_compile_error()
//                         }
//                     })
//                     .fold(TokenStream::new(), |mut token_stream, variant| {
//                         token_stream.extend(variant);
//                         token_stream
//                     });

//                 let struct_identifier = input.ident;
//                 let enum_identifier = format_ident!("{}Discriminant", struct_identifier);
//                 let variant_identifiers = variants.iter().map(|variant| variant.identifier);
//                 let field_identifiers = variant_identifiers.clone().map(|identifier| format_ident!("properties_{}", identifier));
//                 let field_type_identifiers = variants.iter().map(|variant| format_ident!("DrawInstruction{}PropertiesContainer", variant.identifier));

//                 quote! {
//                     #struct_declarations

//                     #[allow(non_snake_case)]
//                     struct Properties {
//                         #(#field_identifiers: Vec<#field_type_identifiers>), *
//                     }

//                     enum #enum_identifier {
//                         #(#variant_identifiers),*
//                     }

//                     struct #struct_identifier {
//                         discriminant: #enum_identifier,
//                         properties_index: u16,
//                     }
//                 }
//             }
//             Err(error) => error.to_compile_error(),
//         },
//         _ => syn::Error::new(
//             input.span(),
//             "Only enums are supported within a draw instruction declaration.",
//         )
//         .to_compile_error(),
//     };

//     proc_macro::TokenStream::from(result)
// }
