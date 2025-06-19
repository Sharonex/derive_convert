use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{DataStruct, spanned::Spanned};

use crate::{
    attribute_parsing::{
        conversion_field::extract_convertible_fields, conversion_meta::ConversionMeta,
    },
    derive_into::build_field_conversions,
};

pub(super) fn implement_all_struct_conversions(
    data_struct: &DataStruct,
    conversions: Vec<ConversionMeta>,
) -> syn::Result<TokenStream2> {
    let named_struct = match &data_struct.fields {
        syn::Fields::Named(_) => true,
        syn::Fields::Unnamed(_) => false,
        syn::Fields::Unit => panic!("Unit structs are not supported for conversion"),
    };

    let conversion_impls: Vec<_> = conversions
        .into_iter()
        .map(|conversion| {
            implement_struct_conversion(
                conversion.clone(),
                named_struct,
                build_field_conversions(
                    &conversion,
                    named_struct,
                    true,
                    &extract_convertible_fields(
                        &data_struct.fields,
                        conversion.method,
                        &conversion.other_type(),
                    )?,
                )?,
            )
        })
        .collect::<Result<_, _>>()?;

    Ok(quote! {
        #(#conversion_impls)*
    })
}

fn implement_struct_conversion(
    meta: ConversionMeta,
    named_struct: bool,
    fields: Vec<TokenStream2>,
) -> syn::Result<TokenStream2> {
    let ConversionMeta {
        source_name,
        target_name,
        method,
        default_allowed,
    } = meta;

    if !named_struct && default_allowed {
        return Err(syn::Error::new(
            source_name.span(),
            "Default values are not supported for unnamed structs",
        ));
    }

    let default_fields = if default_allowed {
        quote! { ..Default::default() }
    } else {
        quote! {}
    };

    let inner = if named_struct {
        quote! { #target_name { #(#fields)* #default_fields } }
    } else {
        quote! { #target_name(#(#fields)* #default_fields) }
    };

    let error_type = if cfg!(feature = "anyhow") {
        quote! { anyhow::Error }
    } else {
        quote! { String }
    };

    Ok(if method.is_falliable() {
        quote! {
            impl TryFrom<#source_name> for #target_name {
                type Error = #error_type;
                fn try_from(source: #source_name) -> Result<#target_name, Self::Error> {
                    Ok(#inner)
                }
            }
        }
    } else {
        quote! {
            impl From<#source_name> for #target_name {
                fn from(source: #source_name) -> #target_name {
                    #inner
                }
            }
        }
    })
}
