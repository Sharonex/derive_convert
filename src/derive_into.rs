use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned};
use syn::{DeriveInput, Path};

use crate::{
    attribute_parsing::{
        conversion_field::{ConvertibleField, FieldConversionMethod},
        conversion_meta::{ConversionMeta, extract_conversions},
    },
    enum_convert::implement_all_enum_conversions,
    struct_convert::implement_all_struct_conversions,
};

pub(super) fn field_falliable_conversion(
    ConvertibleField {
        source_name,
        target_name,
        skip,
        method,
        span,
        default,
        conversion_func,
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

    if default {
        return quote_spanned! { span =>
            #named_start Default::default(),
        };
    }

    if let Some(func) = conversion_func {
        return quote_spanned! { span =>
            #named_start #func(&source),
        };
    }

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
                #named_start {
                    let mut result = ::std::collections::HashMap::new();
                    for (k, v) in #source_name {
                        let key = k.try_into().map_err(|e|
                            format!("Failed to convert key in HashMap {}: {:?}",
                                stringify!(#source_name), e))?;
                        let value = v.try_into().map_err(|e|
                            format!("Failed to convert value in HashMap {}: {:?}",
                                stringify!(#source_name), e))?;
                        result.insert(key, value);
                    }
                    result
                },
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
        default,
        conversion_func,
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

    if default {
        return quote_spanned! { span =>
            #named_start Default::default(),
        };
    }

    if let Some(func) = conversion_func {
        return quote_spanned! { span =>
            #named_start #func(&source),
        };
    }

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
