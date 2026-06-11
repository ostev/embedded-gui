use proc_macro2::{Span, TokenStream};
use quote::{TokenStreamExt, format_ident, quote, quote_spanned};
use syn::{
    Attribute, DeriveInput, Error, Expr, Field, Fields, Ident, ImplGenerics, Lifetime,
    LifetimeParam, Path, Token, Type, TypeGenerics, TypeParam, TypePath, Visibility, braced,
    parenthesized,
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
                        #(#method_calls)|*
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
                        #(#method_calls)|*
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
                            #(#method_calls)|*
                        };

                        let field_names = fields.named.iter().map(|field| field.ident.as_ref().unwrap());

                        quote! {
                            #qualified_name { #(#field_names),* } => #interior,
                        }
                    }
                    Fields::Unnamed(ref fields) => {
                        let field_names: Vec<Ident> = fields.unnamed.iter().enumerate().map(|(index, field)| Ident::new(&format!("field_{}", index), field.span())).collect();

                        let method_calls = fields.unnamed.iter().zip(field_names.iter()).map(|(field, field_name)| {
                            quote_spanned! {field.span() => ::embedded_gui::signal::Reactive::has_changed(#field_name)}
                        });

                        let interior = quote! {
                            #(#method_calls)|*
                        };

                        quote_spanned! { variant.span() =>
                            #qualified_name(#(#field_names),*) => #interior,
                        }
                    },
                    Fields::Unit => {
                        // quote_spanned! { variant.span() =>
                        //     #qualified_name => false,
                        // }
                        Error::new(Span::mixed_site(), "Enums with empty variants cannot be reactive!")
                            .to_compile_error()
                            .into()
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
    target: Type,
    event: Type,
    msg: Type,
    focus_key: Type,
    any_primitive: Type,
}

struct AnyPrimitiveArgs {
    target: Type,
}

impl AnyPrimitiveArgs {
    pub fn new(pairs: AttributeKeyValuePairs) -> AnyPrimitiveArgs {
        let mut args = AnyPrimitiveArgs {
            target: parse_quote!(Display),
        };

        for pair in pairs.pairs {
            match pair.key.to_string().as_str() {
                "target" => args.target = pair.value,
                _ => {}
            }
        }

        args
    }
}

impl AnyComponentArgs {
    pub fn new(pairs: AttributeKeyValuePairs) -> AnyComponentArgs {
        let mut args = AnyComponentArgs {
            target: parse_quote!(Display),
            event: parse_quote!(Event),
            msg: parse_quote!(Event),
            focus_key: parse_quote!(FocusKey),
            any_primitive: parse_quote!(AnyPrimitive<'a>),
        };

        for pair in pairs.pairs {
            match pair.key.to_string().as_str() {
                "target" => args.target = pair.value,
                "event" => args.event = pair.value,
                "msg" => args.msg = pair.value,
                "focus_key" => args.focus_key = pair.value,
                "any_primitive" => args.any_primitive = pair.value,
                _ => {}
            }
        }

        args
    }
}

struct EnumVariantInfo {
    qualified_name: TokenStream,
    unqualified_name: Ident,
    field_type: Type,
}

fn collect_enum_variants(
    type_name: &Ident,
    data: &syn::Data,
) -> Result<Vec<EnumVariantInfo>, TokenStream> {
    let data = match data {
        syn::Data::Enum(data) => data,
        _ => {
            return Err(
                Error::new(Span::mixed_site(), "Attribute must be used on an enum")
                    .to_compile_error()
                    .into(),
            );
        }
    };

    let mut variants = Vec::new();

    for variant in &data.variants {
        let variant_name = &variant.ident;
        let qualified_name = quote_spanned! { variant.span() => #type_name::#variant_name};

        let first_field = match variant.fields.iter().next() {
            Some(field) => field,
            None => {
                return Err(
                    Error::new(variant.span(), "All variants must contain one field")
                        .to_compile_error()
                        .into(),
                );
            }
        };

        variants.push(EnumVariantInfo {
            qualified_name,
            unqualified_name: variant_name.clone(),
            field_type: first_field.ty.clone(),
        });
    }

    Ok(variants)
}

fn build_match_interior(branches: Vec<TokenStream>) -> TokenStream {
    quote! {
        match self {
            #(#branches),*
        }
    }
}

fn build_enum_variants(variants: &[EnumVariantInfo]) -> Vec<TokenStream> {
    variants
        .iter()
        .map(|variant| {
            let unqualified_name = &variant.unqualified_name;
            let field_type = &variant.field_type;
            quote! {
                #unqualified_name(::bumpalo::boxed::Box<'a, #field_type>)
            }
        })
        .collect()
}

fn build_from_impls(
    impl_generics: &ImplGenerics,
    ty_generics: &TypeGenerics,
    where_clause: Option<&syn::WhereClause>,
    type_name: &Ident,
    lifetime_generic: &LifetimeParam,
    variants: &[EnumVariantInfo],
    param_name: &Ident,
) -> Vec<TokenStream> {
    variants
        .iter()
        .map(|variant| {
            let qualified_name = &variant.qualified_name;
            let field_type = &variant.field_type;
            quote! {
                impl #impl_generics From<::bumpalo::boxed::Box<#lifetime_generic, #field_type>> for #type_name #ty_generics #where_clause {
                    fn from(#param_name: ::bumpalo::boxed::Box<#lifetime_generic, #field_type>) -> Self {
                        #qualified_name(#param_name)
                    }
                }
            }
        })
        .collect()
}

#[proc_macro_attribute]
pub fn any_component(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let key_value_pairs = parse_macro_input!(attr as AttributeKeyValuePairs);
    let args = AnyComponentArgs::new(key_value_pairs);

    let type_name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let lifetime_generic: LifetimeParam = parse_quote!('a);

    let variants = match collect_enum_variants(&type_name, &input.data) {
        Ok(variants) => variants,
        Err(error) => return proc_macro::TokenStream::from(error),
    };

    let view_branches = variants.iter().map(|variant| {
        let qualified_name = &variant.qualified_name;
        quote! {
            #qualified_name(reference) => reference.view(v, children)
        }
    });

    let size_branches = variants.iter().map(|variant| {
        let qualified_name = &variant.qualified_name;
        quote! {
            #qualified_name(reference) => reference.intrinsic_size()
        }
    });

    let view_interior = build_match_interior(view_branches.collect());
    let size_interior = build_match_interior(size_branches.collect());

    let from_impls = build_from_impls(
        &impl_generics,
        &ty_generics,
        where_clause,
        &type_name,
        &lifetime_generic,
        &variants,
        &format_ident!("component"),
    );

    let enum_variants = build_enum_variants(&variants);

    let target = &args.target;
    let event = &args.event;
    let msg = &args.msg;
    let focus_key = &args.focus_key;
    let any_primitive = &args.any_primitive;

    // let mut where_clause_with_any = input
    //     .generics
    //     .where_clause
    //     .clone()
    //     .unwrap_or_else(|| parse_quote!(where));
    // where_clause_with_any.predicates;
    // .push(parse_quote!(AnyPrimitive: ::embedded_gui::primitive::Primitive<#target>));

    let expanded = quote! {
        // #input
        enum #type_name #ty_generics #where_clause {
            #(#enum_variants),*
        }

        impl #impl_generics ::embedded_gui::component::Component<#lifetime_generic, #target, #event, #msg, #focus_key, Self, #any_primitive> for #type_name #ty_generics #where_clause {
            fn view(
                &self,
                v: &'a ::embedded_gui::view::Factory<#event, #msg, #focus_key>,
                children: ::embedded_gui::view::Children<#lifetime_generic, #target, #event, #msg, #focus_key, Self, #any_primitive>,
            ) -> ::embedded_gui::view::View<#lifetime_generic, #target, #event, #msg, #focus_key, Self, #any_primitive> {
                #view_interior
            }
        }


        impl #impl_generics ::embedded_gui::layout::IntrinsicSize for #type_name #ty_generics #where_clause {
            #[inline]
            fn intrinsic_size(&self) -> ::embedded_gui::size::Size {
                #size_interior
            }
        }


        #(#from_impls)*

    };

    proc_macro::TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn any_primitive(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let key_value_pairs = parse_macro_input!(attr as AttributeKeyValuePairs);
    let args = AnyPrimitiveArgs::new(key_value_pairs);

    let type_name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let lifetime_generic: LifetimeParam = parse_quote!('a);

    let variants = match collect_enum_variants(&type_name, &input.data) {
        Ok(variants) => variants,
        Err(error) => return proc_macro::TokenStream::from(error),
    };

    let draw_branches = variants.iter().map(|variant| {
        let qualified_name = &variant.qualified_name;
        quote! {
            #qualified_name(reference) => reference.draw(target)
        }
    });

    let size_branches = variants.iter().map(|variant| {
        let qualified_name = &variant.qualified_name;
        quote! {
            #qualified_name(reference) => reference.intrinsic_size()
        }
    });

    let draw_interior = build_match_interior(draw_branches.collect());
    let size_interior = build_match_interior(size_branches.collect());

    let from_impls = build_from_impls(
        &impl_generics,
        &ty_generics,
        where_clause,
        &type_name,
        &lifetime_generic,
        &variants,
        &format_ident!("primitive"),
    );

    let enum_variants = build_enum_variants(&variants);

    let target = &args.target;

    let expanded = quote! {
        enum #type_name #ty_generics #where_clause {
            #(#enum_variants),*
        }

        impl #impl_generics ::embedded_gui::primitive::Primitive<#target> for #type_name #ty_generics #where_clause {
            fn draw(
                &self,
                target: &mut ::embedded_gui::draw::LocalTarget<#target>,
            ) -> Result<(), <#target as ::embedded_graphics::draw_target::DrawTarget>::Error> {
                #draw_interior
            }
        }

        impl #impl_generics ::embedded_gui::layout::IntrinsicSize for #type_name #ty_generics #where_clause {
            #[inline]
            fn intrinsic_size(&self) -> ::embedded_gui::size::Size {
                #size_interior
            }
        }

        #(#from_impls)*
    };

    proc_macro::TokenStream::from(expanded)
}
