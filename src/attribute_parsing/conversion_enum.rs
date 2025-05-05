use darling::{FromMeta, FromVariant};
use syn::{DataEnum, Path, spanned::Spanned};

use super::{
    conversion_field::{ConvertibleField, extract_convertible_fields},
    conversion_meta::ConversionMethod,
};

#[derive(FromMeta)]
struct VariantConvAttrs {
    #[darling(default)]
    rename: Option<String>,
    // Add other variant-specific attributes here
    #[darling(default)]
    skip: bool,
}

#[derive(FromVariant)]
#[darling(attributes(convert))]
struct ConvertVariant {
    ident: syn::Ident,
    #[darling(default)]
    rename: Option<String>,
    #[darling(default)]
    skip: bool,

    // Different conversion types for variants
    #[darling(default)]
    from: Option<VariantConvAttrs>,
    #[darling(default)]
    try_from: Option<VariantConvAttrs>,
    #[darling(default)]
    into: Option<VariantConvAttrs>,
    #[darling(default)]
    try_into: Option<VariantConvAttrs>,
}

#[derive(Clone)]
pub(crate) struct ConversionVariant {
    pub(crate) source_name: syn::Ident,
    pub(crate) target_name: syn::Ident,
    pub(crate) named_variant: bool,
    pub(crate) fields: Vec<ConvertibleField>,
}

pub(crate) fn extract_enum_variants(
    data_enum: &DataEnum,
    conversion_type: ConversionMethod,
    other_type: &Path,
) -> syn::Result<Vec<ConversionVariant>> {
    let is_from = conversion_type.is_from();
    data_enum
        .variants
        .iter()
        .map(|variant| {
            // Parse variant attributes using darling
            let convert_variant = match ConvertVariant::from_variant(variant) {
                Ok(cv) => cv,
                Err(e) => {
                    return Err(syn::Error::new(
                        variant.span(),
                        format!("Failed to parse variant attributes: {}", e),
                    ));
                }
            };

            let named_variant = matches!(variant.fields, syn::Fields::Named(_));

            // Get the specific conversion attributes based on conversion type
            let variant_conv_attrs = match conversion_type {
                ConversionMethod::From => convert_variant.from,
                ConversionMethod::TryFrom => convert_variant.try_from,
                ConversionMethod::Into => convert_variant.into,
                ConversionMethod::TryInto => convert_variant.try_into,
            };

            // Skip if marked with skip
            if convert_variant.skip || variant_conv_attrs.as_ref().is_some_and(|attr| attr.skip) {
                return Ok(None); // Return None to filter out later
            }

            // Determine the target variant name with priority:
            // 1. Conversion-specific rename
            // 2. Top-level rename
            // 3. Original variant name
            let other_variant_name = variant_conv_attrs
                .as_ref()
                .and_then(|attrs| attrs.rename.as_ref())
                .or(convert_variant.rename.as_ref())
                .map(|rename| syn::Ident::new(rename, variant.span()))
                .unwrap_or_else(|| convert_variant.ident.clone());

            let (source_name, target_name) = if is_from {
                (other_variant_name, convert_variant.ident.clone())
            } else {
                (convert_variant.ident.clone(), other_variant_name)
            };

            Ok(Some(ConversionVariant {
                source_name,
                target_name,
                named_variant,
                fields: extract_convertible_fields(&variant.fields, conversion_type, other_type)?,
            }))
        })
        .filter_map(|result| result.transpose())
        .collect()
}
