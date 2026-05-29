use darling::FromMeta;
use proc_macro2::{Span, TokenStream};
use quote::{TokenStreamExt, format_ident, quote, quote_spanned};
use syn::{
    Attribute, DeriveInput, Error, Expr, Field, Fields, Ident, ImplGenerics, Lifetime,
    LifetimeParam, Token, Type, TypeParam, Visibility, braced, parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input, parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
    token,
};

#[proc_macro_derive(Reactive)]
pub fn derive_reactive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let has_changed = fields_have_changed(&name, &input.data);

    let expanded = quote! {
        impl #impl_generics ::embedded_gui::signal::Reactive for #name #ty_generics #where_clause {
            fn has_changed(&self) -> bool {
                #has_changed
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

fn fields_have_changed(type_name: &Ident, data: &syn::Data) -> TokenStream {
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
        syn::Data::Enum(ref data) => {
            let branches = data.variants.iter().map(|variant| {
                let variant_name = &variant.ident;
                let qualified_name = quote_spanned! { variant.span() => #type_name::#variant_name};

                match variant.fields {
                    Fields::Named(ref fields) => {
                        let method_calls = fields.named.iter().map(|field| {
                            let field_name = field.ident.as_ref().unwrap();

                            quote_spanned! {field.span() => ::embedded_gui::signal::Reactive::has_changed(#field_name)}
                        });

                        let interior = quote! {
                            #(#method_calls)&*
                        };

                        let field_names = fields.named.iter().map(|field| field.ident.as_ref().unwrap());

                        quote! {
                            #qualified_name { #(#field_names),* } => #interior
                        }
                    }
                    Fields::Unnamed(ref fields) => {
                        let field_names: Vec<Ident> = fields.unnamed.iter().enumerate().map(|(index, field)| Ident::new(&format!("field_{}", index), field.span())).collect();

                        let method_calls = fields.unnamed.iter().zip(field_names.iter()).map(|(field, field_name)| {
                            quote_spanned! {field.span() => ::embedded_gui::signal::Reactive::has_changed(#field_name)}
                        });

                        let interior = quote! {
                            #(#method_calls)&*
                        };

                        quote_spanned! { variant.span() =>
                            #qualified_name(#(#field_names),*) => #interior
                        }
                    },
                    Fields::Unit => {
                        quote_spanned! { variant.span() =>
                            #qualified_name => false
                        }
                    },
                }
            });

            quote! {
                match self {
                    #(#branches),*
                    _ => false
                }
            }
        }
        syn::Data::Union(_) => unimplemented!(),
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
            fn mark_resolved(&mut self) {
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

#[derive(FromMeta)]
#[darling(derive_syn_parse)]
struct AnyComponentArgs {
    #[darling(default = || Ident::new("a", Span::mixed_site()))]
    lifetime_name: Ident,

    target: Type,
    event: Type,
    msg: Type,
    focus_key: Type,
}

#[proc_macro_attribute]
pub fn any_component(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args: AnyComponentArgs = match syn::parse(attr) {
        Ok(v) => v,
        Err(e) => {
            return e.to_compile_error().into();
        }
    };

    let input = parse_macro_input!(input as DeriveInput);

    let type_name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let lifetime_generic = LifetimeParam::new(Lifetime::new(
        &args.lifetime_name.to_string(),
        args.lifetime_name.span(),
    ));

    let mut field_types = Vec::new();

    let interior = match input.data {
        syn::Data::Enum(data) => {
            let branches = data.variants.iter().map(|variant| {
                let variant_name = &variant.ident;
                let qualified_name = quote_spanned! { variant.span() => #type_name::#variant_name};

                let first_field = match variant.fields.iter().next() {
                    Some(field) => field,
                    None => {
                        return Error::new(variant.span(), "All variants must contain fields")
                            .to_compile_error()
                            .into();
                    }
                };
                field_types.push((qualified_name.clone(), first_field.ty.clone()));

                quote! {
                    #qualified_name(reference) => reference.view(v, children)
                }
            });

            quote! {
                match self {
                    #(#branches),*
                    _ => {}
                }
            }
        }
        _ => unimplemented!(),
    };

    // let mut from_generics = input.generics.clone();
    // from_generics
    //     .params
    //     .push(parse_quote!(FromAnyComponentType));

    let from_impls = field_types.iter().map(|(variant_name, field_type)| {
        quote! {
            impl #impl_generics From<::bumpalo::boxed::Box<#lifetime_generic, #field_type>> for #type_name #ty_generics #where_clause {
                fn from(component: ::bumpalo::boxed::Box<#lifetime_generic, #field_type>) -> Self {
                    #variant_name(component)
                }
            }
        }
    });

    let expanded = quote! {
        impl #impl_generics ::embedded_gui::app::Component<#lifetime_generic, #{args.target}, #{args.event}, #{args.msg}, #{args.focus_key}> for #type_name #ty_generics #where_clause {
            fn view(
                self,
                v: &'a Factory<#{args.focus_key}, #{args.event}, #{args.msg}>,
                children: &'a mut [Widget<#lifetime_generic, #{args.target}, #{args.event}, #{args.msg}, #{args.focus_key}>],
            ) -> View<#lifetime_generic, #{args.target}, #{args.event}, #{args.msg}, #{args.focus_key}> {
                #interior
            }
        }

        #(#from_impls)*
    };

    proc_macro::TokenStream::from(expanded)
}
