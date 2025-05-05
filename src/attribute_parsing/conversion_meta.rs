use darling::{FromDeriveInput, FromMeta};
use syn::{DeriveInput, Path};

#[derive(Clone, Debug)]
pub(crate) struct ConversionMeta {
    pub(crate) source_name: Path,
    pub(crate) target_name: Path,
    pub(crate) method: ConversionMethod,
    // Wether we add ..Default::default() to conversions
    pub(crate) default_allowed: bool,
}

impl ConversionMeta {
    pub(crate) fn other_type(&self) -> Path {
        if self.method.is_from() {
            self.source_name.clone()
        } else {
            self.target_name.clone()
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum ConversionMethod {
    Into,
    TryInto,
    From,
    TryFrom,
}

impl ConversionMethod {
    pub(crate) fn is_from(&self) -> bool {
        matches!(self, ConversionMethod::From | ConversionMethod::TryFrom)
    }

    pub(crate) fn is_falliable(&self) -> bool {
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

#[derive(FromMeta, Debug)]
struct ConvAttrs {
    path: Path,
    #[darling(default)]
    default: bool,
}

#[derive(FromDeriveInput)]
#[darling(attributes(convert))]
struct Conversions {
    ident: syn::Ident,
    #[darling(default, multiple)]
    into: Vec<ConvAttrs>,

    #[darling(default, multiple)]
    try_into: Vec<ConvAttrs>,

    #[darling(default, multiple)]
    from: Vec<ConvAttrs>,

    #[darling(default, multiple)]
    try_from: Vec<ConvAttrs>,
}

pub(crate) fn extract_conversions(ast: &DeriveInput) -> Vec<ConversionMeta> {
    let conversions_data = match Conversions::from_derive_input(ast) {
        Ok(v) => v,
        Err(e) => {
            // You'd typically emit this as a compile error
            panic!("Error parsing conversion attributes: {}", e);
        }
    };

    let mut result = Vec::new();

    // Process "into" attribute
    for attr in conversions_data.into {
        result.push(ConversionMeta {
            source_name: ident_to_path(&conversions_data.ident),
            target_name: attr.path,
            method: ConversionMethod::Into,
            default_allowed: attr.default,
        });
    }

    // Process "try_into" attribute
    for attr in conversions_data.try_into {
        result.push(ConversionMeta {
            source_name: ident_to_path(&conversions_data.ident),
            target_name: attr.path,
            method: ConversionMethod::TryInto,
            default_allowed: attr.default,
        });
    }

    // Process "from" attribute
    for attr in conversions_data.from {
        result.push(ConversionMeta {
            source_name: attr.path,
            target_name: ident_to_path(&conversions_data.ident),
            method: ConversionMethod::From,
            default_allowed: attr.default,
        });
    }

    // Process "try_from" attribute
    for attr in conversions_data.try_from {
        result.push(ConversionMeta {
            source_name: attr.path,
            target_name: ident_to_path(&conversions_data.ident),
            method: ConversionMethod::TryFrom,
            default_allowed: attr.default,
        });
    }

    result
}
