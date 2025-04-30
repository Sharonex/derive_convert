use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{ToTokens, format_ident, quote, quote_spanned};
use syn::{DeriveInput, Error, Field, Ident, Path, spanned::Spanned};

use crate::{
    enum_convert::implement_all_enum_conversions,
    struct_convert::implement_all_struct_conversions,
    util::{
        extract_single_path_attribute, field_has_attribute, get_field_value, has_attribute,
        is_surrounding_type,
    },
};

#[derive(Clone)]
enum FieldConversionMethod {
    Plain,
    UnwrapOption,
    SomeOption,
    Option,
    Iterator,
    HashMap,
}

#[derive(Clone)]
pub(super) enum FieldIdentifier {
    Named(Ident),
    Unnamed(usize),
}

impl ToTokens for FieldIdentifier {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            FieldIdentifier::Named(ident) => {
                tokens.extend(quote! { #ident });
            }
            FieldIdentifier::Unnamed(index) => {
                let index = syn::Index::from(*index);
                tokens.extend(quote! { #index });
            }
        }
    }
}

impl FieldIdentifier {
    pub(super) fn as_named(&self) -> TokenStream2 {
        match self {
            FieldIdentifier::Named(ident) => quote! { #ident },
            FieldIdentifier::Unnamed(index) => {
                let field_name = format_ident!("field{}", index);
                quote! { #field_name }
            }
        }
    }
}

#[derive(Clone)]
pub(super) struct ConvertibleField {
    pub(super) source_name: FieldIdentifier,
    span: Span,
    skip: bool,
    method: FieldConversionMethod,
    pub(super) target_name: FieldIdentifier,
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

#[derive(Clone)]
pub(super) struct ConversionMeta {
    pub(super) source_name: Path,
    pub(super) target_name: Path,
    pub(super) method: ConversionMethod,
    // Wether we add ..Default::default() to conversions
    pub(super) default_allowed: bool,
}

#[derive(Clone)]
pub(super) enum ConversionMethod {
    Into,
    TryInto,
    From,
    TryFrom,
}

impl ConversionMethod {
    pub(super) fn is_from(&self) -> bool {
        matches!(self, ConversionMethod::From | ConversionMethod::TryFrom)
    }

    pub(super) fn is_falliable(&self) -> bool {
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

pub(super) fn field_falliable_conversion(
    ConvertibleField {
        source_name,
        target_name,
        skip,
        method,
        span,
    }: ConvertibleField,
    target_type: &Path,
    named: bool,
    source_prefix: bool,
) -> TokenStream2 {
    if skip {
        return quote! {};
    }

    let named_start = if named {
        quote! { #target_name: }
    } else {
        quote! {}
    };

    let source_name = if source_prefix {
        quote!(source.#source_name)
    } else {
        let source_name = source_name.as_named();
        quote!(#source_name)
    };

    match method {
        FieldConversionMethod::Plain => quote_spanned! { span =>
            #named_start #source_name.try_into().map_err(|e|
                    format!("Failed trying to convert {} to {}: {:?}",
                        stringify!(#source_name),
                        stringify!(#target_type),
                        e,
                    )
                )?,
        },
        FieldConversionMethod::UnwrapOption => {
            quote_spanned! { span =>
                #named_start #source_name.ok_or_else(Err(
                    format!("Failed trying to convert {} to {}: {:?}",
                        stringify!(#source_name),
                        stringify!(#target_type),
                        e,
                    ))
                )
                .try_into()
                .map_err(|e|
                    format!("Failed trying to convert {} to {}: {:?}",
                        stringify!(#source_name),
                        stringify!(#target_type),
                        e,
                    )
                )?,
            }
        }
        FieldConversionMethod::SomeOption => {
            quote_spanned! { span =>
                #named_start Some(#source_name.try_into().map_err(|e|
                    format!("Failed trying to convert {} to {}: {:?}",
                        stringify!(#source_name),
                        stringify!(#target_type),
                        e,
                    )
                )?),
            }
        }
        FieldConversionMethod::Option => {
            quote_spanned! { span =>
                #named_start #source_name.map(TryInto::try_into).transpose().map_err(|e|
                    format!("Failed trying to convert {} to {}: {:?}",
                        stringify!(#source_name),
                        stringify!(#target_type),
                        e,
                    )
                )?,
            }
        }
        FieldConversionMethod::Iterator => {
            quote_spanned! { span =>
                #named_start #source_name.into_iter().map(TryInto::try_into).try_collect().map_err(|e|
                    format!("Failed trying to convert {} to {}: {:?}",
                        stringify!(#source_name),
                        stringify!(#target_type),
                        e,
                    )
                )?,
            }
        }
        FieldConversionMethod::HashMap => {
            quote_spanned! { span =>
                #named_start #source_name.into_iter().map(|(a, b)| (a.try_into()?, b.try_into()?)).try_collect().map_err(|e|
                    format!("Failed trying to convert {} to {}: {:?}",
                        stringify!(#source_name),
                        stringify!(#target_type),
                        e,
                    )
                )?,
            }
        }
    }
}

pub(super) fn field_infalliable_conversion(
    ConvertibleField {
        source_name,
        target_name,
        skip,
        method,
        span,
    }: ConvertibleField,
    target_type: &Path,
    named: bool,
    source_prefix: bool,
) -> TokenStream2 {
    if skip {
        return quote! {};
    }
    let named_start = if named {
        quote! { #target_name: }
    } else {
        quote! {}
    };

    let source_name = if source_prefix {
        quote!(source.#source_name)
    } else {
        let source_name = source_name.as_named();
        quote!(#source_name)
    };

    match method {
        FieldConversionMethod::Plain => quote_spanned! { span =>
            #named_start #source_name.into(),
        },
        FieldConversionMethod::UnwrapOption => {
            quote_spanned! { span =>
                #named_start #source_name.expect(
                    format!("Expected to {} to exist when converting to {}",
                        stringify!(#source_name),
                        stringify!(#target_type),
                    ).as_str()
                ).into(),
            }
        }
        FieldConversionMethod::SomeOption => {
            quote_spanned! { span =>
                #named_start Some(#source_name.into()),
            }
        }
        FieldConversionMethod::Option => {
            quote_spanned! { span =>
                #named_start #source_name.map(Into::into),
            }
        }
        FieldConversionMethod::Iterator => {
            quote_spanned! { span =>
                #named_start #source_name.into_iter().map(Into::into).collect(),
            }
        }
        FieldConversionMethod::HashMap => {
            quote_spanned! { span =>
                #named_start #source_name.into_iter().map(|(a, b)| (a.into(), b.into())).collect(),
            }
        }
    }
}
pub(super) fn build_convertible_field(
    field: &Field,
    meta: &ConversionMeta,
    index: usize,
) -> syn::Result<ConvertibleField> {
    let field_name = field
        .ident
        .clone()
        .map(FieldIdentifier::Named)
        .unwrap_or(FieldIdentifier::Unnamed(index));

    let other_field_name = get_field_value(field, "rename")
        .map(FieldIdentifier::Named)
        .unwrap_or_else(|| field_name.clone());

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

pub(super) fn build_field_conversions(
    meta: &ConversionMeta,
    named: bool,
    source_prefix: bool,
    fields: &Vec<ConvertibleField>,
) -> syn::Result<Vec<TokenStream2>> {
    Ok(fields
        .into_iter()
        .map(|field| {
            if meta.method.is_falliable() {
                field_falliable_conversion(field.clone(), &meta.target_name, named, source_prefix)
            } else {
                field_infalliable_conversion(field.clone(), &meta.target_name, named, source_prefix)
            }
        })
        .collect())
}

pub(super) fn try_convert_derive(ast: &DeriveInput) -> syn::Result<TokenStream2> {
    let conversions = extract_conversions(ast);

    match &ast.data {
        syn::Data::Struct(data_struct) => {
            implement_all_struct_conversions(data_struct, conversions)
        }
        syn::Data::Enum(data_enum) => implement_all_enum_conversions(data_enum, conversions),
        syn::Data::Union(_) => Err(syn::Error::new_spanned(
            ast.ident.clone(),
            "Unions are not supported".to_string(),
        ))?,
    }
}
