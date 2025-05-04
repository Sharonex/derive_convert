use darling::{FromField, FromMeta};
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{ToTokens, format_ident, quote};
use syn::{Field, Ident, Path, spanned::Spanned};

use crate::util::is_surrounding_type;

use super::conversion_meta::ConversionMethod;

// Field level attributes using darling
#[derive(FromMeta, Debug)]
struct ConvertFieldAttr {
    path: Option<Path>,

    #[darling(default)]
    skip: bool,

    #[darling(default)]
    unwrap: bool,

    #[darling(default)]
    default: bool,

    // Add any other field attributes you need
    #[darling(default)]
    rename: Option<String>,

    #[darling(default)]
    with_func: Option<syn::Path>,
}

#[derive(FromField, Debug)]
#[darling(attributes(convert))]
struct ConvertField {
    ident: Option<Ident>,

    #[darling(default)]
    skip: bool,

    #[darling(default)]
    rename: Option<String>,

    #[darling(default)]
    default: bool,

    #[darling(default)]
    unwrap: bool,

    #[darling(default)]
    with_func: Option<syn::Path>,

    // Different conversion types
    #[darling(default)]
    from: Option<ConvertFieldAttr>,

    #[darling(default)]
    try_from: Option<ConvertFieldAttr>,

    #[darling(default)]
    into: Option<ConvertFieldAttr>,

    #[darling(default)]
    try_into: Option<ConvertFieldAttr>,
}

#[derive(Clone)]
pub(crate) enum FieldConversionMethod {
    Plain,
    UnwrapOption,
    SomeOption,
    Option,
    Iterator,
    HashMap,
}

#[derive(Clone)]
pub(crate) enum FieldIdentifier {
    Named(Ident),
    Unnamed(usize),
}

#[derive(Clone)]
pub(crate) struct ConvertibleField {
    pub(crate) source_name: FieldIdentifier,
    pub(crate) span: Span,
    pub(crate) skip: bool,
    pub(crate) default: bool,
    pub(crate) method: FieldConversionMethod,
    pub(crate) target_name: FieldIdentifier,
    pub(crate) conversion_func: Option<syn::Path>,
}

pub(crate) fn extract_convertible_fields(
    fields: &syn::Fields,
    conversion_type: ConversionMethod,
    other_type: &Path,
) -> syn::Result<Vec<ConvertibleField>> {
    let mut result = Vec::new();

    // Determine which nested field we should check based on conversion type
    let is_from = matches!(
        conversion_type,
        ConversionMethod::From | ConversionMethod::TryFrom
    );

    for (i, field) in fields.iter().enumerate() {
        // Use darling to parse field attributes
        let convert_field = match ConvertField::from_field(field) {
            Ok(cf) => cf,
            Err(e) => {
                return Err(syn::Error::new(
                    field.span(),
                    format!("Failed to parse field attributes: {}", e),
                ));
            }
        };

        // Determine source field identifier
        let source_name = match &convert_field.ident {
            Some(ident) => FieldIdentifier::Named(ident.clone()),
            None => FieldIdentifier::Unnamed(i),
        };

        // Get the specific conversion attributes based on conversion type
        let mut field_conv_attrs = match conversion_type {
            ConversionMethod::From => convert_field.from,
            ConversionMethod::TryFrom => convert_field.try_from,
            ConversionMethod::Into => convert_field.into,
            ConversionMethod::TryInto => convert_field.try_into,
        };

        if field_conv_attrs
            .as_ref()
            .is_some_and(|attrs| attrs.path.as_ref().is_some_and(|path| path != other_type))
        {
            field_conv_attrs = None;
        }

        let unwrap = field_conv_attrs
            .as_ref()
            .map_or(convert_field.unwrap, |attrs| attrs.unwrap);

        let default = field_conv_attrs
            .as_ref()
            .map_or(convert_field.default, |attrs| attrs.default);

        // Skip applies if either top-level or field-specific skip is true
        let skip =
            convert_field.skip || field_conv_attrs.as_ref().map_or(false, |attrs| attrs.skip);

        // Skip if marked with skip
        if skip {
            continue;
        }

        // Determine target field identifier with priority:
        // 1. Field-specific rename
        // 2. Top-level rename
        // 3. Original field name
        let target_name = field_conv_attrs
            .as_ref()
            .and_then(|attrs| attrs.rename.as_ref())
            .or(convert_field.rename.as_ref())
            .map(|rename| FieldIdentifier::Named(Ident::new(rename, field.span())))
            .unwrap_or_else(|| source_name.clone());

        // Determine field conversion method
        let method = decide_field_method(field, is_from, unwrap)?;

        let conversion_func = field_conv_attrs
            .as_ref()
            .and_then(|attrs| attrs.with_func.as_ref())
            .or(convert_field.with_func.as_ref())
            .cloned();

        let (source_name, target_name) = if is_from {
            (target_name.clone(), source_name.clone())
        } else {
            (source_name.clone(), target_name.clone())
        };

        result.push(ConvertibleField {
            source_name,
            span: field.span(),
            skip: false, // We've already filtered out skipped fields
            method,
            target_name,
            default,
            conversion_func,
        });
    }

    // sort so that fields with conversion functions are first
    result.sort_by(|a, b| {
        if a.conversion_func.is_some() && b.conversion_func.is_none() {
            std::cmp::Ordering::Less
        } else if a.conversion_func.is_none() && b.conversion_func.is_some() {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Equal
        }
    });

    Ok(result)
}

pub(crate) fn decide_field_method(
    field: &Field,
    is_from: bool,
    unwrap: bool,
) -> syn::Result<FieldConversionMethod> {
    let is_option = is_surrounding_type(&field.ty, "Option");
    let is_vec = is_surrounding_type(&field.ty, "Vec");
    let is_hash_map = is_surrounding_type(&field.ty, "HashMap");

    if unwrap {
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
                return Err(syn::Error::new_spanned(
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
    pub(crate) fn as_named(&self) -> TokenStream2 {
        match self {
            FieldIdentifier::Named(ident) => quote! { #ident },
            FieldIdentifier::Unnamed(index) => {
                let field_name = format_ident!("field{}", index);
                quote! { #field_name }
            }
        }
    }
}
