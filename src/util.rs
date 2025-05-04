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
