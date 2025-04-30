use itertools::Itertools;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::DataEnum;

use crate::derive_convert::{
    ConversionMeta, ConvertibleField, build_convertible_field, build_field_conversions,
};

#[derive(Clone)]
struct ConversionVariant {
    source_name: syn::Ident,
    target_name: syn::Ident,
    named_variant: bool,
    fields: Vec<ConvertibleField>,
}

pub(super) fn implement_all_enum_conversions(
    data_enum: &DataEnum,
    conversions: Vec<ConversionMeta>,
) -> syn::Result<TokenStream2> {
    let conversion_impls: Vec<_> = conversions
        .into_iter()
        .map(|conversion| {
            let variants = extract_enum_variants(data_enum, &conversion)?;
            implement_enum_conversion(conversion.clone(), &variants)
        })
        .try_collect()?;

    Ok(quote! {
        #(#conversion_impls)*
    })
}
fn extract_enum_variants(
    data_enum: &DataEnum,
    meta: &ConversionMeta,
) -> syn::Result<Vec<ConversionVariant>> {
    data_enum
        .variants
        .iter()
        .map(|variant| {
            let (fields, named_variant) = match &variant.fields {
                syn::Fields::Named(fields_named) => {
                    (fields_named.named.iter().cloned().collect(), true)
                }
                syn::Fields::Unnamed(fields_unnamed) => {
                    (fields_unnamed.unnamed.iter().cloned().collect(), false)
                }
                syn::Fields::Unit => (Vec::new(), false),
            };

            Ok(ConversionVariant {
                source_name: variant.ident.clone(),
                target_name: variant.ident.clone(),
                named_variant,
                fields: fields
                    .iter()
                    .enumerate()
                    .map(|(index, f)| build_convertible_field(f, meta, index))
                    .try_collect()?,
            })
        })
        .try_collect()
}

fn implement_enum_conversion(
    meta: ConversionMeta,
    variants: &Vec<ConversionVariant>,
) -> syn::Result<TokenStream2> {
    let ConversionMeta {
        source_name,
        target_name,
        method,
        default_allowed,
    } = meta.clone();

    let default_fields = if default_allowed {
        quote! { ..Default::default() }
    } else {
        quote! {}
    };

    let variant_conversions = variants.into_iter().map(|variant| {
        let ConversionVariant {
            source_name: source_variant_name,
            target_name: target_variant_name,
            named_variant,
            fields,
        } = variant;

        let source_fields = fields.iter().map(|f| f.source_name.as_named());

        let field_conversions =
            build_field_conversions(&meta, *named_variant, false, fields).unwrap();

        if variant.named_variant {
            quote! {
                #source_name::#source_variant_name{ #(#source_fields),* } => #target_name::#target_variant_name {
                    #(#field_conversions)*
                    #default_fields
                }
            }
        } else {
            quote! {
                #source_name::#source_variant_name(#(#source_fields),*) => {
                    #target_name::#target_variant_name(#(#field_conversions)*)
                }
            }
        }
    });

    Ok(if method.is_falliable() {
        quote! {
            impl TryFrom<#source_name> for #target_name {
                type Error = anyhow::Error;
                fn try_from(source: #source_name) -> Result<#target_name, Self::Error> {
                    // Import itertools for try_collect
                    use itertools::Itertools;
                    anyhow::Ok(
                        match source {
                            #(#variant_conversions)*
                        }
                    )
                }
            }
        }
    } else {
        quote! {
            impl From<#source_name> for #target_name {
                fn from(source: #source_name) -> #target_name {
                    match source {
                        #(#variant_conversions)*
                    }
                }
            }
        }
    })
}
