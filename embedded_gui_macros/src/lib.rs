use proc_macro2::{Span, TokenStream};
use quote::{TokenStreamExt, format_ident, quote, quote_spanned};
use syn::{
    DeriveInput, Expr, Field, Fields, Ident, Token, braced, parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    spanned::Spanned,
    token,
};

struct PrimitivesVariant {
    variant_ident: Ident,
    paren_token: syn::token::Paren,
    type_ident: Ident,
}

impl Parse for PrimitivesVariant {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;

        Ok(PrimitivesVariant {
            variant_ident: input.parse()?,
            paren_token: parenthesized!(content in input),
            type_ident: content.parse()?,
        })
    }
}

struct PrimitivesInput {
    ident: Ident,
    brace_token: token::Brace,
    variants: Punctuated<PrimitivesVariant, Token![,]>,
}

impl Parse for PrimitivesInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;

        Ok(PrimitivesInput {
            ident: input.parse()?,
            brace_token: braced!(content in input),
            variants: content.parse_terminated(PrimitivesVariant::parse, Token![,])?,
        })
    }
}

#[proc_macro]
pub fn primitives(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(item as PrimitivesInput);

    let struct_identifier = input.ident;
    let enum_identifier = format_ident!("{}Discriminant", struct_identifier);
    let properties_identifier = format_ident!("{}Properties", struct_identifier);

    let variant_identifiers = input.variants.iter().map(|variant| &variant.variant_ident);
    let draw_match_identifiers = variant_identifiers.clone();
    let field_identifiers = variant_identifiers
        .clone()
        .map(|identifier| format_ident!("properties_{}", identifier));
    let field_type_identifiers = input.variants.iter().map(|variant| &variant.type_ident);

    let draw_field_identifiers = field_identifiers.clone();
    let new_variant_identifiers = variant_identifiers.clone();
    let new_function_identifiers = variant_identifiers
        .clone()
        .map(|variant| format_ident!("new_{}", variant));
    let new_type_identifiers = field_type_identifiers.clone();
    let output = quote! {
        #[allow(non_snake_case)]
        struct #properties_identifier<'a> {
            #(#field_identifiers: ::embedded_gui::arena::Arena<'a, #field_type_identifiers>), *
        }

        enum #enum_identifier {
            #(#variant_identifiers),*
        }

        struct #struct_identifier<'a> {
            discriminant: #enum_identifier,
            properties: ::embedded_gui::arena::UntypedPointer<'a>,
        }

        impl<'a> #struct_identifier<'a> {
            #(
            #[allow(non_snake_case)]
            fn #new_function_identifiers(arena: &mut ::embedded_gui::arena::Arena<'a, #new_type_identifiers>, element_properties: #new_type_identifiers) -> Self {
                Self {
                    discriminant: #enum_identifier::#new_variant_identifiers,
                    properties: arena.push(element_properties).into()
                }
            }
            )*

            fn draw(&self, properties: #properties_identifier<'a>, target: impl ::embedded_gui::draw::Target) {
                match self.discriminant {
                    #(#enum_identifier::#draw_match_identifiers => {
                        let element_properties = &properties.#draw_field_identifiers[self.properties];

                        ::embedded_gui::primitive::Primitive::draw(element_properties, target);
                    }),*
                }
            }
        }


    };

    proc_macro::TokenStream::from(output)
}

#[proc_macro_derive(Reactive)]
pub fn derive_reactive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let has_changed = fields_have_changed(&input.data);

    let expanded = quote! {
        impl #impl_generics ::embedded_gui::signal::Reactive for #name #ty_generics #where_clause {
            fn has_changed(&self) -> bool {
                #has_changed
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

fn fields_have_changed(data: &syn::Data) -> TokenStream {
    match *data {
        syn::Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                if fields.named.len() > 0 {
                    let recurse = fields.named.iter().map(|field| {
                        let name = &field.ident;
                        quote_spanned! {field.span() =>
                            ::embedded_gui::signal::Reactive::has_changed(&self.#name)
                        }
                    });
                    quote! {
                        #(#recurse)&*
                    }
                } else {
                    quote!(false)
                }
            }
            Fields::Unnamed(ref fields) => {
                if fields.unnamed.len() > 0 {
                    let recurse = fields.unnamed.iter().enumerate().map(|(i, field)| {
                        let index = syn::Index::from(i);
                        quote_spanned! {field.span() =>
                            ::embedded_gui::signal::Reactive::has_changed(&self.#index)
                        }
                    });
                    quote! {
                        #(#recurse)&*
                    }
                } else {
                    quote!(false)
                }
            }
            Fields::Unit => {
                quote!(false)
            }
        },
        syn::Data::Enum(_) | syn::Data::Union(_) => unimplemented!(),
    }
}
