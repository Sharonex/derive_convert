use derive_convert::try_convert_derive;
use syn::{DeriveInput, parse_macro_input};

mod derive_convert;
mod struct_convert;
mod util;

#[proc_macro_derive(Convert, attributes(convert))]
pub fn derive_convert(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    try_convert_derive(&input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_derive_macro() {
        let t = trybuild::TestCases::new();
        // Use the correct relative path from the project root
        t.pass("tests/cases/basic.rs");
    }
}
