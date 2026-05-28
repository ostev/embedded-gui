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
                    let method_calls = fields.named.iter().map(|field| {
                        let name = &field.ident;
                        quote_spanned! {field.span() =>
                            ::embedded_gui::signal::Reactive::has_changed(&self.#name)
                        }
                    });
                    quote! {
                        #(#method_calls)&*
                    }
                } else {
                    quote!(false)
                }
            }
            Fields::Unnamed(ref fields) => {
                if fields.unnamed.len() > 0 {
                    let method_calls = fields.unnamed.iter().enumerate().map(|(i, field)| {
                        let index = syn::Index::from(i);
                        quote_spanned! {field.span() =>
                            ::embedded_gui::signal::Reactive::has_changed(&self.#index)
                        }
                    });
                    quote! {
                        #(#method_calls)&*
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

#[proc_macro_derive(State)]
pub fn derive_state(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let mark_resolved = mark_fields_resolved(&input.data);

    let expanded = quote! {
        impl #impl_generics ::embedded_gui::app::State for #name #ty_generics #where_clause {
            fn has_changed(&self) -> bool {
                #mark_resolved
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

fn mark_fields_resolved(data: &syn::Data) -> TokenStream {
    match *data {
        syn::Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                if fields.named.len() > 0 {
                    let method_calls = fields.named.iter().map(|field| {
                        let name = &field.ident;
                        quote_spanned! {field.span() =>
                            ::embedded_gui::app::State::mark_resolved(&mut self.#name);
                        }
                    });
                    quote! {
                        #(#method_calls)*
                    }
                } else {
                    quote!(false)
                }
            }
            Fields::Unnamed(ref fields) => {
                let method_calls = fields.unnamed.iter().enumerate().map(|(i, field)| {
                    let index = syn::Index::from(i);
                    quote_spanned! {field.span() =>
                        ::embedded_gui::app::State::mark_resolved(&mut self.#index);
                    }
                });
                quote! {
                    #(#method_calls)*
                }
            }
            Fields::Unit => {
                quote!({})
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
