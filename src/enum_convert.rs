use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::DataEnum;

use crate::{
    attribute_parsing::{
        conversion_enum::{ConversionVariant, extract_enum_variants},
        conversion_meta::ConversionMeta,
    },
    derive_into::build_field_conversions,
};

pub(super) fn implement_all_enum_conversions(
    data_enum: &DataEnum,
    conversions: Vec<ConversionMeta>,
) -> syn::Result<TokenStream2> {
    let conversion_impls: Vec<_> = conversions
        .into_iter()
        .map(|conversion| {
            let variants =
                extract_enum_variants(data_enum, conversion.method, &conversion.other_type())?;
            implement_enum_conversion(conversion.clone(), &variants)
        })
        .collect::<Result<_, _>>()?;

    Ok(quote! {
        #(#conversion_impls)*
    })
}

fn implement_enum_conversion(
    meta: ConversionMeta,
    variants: &[ConversionVariant],
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

    let variant_conversions = variants.iter().map(|variant| {
        let ConversionVariant {
            source_name: source_variant_name,
            target_name: target_variant_name,
            named_variant,
            fields,
        } = variant;

        let source_fields = fields.iter().map(|f| f.source_name.as_named());

        let field_conversions =
            build_field_conversions(&meta, *named_variant, false, fields).unwrap();

        if variant.fields.is_empty() {
            return quote! {
                #source_name::#source_variant_name => #target_name::#target_variant_name,
            };
        }

        if variant.named_variant {
            quote! {
                #source_name::#source_variant_name{ #(#source_fields),* } => #target_name::#target_variant_name {
                    #(#field_conversions)*
                    #default_fields
                },
            }
        } else {
            quote! {
                #source_name::#source_variant_name(#(#source_fields),*) => {
                    #target_name::#target_variant_name(#(#field_conversions)*)
                },
            }
        }
    });

    Ok(if method.is_falliable() {
        quote! {
            impl TryFrom<#source_name> for #target_name {
                type Error = String;
                fn try_from(source: #source_name) -> Result<#target_name, Self::Error> {
                    // Import itertools for try_collect
                    use itertools::Itertools;
                    Ok(
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
