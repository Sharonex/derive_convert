use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, quote_spanned};
use syn::{DeriveInput, Error, Field, Ident, Path, spanned::Spanned};

use crate::util::{
    extract_single_path_attribute, field_has_attribute, get_attribute_value, get_field_value,
    has_attribute, is_surrounding_type,
};

enum FieldConversionMethod {
    Plain,
    UnwrapOption,
    SomeOption,
    Option,
    Iterator,
    HashMap,
}

struct ConvertibleField {
    source_name: Ident,
    span: Span,
    skip: bool,
    method: FieldConversionMethod,
    target_name: Ident,
}

fn decide_field_method(field: &Field, is_from: bool) -> syn::Result<FieldConversionMethod> {
    let is_option = is_surrounding_type(&field.ty, "Option");
    let is_vec = is_surrounding_type(&field.ty, "Vec");
    let is_hash_map = is_surrounding_type(&field.ty, "HashMap");

    if field_has_attribute(field, "unwrap") {
        match (is_option, is_from) {
            (true, false) => {
                return Ok(FieldConversionMethod::UnwrapOption);
            }
            (true, true) => {
                return Ok(FieldConversionMethod::SomeOption);
            }
            (false, true) => {
                return Ok(FieldConversionMethod::UnwrapOption);
            }
            _ => {
                return Err(Error::new_spanned(
                    &field.ty,
                    "Cannot unwrap non-Option field",
                ));
            }
        }
    }

    if is_option {
        return Ok(FieldConversionMethod::Option);
    }
    if is_vec {
        return Ok(FieldConversionMethod::Iterator);
    }

    if is_hash_map {
        return Ok(FieldConversionMethod::HashMap);
    }

    Ok(FieldConversionMethod::Plain)
}

struct ConversionMeta {
    source_name: Path,
    target_name: Path,
    method: ConversionMethod,
    // Wether we add ..Default::default() to conversions
    default_allowed: bool,
}

enum ConversionMethod {
    Into,
    TryInto,
    From,
    TryFrom,
}

impl ConversionMethod {
    fn is_from(&self) -> bool {
        matches!(self, ConversionMethod::From | ConversionMethod::TryFrom)
    }

    fn is_falliable(&self) -> bool {
        matches!(self, ConversionMethod::TryInto | ConversionMethod::TryFrom)
    }
}

fn ident_to_path(ident: &syn::Ident) -> syn::Path {
    syn::Path {
        leading_colon: None,
        segments: std::iter::once(syn::PathSegment {
            ident: ident.clone(),
            arguments: syn::PathArguments::None,
        })
        .collect(),
    }
}

fn extract_conversions(ast: &DeriveInput) -> Vec<ConversionMeta> {
    ast.attrs
        .iter()
        .filter_map(|attr| {
            let (other_type, method) = extract_single_path_attribute(attr, "into")
                .map(|t| (t, ConversionMethod::Into))
                .or_else(|| {
                    extract_single_path_attribute(attr, "try_into")
                        .map(|t| (t, ConversionMethod::TryInto))
                })
                .or_else(|| {
                    extract_single_path_attribute(attr, "from").map(|t| (t, ConversionMethod::From))
                })
                .or_else(|| {
                    extract_single_path_attribute(attr, "try_from")
                        .map(|t| (t, ConversionMethod::TryFrom))
                })?;

            let (source_name, target_name) = if method.is_from() {
                (other_type, ident_to_path(&ast.ident))
            } else {
                (ident_to_path(&ast.ident), other_type)
            };

            Some(ConversionMeta {
                source_name,
                target_name,
                method,
                default_allowed: has_attribute(attr, "default"),
            })
        })
        .collect()
}

