use derive_into::try_convert_derive;
use syn::{DeriveInput, parse_macro_input};

mod attribute_parsing;
mod derive_into;
mod enum_convert;
mod struct_convert;
mod util;

/** # derive-into

 For more information, visit the [github repository](https://github.com/sharonex/derive-into/tree/darling-migration).

 A derive macro for creating conversions between structs and enums with similar structures.

 This crate provides the `#[derive(Convert)]` macro that automates implementations of
 conversion traits (`From`, `Into`, `TryFrom`, `TryInto`) between types.

 ## Installation

 ```toml
 [dependencies]
 derive-into = "0.1.0"
 ```

 ## Basic Usage

 ```rust
 use derive_into::Convert;

 #[derive(Convert)]
 #[convert(into(path = "Destination"))]
 struct Source {
     id: u32,
     #[convert(rename = "full_name")]
     name: String,
 }

 struct Destination {
     id: u32,
     full_name: String,
 }

 // Usage: let destination: Destination = source.into();
 ```

 ## Attribute Reference

 ### Struct/Enum Level Attributes

 | Attribute | Description |
 |-----------|-------------|
 | `#[convert(into(path = "Type"))]` | Implements `From<Self> for Type` |
 | `#[convert(from(path = "Type"))]` | Implements `From<Type> for Self` |
 | `#[convert(try_into(path = "Type"))]` | Implements `TryFrom<Self> for Type` |
 | `#[convert(try_from(path = "Type"))]` | Implements `TryFrom<Type> for Self` |
 | `#[convert(into(path = "Type", default))]` | Uses `Default::default()` for unmapped fields |

 Multiple conversion attributes can be specified for a single type:

 ```rust
 #[derive(Convert)]
 #[convert(into(path = "ApiModel"))]
 #[convert(try_from(path = "DbModel"))]
 struct DomainModel {
     // fields
 }
 ```

 ### Field Level Attributes

 Field attributes can be applied at three different scopes:

 1. **Global scope** - applies to all conversions
 ```rust
 #[convert(rename = "new_name")]
 ```

 2. **Conversion type scope** - applies to a specific conversion type
 ```rust
 #[convert(try_from(skip))]
 ```

 3. **Specific conversion scope** - applies to a specific conversion path
 ```rust
 #[convert(try_from(path = "ApiModel", skip))]
 ```

 | Attribute | Description |
 |-----------|-------------|
 | `#[convert(rename = "new_name")]` | Maps field to different name in target |
 | `#[convert(skip)]` | Excludes field from conversion |
 | `#[convert(default)]` | Uses `Default::default()` for this field |
 | `#[convert(unwrap)]` | Unwraps `Option` (`try_from` fails if `None`) |
 | `#[convert(with_func = "func_name")]` | Uses custom conversion function |

 ### Custom Conversion Functions

 Functions specified with `with_func` must accept a reference to the source type:

 ```rust
 #[derive(Convert)]
 #[convert(try_from(path = "ApiModel"))]
 struct Product {
     #[convert(try_from(with_func = "validate_field"))]
     validated: ValidatedType,
 }

 fn validate_field(source: &ApiModel) -> Result<ValidatedType, String> {
     // Custom validation/conversion logic
 }
 ```

 ## Type Conversion Behavior

 * **Direct mapping**: Identical types are copied directly
 * **Automatic conversion**: Uses `From`/`Into` for different types
 * **Container types**: Handles `Option<T>`, `Vec<T>`, and `HashMap<K,V>`
 * **Nested conversions**: Converts nested structs/enums automatically

 ## Container Type Examples

 ### Option and Vec

 ```rust
 #[derive(Convert)]
 #[convert(into(path = "Target"))]
 struct Source {
     // Inner type u8 -> Number conversion happens automatically
     optional: Option<u8>,
     vector: Vec<u8>,
 }

 struct Target {
     optional: Option<Number>, // Number implements From<u8>
     vector: Vec<Number>,
 }
 ```

 ### HashMap

 ```rust
 #[derive(Convert)]
 #[convert(into(path = "Target"))]
 struct Source {
     // Both keys and values convert if they implement From/Into
     map: HashMap<String, u32>,
 }

 struct Target {
     map: HashMap<CustomString, CustomInt>,
 }
 ```

 ## Enum Conversion

 ```rust
 #[derive(Convert)]
 #[convert(into(path = "TargetEnum"))]
 enum SourceEnum {
     Variant1(u32),
     #[convert(rename = "RenamedVariant")]
     Variant2 {
         value: String,
         #[convert(rename = "renamed_field")]
         field: u8,
     },
     Unit,
 }
 ```

 Derive macro for generating conversion implementations between similar types.

 The `Convert` derive macro generates implementations of standard conversion traits
 (`From`, `Into`, `TryFrom`, `TryInto`) between structs and enums with similar structures.

 # Examples

 Basic struct conversion with field renaming:

 ```rust
 use derive_into::Convert;

 #[derive(Convert)]
 #[convert(into(path = "Destination"))]
 struct Source {
     id: u32,
     #[convert(rename = "full_name")]
     name: String,
 }
 ```

 Enum conversion with variant renaming:

 ```rust
 #[derive(Convert)]
 #[convert(into(path = "TargetEnum"))]
 enum SourceEnum {
     Variant1(u32),
     #[convert(rename = "RenamedVariant")]
     Variant2 { value: String },
 }
 ```
*/
#[proc_macro_derive(Convert, attributes(convert))]
pub fn derive_into(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
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
        t.pass("tests/cases/test_complex_conversions.rs");
        t.pass("tests/cases/test_enum_conversions.rs");
        t.pass("tests/cases/test_struct_conversions.rs");
    }
}
