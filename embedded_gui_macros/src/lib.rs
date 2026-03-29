use std::env::var;

use proc_macro2::{Span, TokenStream};
use quote::{TokenStreamExt, format_ident, quote, quote_spanned};
use syn::{
    DeriveInput, Expr, Field, Fields, Ident, Token, Visibility, braced, parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    spanned::Spanned,
    token,
};

struct WidgetVariant {
    variant_ident: Ident,
    paren_token: syn::token::Paren,
    type_ident: Ident,
}

impl Parse for WidgetVariant {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;

        Ok(WidgetVariant {
            variant_ident: input.parse()?,
            paren_token: parenthesized!(content in input),
            type_ident: content.parse()?,
        })
    }
}

struct TypeInput {
    visibility: Visibility,
    ident: Ident,
    brace_token: token::Brace,
    variants: Punctuated<WidgetVariant, Token![,]>,
}

impl TypeInput {
    fn declaration(&self) -> TokenStream {
        let enum_identifier = &self.ident;
        let visibility = &self.visibility;

        let variant_identifiers = self.variants.iter().map(|variant| &variant.variant_ident);
        let match_identifiers = variant_identifiers.clone();
        let properties = spanned_properties(self.variants.iter());

        let type_identifiers = self.variants.iter().map(|variant| &variant.type_ident);

        let macro_variant_identifiers = variant_identifiers
            .clone()
            .map(|ident| Ident::new(&lowercase_first_letter(&ident.to_string()), ident.span()));

        quote! {
            #visibility enum #enum_identifier<'a> {
                #(#variant_identifiers(&'a #type_identifiers)),*
            }

            impl<'a> ::embedded_gui::signal::Reactive for #enum_identifier<'a> {
                fn has_changed(&self) -> bool {
                    match self {
                        #(#enum_identifier::#match_identifiers(properties) =>
                            ::embedded_gui::signal::Reactive::has_changed(*#properties)),*
                    }
                }
            }
        }
    }
}

impl Parse for TypeInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;

        let visibility = input.parse()?;
        let ident: Ident = input.parse()?;
        let brace_token = braced!(content in input);

        let variants = {
            let variants = content.parse_terminated(WidgetVariant::parse, Token![,])?;

            if variants.len() > 0 {
                Ok(variants)
            } else {
                Err(syn::Error::new(
                    ident.span(),
                    "expected at least one variant",
                ))
            }?
        };

        Ok(TypeInput {
            visibility,
            ident,
            brace_token,
            variants,
        })
    }
}

struct WidgetsInput {
    primitives: TypeInput,
    components: TypeInput,
}

impl Parse for WidgetsInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(WidgetsInput {
            primitives: input.parse()?,
            components: input.parse()?,
        })
    }
}

fn lower_variant_identifiers<'a>(
    variants: impl Iterator<Item = &'a WidgetVariant>,
) -> impl Iterator<Item = Ident> {
    variants.map(|variant| {
        Ident::new(
            &lowercase_first_letter(&variant.variant_ident.to_string()),
            variant.variant_ident.span(),
        )
    })
}

fn spanned_properties<'a>(
    variants: impl Iterator<Item = &'a WidgetVariant>,
) -> impl Iterator<Item = TokenStream> {
    variants
        .map(|variant| &variant.type_ident)
        .map(|identifier| quote_spanned! {identifier.span()=> properties})
}

#[proc_macro]
pub fn widgets(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(item as WidgetsInput);

    let primitive_variant_identifiers = lower_variant_identifiers(input.primitives.variants.iter());
    let component_variant_identifiers = lower_variant_identifiers(input.primitives.variants.iter());

    let primitives_type_name = &input.primitives.ident;
    let components_type_name = &input.components.ident;

    let primitives_declaration = input.primitives.declaration();
    let components_declaration = input.components.declaration();

    let primitive_properties = spanned_properties(input.primitives.variants.iter());
    let draw_match_identifiers = input
        .primitives
        .variants
        .iter()
        .map(|variant| &variant.variant_ident);

    let primitive_trait_impl = quote! {
        impl<'a> ::embedded_gui::primitive::Primitive for #primitives_type_name<'a> {
            fn draw(&self, target: impl ::embedded_gui::draw::Target) {
                match self {
                    #(#primitives_type_name::#draw_match_identifiers(properties) => {
                        ::embedded_gui::primitive::Primitive::draw(*#primitive_properties, target);
                    }),*
                }
            }
        }
    };

    let component_properties = spanned_properties(input.components.variants.iter());
    let view_match_identifiers = input
        .components
        .variants
        .iter()
        .map(|variant| &variant.variant_ident);

    let components_trait_impl = quote! {
        impl<'a> ::embedded_gui::component::Component<'a, #primitives_type_name<'a>> for #components_type_name<'a> {
            fn view(&self, bump: &::bumpalo::Bump, children: ::embedded_gui::view::View<'a, #primitives_type_name<'a>, #components_type_name<'a>>) {
                match self {
                    #(#components_type_name::#view_match_identifiers(properties) => {
                        ::embedded_gui::component::Component::view(*#component_properties, bump, children);
                    }),*
                }
            }
        }
    };

    let output = quote! {
        #primitives_declaration
        #primitive_trait_impl

        #components_declaration
        #components_trait_impl

        pub struct Factory {

        }

        impl Factory {
        }

        // #(macro_rules! #macro_variant_identifiers {
        //     ($bump:expr, $expr:expr) => {
        //         $crate::Primitives::Button(::bumpalo::Bump::alloc($bump, $expr))
        //     };
        // })*

        // macro_rules! primitive {
        //     ($bump:expr, $($name:ident, $($field_name:ident: $field_value:expr),*),*) => {
        //         let props = $name {
        //             $($field_name: $field_value),*
        //         };
        //         if ::embedded_gui::signal::Reactive::has_changed(props) {

        //         } else {

        //         }
        //     };
        // }

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

fn lowercase_first_letter(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_lowercase().collect::<String>() + c.as_str(),
    }
}