fn field_falliable_conversion(
    ConvertibleField {
        source_name,
        target_name,
        skip,
        method,
        span,
    }: ConvertibleField,
    target_type: &Path,
) -> TokenStream2 {
    if skip {
        return quote! {};
    }

    match method {
        FieldConversionMethod::Plain => quote_spanned! { span =>
            #target_name: source.#source_name.try_into()?,
        },
        FieldConversionMethod::UnwrapOption => {
            quote_spanned! { span =>
                #target_name: source.#source_name.expect(
                    format!("Expected to {} to exist when converting to {}",
                        stringify!(#source_name),
                        stringify!(#target_type))
                )
                    .try_into()?,
            }
        }
        FieldConversionMethod::SomeOption => {
            quote_spanned! { span =>
                #target_name: Some(source.#source_name.try_into()?),
            }
        }
        FieldConversionMethod::Option => {
            quote_spanned! { span =>
                #target_name: source.#source_name.map(TryInto::try_into).transpose()?,
            }
        }
        FieldConversionMethod::Iterator => {
            quote_spanned! { span =>
                #target_name: source.#source_name.into_iter().map(TryInto::try_into).try_collect()?,
            }
        }
        FieldConversionMethod::HashMap => {
            quote_spanned! { span =>
                #target_name: source.#source_name.into_iter().map(|(a, b)| (a.try_into()?, b.try_into()?)).try_collect()?,
            }
        }
    }
}

fn field_infalliable_conversion(
    ConvertibleField {
        source_name,
        target_name,
        skip,
        method,
        span,
    }: ConvertibleField,
    target_type: &Path,
) -> TokenStream2 {
    if skip {
        return quote! {};
    }

    match method {
        FieldConversionMethod::Plain => quote_spanned! { span =>
            #target_name: source.#source_name.into(),
        },
        FieldConversionMethod::UnwrapOption => {
            quote_spanned! { span =>
                #target_name: source.#source_name.expect(format!("Expected to {} to exist when converting to {}", stringify!(#source_name), stringify!(#target_type)).as_str()).into(),
            }
        }
        FieldConversionMethod::SomeOption => {
            quote_spanned! { span =>
                #target_name: Some(source.#source_name.into()),
            }
        }
        FieldConversionMethod::Option => {
            quote_spanned! { span =>
                #target_name: source.#source_name.map(Into::into),
            }
        }
        FieldConversionMethod::Iterator => {
            quote_spanned! { span =>
                #target_name: source.#source_name.into_iter().map(Into::into).collect(),
            }
        }
        FieldConversionMethod::HashMap => {
            quote_spanned! { span =>
                #target_name: source.#source_name.into_iter().map(|(a, b)| (a.into(), b.into)).collect()
            }
        }
    }
}
fn build_convertible_field(field: &Field, meta: &ConversionMeta) -> syn::Result<ConvertibleField> {
    let field_name = field
        .ident
        .clone()
        .expect("Expected field to have an identifier");
    let other_field_name = get_field_value(field, "rename").unwrap_or_else(|| field_name.clone());

    let skip = field_has_attribute(field, "skip");

    if meta.method.is_from() {
        Ok(ConvertibleField {
            source_name: other_field_name,
            span: field.span(),
            skip,
            method: decide_field_method(field, true)?,
            target_name: field_name,
        })
    } else {
        Ok(ConvertibleField {
            source_name: field_name,
            span: field.span(),
            skip,
            method: decide_field_method(field, false)?,
            target_name: other_field_name,
        })
    }
}
fn extract_fields(ast: &DeriveInput) -> syn::Result<Vec<Field>> {
    // Check if the input is a struct
    if let syn::Data::Struct(data_struct) = &ast.data {
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
    } else {
        // Return an error if the input is not a struct
        Err(syn::Error::new_spanned(
            ast.ident.clone(),
            "Expected a struct".to_string(),
        ))
    }
}
pub(super) fn try_convert_derive(ast: &DeriveInput) -> syn::Result<TokenStream2> {
    let conversions = extract_conversions(ast);

    let fields = extract_fields(ast)?;

    let conversions: Vec<TokenStream2> = conversions
        .into_iter()
        .map(|c| implement_conversion(c, &fields))
        .collect::<syn::Result<_>>()?;

    Ok(quote! {
        #(#conversions)*
    })
}

fn implement_conversion(meta: ConversionMeta, fields: &Vec<Field>) -> syn::Result<TokenStream2> {
    let fields: Vec<_> = fields
        .into_iter()
        .map(|field| build_convertible_field(field, &meta))
        .collect::<syn::Result<Vec<_>>>()?
        .into_iter()
        .map(|field| {
            if meta.method.is_falliable() {
                field_falliable_conversion(field, &meta.target_name)
            } else {
                field_infalliable_conversion(field, &meta.target_name)
            }
        })
        .collect();

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
