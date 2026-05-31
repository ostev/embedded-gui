use proc_macro2::{Span, TokenStream};
use quote::{TokenStreamExt, format_ident, quote, quote_spanned};
use syn::{
    Attribute, DeriveInput, Error, Expr, Field, Fields, Ident, ImplGenerics, Lifetime,
    LifetimeParam, Path, Token, Type, TypeParam, TypePath, Visibility, braced, parenthesized,
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
                            #qualified_name(#(#field_names),*) => #interior,
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
                    #(#branches)*
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

// #[derive(FromMeta)]
// #[darling(derive_syn_parse)]

struct KeyValuePair {
    key: Ident,
    eq_token: Token![=],
    value: Type,
}

impl Parse for KeyValuePair {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(KeyValuePair {
            key: input.parse()?,
            eq_token: input.parse()?,
            value: input.parse()?,
        })
    }
}

struct AttributeKeyValuePairs {
    pairs: Punctuated<KeyValuePair, Token![,]>,
}

impl Parse for AttributeKeyValuePairs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            pairs: Punctuated::parse_terminated(input)?,
        })
    }
}

struct AnyComponentArgs {
    // #[darling(default = || Ident::new("a", Span::mixed_site()))]
    // lifetime_name: LifetimeParam,
    target: Type,
    event: Type,
    msg: Type,
    focus_key: Type,
}

impl AnyComponentArgs {
    pub fn new(pairs: AttributeKeyValuePairs) -> AnyComponentArgs {
        // let mut lifetime_name = LifetimeParam::new(Lifetime::new(symbol, span));
        let mut args = AnyComponentArgs {
            target: parse_quote!(Display),
            event: parse_quote!(Event),
            msg: parse_quote!(Event),
            focus_key: parse_quote!(FocusKey),
        };

        for pair in pairs.pairs {
            match pair.key.to_string().as_str() {
                "target" => args.target = pair.value,
                "event" => args.event = pair.value,
                "msg" => args.msg = pair.value,
                "focus_key" => args.focus_key = pair.value,
                _ => {}
            }
        }

        args
    }
}

#[proc_macro_attribute]
pub fn any_component(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // let args: AnyComponentArgs = match syn::parse(attr) {
    //     Ok(v) => v,
    //     Err(e) => {
    //         return e.to_compile_error().into();
    //     }
    // };

    let input = parse_macro_input!(input as DeriveInput);

    let key_value_pairs = parse_macro_input!(attr as AttributeKeyValuePairs);
    let args = AnyComponentArgs::new(key_value_pairs);

    let type_name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // let lifetime_generic = LifetimeParam::new(Lifetime::new(
    //     &args.lifetime_name.to_string(),
    //     args.lifetime_name.span(),
    // ));

    let lifetime_generic: LifetimeParam = parse_quote!('a);

    let mut field_types = Vec::new();

    let (view_interior, size_interior) = match input.data {
        syn::Data::Enum(data) => {
            let (view_branches, size_branches): (Vec<TokenStream>, Vec<TokenStream>) = data
                .variants
                .iter()
                .map(|variant| {
                    let variant_name = &variant.ident;
                    let qualified_name =
                        quote_spanned! { variant.span() => #type_name::#variant_name};

                    let first_field = match variant.fields.iter().next() {
                        Some(field) => field,
                        None => {
                            let error: TokenStream =
                                Error::new(variant.span(), "All variants must contain one field")
                                    .to_compile_error()
                                    .into();

                            return (error.clone(), error);
                        }
                    };
                    field_types.push((
                        qualified_name.clone(),
                        variant_name.clone(),
                        first_field.ty.clone(),
                    ));

                    (
                        quote! {
                            #qualified_name(reference) => reference.view(v, children)
                        },
                        quote! {
                            #qualified_name(reference) => reference.intrinsic_size()
                        },
                    )
                })
                .unzip();

            (
                quote! {
                    match self {
                        #(#view_branches),*
                    }
                },
                quote! {
                    match self {
                        #(#size_branches),*
                    }
                },
            )
        }
        _ => unimplemented!(),
    };

    // let mut from_generics = input.generics.clone();
    // from_generics
    //     .params
    //     .push(parse_quote!(FromAnyComponentType));

    let from_impls = field_types.iter().map(|(qualified_name, unqualified_name, field_type)| {
        quote! {
            impl #impl_generics From<::bumpalo::boxed::Box<#lifetime_generic, #field_type>> for #type_name #ty_generics #where_clause {
                fn from(component: ::bumpalo::boxed::Box<#lifetime_generic, #field_type>) -> Self {
                    #qualified_name(component)
                }
            }
        }
    });

    let enum_variants = field_types
        .iter()
        .map(|(qualified_name, unqualified_name, field_type)| {
            quote! {
                #unqualified_name(::bumpalo::boxed::Box<'a, #field_type>)
            }
        });

    let target = &args.target;
    let event = &args.event;
    let msg = &args.msg;
    let focus_key = &args.focus_key;

    let expanded = quote! {
        // #input
        enum #type_name #ty_generics #where_clause {
            #(#enum_variants),*
        }

        impl #impl_generics ::embedded_gui::component::Component<#lifetime_generic, #target, #event, #msg, #focus_key, Self> for #type_name #ty_generics #where_clause {
            fn view(
                &self,
                v: &'a ::embedded_gui::view::Factory<#event, #msg, #focus_key>,
                children: ::embedded_gui::view::Children<#lifetime_generic, #target, #event, #msg, #focus_key, Self>,
            ) -> ::embedded_gui::view::View<#lifetime_generic, #target, #event, #msg, #focus_key, Self> {
                #view_interior
            }
        }


        impl #impl_generics ::embedded_gui::layout::IntrinsicSize for #type_name #ty_generics #where_clause {
            #[inline]
            fn intrinsic_size(&self) -> embedded_gui::size::Size {
                #size_interior
            }
        }


        #(#from_impls)*

    };

    proc_macro::TokenStream::from(expanded)
}
