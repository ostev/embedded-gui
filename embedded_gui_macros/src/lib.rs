use proc_macro2::{Span, TokenStream};
use quote::{TokenStreamExt, format_ident, quote};
use syn::{DeriveInput, Fields, Ident, parse_macro_input, spanned::Spanned};

struct DrawInstructionVariant<'ast> {
    identifier: &'ast Ident,
    fields: &'ast Fields,
    span: Span,
}

fn parse_enum_variants<'ast>(
    enum_data: &'ast syn::DataEnum,
) -> Result<Vec<DrawInstructionVariant<'ast>>, syn::Error> {
    enum_data.variants.iter().map(|variant| {
        if variant.attrs.len() > 0 {
            Err(syn::Error::new(variant.span(), "Attributes are not supported within a draw instruction declaration."))
        } else if variant.discriminant.is_some() {
            Err(syn::Error::new(variant.span(), "Explicit discriminants are not supported within a draw instruction declaration."))
        } else {
            Ok(DrawInstructionVariant {
                span: variant.span(),
                identifier: &variant.ident,
                fields: &variant.fields
            })
        }
    }).collect()
}

#[proc_macro_attribute]
pub fn draw_instruction(
    _attribute: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    let result = match input.data {
        syn::Data::Enum(ref enum_data) => match parse_enum_variants(enum_data) {
            Ok(variants) => {
                let struct_declarations = variants
                    .iter()
                    .map(|variant| {
                        let identifier =
                            format_ident!("DrawInstruction{}Properties", variant.identifier);

                        let field_types = variant.fields.iter().map(|field| &field.ty);

                        if variant
                            .fields
                            .iter()
                            .all(|field| field.ident.is_some() == true)
                        {
                            let field_identifiers = variant
                                .fields
                                .iter()
                                .map(|field| field.ident.as_ref().unwrap());
                            quote! {
                                struct #identifier {
                                    #(#field_identifiers: #field_types),*
                                }
                            }
                        } else if variant
                            .fields
                            .iter()
                            .all(|field| field.ident.is_none() == true)
                        {
                            quote! {
                                struct #identifier(#(#field_types),*);
                            }
                        } else {
                            syn::Error::new(
                                variant.span, 
                                "Enum variants within a draw instruction declaration are not allowed to mix named and unnamed fields."
                            ).to_compile_error()
                        }
                    })
                    .fold(TokenStream::new(), |mut token_stream, variant| {
                        token_stream.extend(variant);
                        token_stream
                    });

                quote! { #struct_declarations }
            }
            Err(error) => error.to_compile_error(),
        },
        _ => syn::Error::new(
            input.span(),
            "Only enums are supported within a draw instruction declaration.",
        )
        .to_compile_error(),
    };

    proc_macro::TokenStream::from(result)
}
