use syn::{Attribute, Field, Ident, Meta, NestedMeta, Type, parse_str, spanned::Spanned};

pub(super) fn has_attribute(attr: &Attribute, attribute_name: &str) -> bool {
    if !attr.path.is_ident("convert") {
        return false;
    }

    if let Ok(Meta::List(meta_list)) = attr.parse_meta() {
        for nested_meta in meta_list.nested {
            if let NestedMeta::Meta(Meta::Path(path)) = nested_meta {
                if path.is_ident(attribute_name) {
                    return true;
                }
            }
        }
    }
    false
}

pub(super) fn field_has_attribute(field: &Field, attribute_name: &str) -> bool {
    field
        .attrs
        .iter()
        .any(|attr| has_attribute(attr, attribute_name))
}

pub(super) fn get_attribute_value(attr: &Attribute, attribute_name: &str) -> Option<String> {
    if !attr.path.is_ident("convert") {
        return None;
    }

    // Check for nested attribute: #[outer(attribute_name = "value")]
    attr.parse_meta().ok().and_then(|meta| {
        if let syn::Meta::List(list) = meta {
            list.nested.iter().find_map(|nested| {
                if let syn::NestedMeta::Meta(syn::Meta::NameValue(name_value)) = nested {
                    if name_value.path.is_ident(attribute_name) {
                        if let syn::Lit::Str(lit_str) = &name_value.lit {
                            return Some(lit_str.value());
                        }
                    }
                }
                None
            })
        } else {
            None
        }
    })
}

pub(super) fn get_field_value(field: &Field, attribute_name: &str) -> Option<syn::Ident> {
    field
        .attrs
        .iter()
        .find_map(|attr| get_attribute_value(attr, attribute_name))
        .map(|attr_val| syn::Ident::new(attr_val.as_str(), field.span()))
}

pub(super) fn extract_single_path_attribute(
    attr: &syn::Attribute,
    value_name: &str,
) -> Option<syn::Path> {
    get_attribute_value(attr, value_name)
        .and_then(|attr_val| syn::parse_str::<syn::Path>(&attr_val).ok())
}

pub(super) fn is_surrounding_type(ty: &syn::Type, surrounding_type: &'static str) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if type_path.path.segments.len() == 1 {
            let segment = &type_path.path.segments[0];
            if segment.ident == surrounding_type {
                return true;
            }
        }
    }
    false
}
