use itertools::Itertools;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{DataStruct, spanned::Spanned};

use crate::derive_convert::{ConversionMeta, build_field_conversions};

pub(super) fn implement_all_struct_conversions(
    data_struct: &DataStruct,
    conversions: Vec<ConversionMeta>,
) -> syn::Result<TokenStream2> {
    let (fields, named_struct) = match &data_struct.fields {
        syn::Fields::Named(fields_named) => (fields_named.named.iter().cloned().collect(), true),
        syn::Fields::Unnamed(fields_unnamed) => {
            (fields_unnamed.unnamed.iter().cloned().collect(), false)
        }
        syn::Fields::Unit => panic!("Unit structs are not supported for conversion"),
    };

    let conversion_impls: Vec<_> = conversions
        .into_iter()
        .map(|conversion| {
            implement_struct_conversion(
                conversion.clone(),
                named_struct,
                build_field_conversions(conversion, named_struct, &fields)?,
            )
        })
        .try_collect()?;

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

    Ok(if method.is_falliable() {
        quote! {
            impl TryFrom<#source_name> for #target_name {
                type Error = anyhow::Error;
                fn try_from(source: #source_name) -> Result<#target_name, Self::Error> {
                    // Import itertools for try_collect
                    use itertools::Itertools;

                    anyhow::Ok(#inner)
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
