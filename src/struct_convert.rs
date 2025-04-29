use itertools::Itertools;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{DataStruct, Field};

use crate::derive_convert::{ConversionMeta, build_field_conversions};

pub(super) fn implement_all_struct_conversions(
    data_struct: &DataStruct,
    conversions: Vec<ConversionMeta>,
) -> syn::Result<TokenStream2> {
    let fields = extract_struct_fields(data_struct)?;
    let conversion_impls: Vec<_> = conversions
        .into_iter()
        .map(|conversion| {
            implement_struct_conversion(
                conversion.clone(),
                build_field_conversions(conversion, &fields)?,
            )
        })
        .try_collect()?;

    Ok(quote! {
        #(#conversion_impls)*
    })
}

fn extract_struct_fields(data_struct: &DataStruct) -> syn::Result<Vec<Field>> {
    // Extract fields based on the struct type (named, unnamed, or unit)
    match &data_struct.fields {
        // For named fields (regular structs with named fields)
        syn::Fields::Named(fields_named) => Ok(fields_named.named.iter().cloned().collect()),
        // For unnamed fields (tuple structs)
        syn::Fields::Unnamed(fields_unnamed) => {
            Ok(fields_unnamed.unnamed.iter().cloned().collect())
        }
        // For unit structs (no fields)
        syn::Fields::Unit => Ok(Vec::new()),
    }
}

fn implement_struct_conversion(
    meta: ConversionMeta,
    fields: Vec<TokenStream2>,
) -> syn::Result<TokenStream2> {
    let ConversionMeta {
        source_name,
        target_name,
        method,
        default_allowed,
    } = meta;

    let default_fields = if default_allowed {
        quote! { ..Default::default() }
    } else {
        quote! {}
    };

    Ok(if method.is_falliable() {
        quote! {
            impl TryFrom<#source_name> for #target_name {
                type Error = anyhow::Error;
                fn try_from(source: #source_name) -> Result<#target_name, Self::Error> {
                    // Import itertools for try_collect
                    use itertools::Itertools;

                    anyhow::Ok(#target_name {
                        #(#fields)*
                        #default_fields
                    })
                }
            }
        }
    } else {
        quote! {
            impl From<#source_name> for #target_name {
                fn from(source: #source_name) -> #target_name {
                    #target_name {
                        #(#fields)*
                        #default_fields
                    }
                }
            }
        }
    })
}
