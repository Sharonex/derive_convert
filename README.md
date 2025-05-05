# derive-into

A Rust derive macro for easily creating conversions between structs and enums.

## Features

- Automate conversions between similar data structures
- Support for struct-to-struct, tuple struct, and enum conversions
- Field renaming capabilities
- Automatic handling of wrapped types with `From`/`Into` implementations
- Special handling for `Option` and `Vec` types
- Support for both infallible (`From`/`Into`) and fallible (`TryFrom`) conversions
- Fine-grained control with field-level attributes
- Support for nested type conversions
- HashMap conversion with key and value type conversions
- Custom conversion functions with the `with_func` attribute

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
derive-into = "0.1.0"
```

## Quick Start

```rust
use derive_into::Convert;

// Source struct with conversion attributes
#[derive(Convert)]
#[convert(into(path = "Destination"))] // Generate Into<Destination> implementation
struct Source {
    id: u32,
    #[convert(rename = "full_name")] // Field will be mapped to "full_name" in target
    name: String,
}

// Destination struct
struct Destination {
    id: u32,
    full_name: String,
}

// Usage
let source = Source {
    id: 1,
    name: "Example".to_string(),
};
let destination: Destination = source.into();
```

## Struct-Level Attributes

Struct-level attributes can be applied at the struct or enum level to control conversion behavior:

| Attribute | Description |
|-----------|-------------|
| `#[convert(into(path = "Type"))]` | Generate an `From<Self> for Type` implementation |
| `#[convert(try_into(path = "Type"))]` | Generate an `TryFrom<Self> for Type` implementation |
| `#[convert(try_from(path = "Type"))]` | Generate a `TryFrom<Type> for Self` implementation |
| `#[convert(from(path = "Type"))]` | Generate a `From<Type> for Self` implementation |
| `#[convert(into(path = "Type", default))]` | Enable default values for fields not explicitly mapped in the target type |

Multiple conversion types can be specified for a single struct:

```rust
#[derive(Convert)]
#[convert(into(path = "TargetType"))]
#[convert(try_from(path = "TargetType"))]
struct MyStruct {
    // fields
}
```
| `#[convert(try_from(path = "Type"))]` | Specify a path for try_from conversion |

## Field-Level Attributes

Field-level attributes can be applied at three different scopes:

1. **Global scope** - applies to all conversion types:
   ```rust
   #[convert(rename = "new_name", skip)]
   ```

2. **Conversion type scope** - applies only to a specific conversion type (into, from, try_from):
   ```rust
   #[convert(try_from(skip, default))]
   ```

3. **Specific conversion scope** - applies only to a singular conversion target:
   ```rust
   #[convert(try_from(path = "ApiProduct", skip, default))]
   ```

Common field-level attributes:

| Attribute | Description |
|-----------|-------------|
| `#[convert(rename = "new_name")]` | Map this field to a differently named field in the target type |
| `#[convert(unwrap_or_default)]` | Automatically calls unwrap_or_default on `Option` value before converting it |
| `#[convert(unwrap)]` | Automatically unwrap an `Option` value (fails in `try_from` if `None`) |
| `#[convert(skip)]` | Skip this field during conversion (target must provide a default) |
| `#[convert(default)]` | Use default value for this field during conversion |
| `#[convert(with_func = func_name)]` | Use custom function for conversion. The function needs to take a reference to the parent struct |

## Enum Conversion

The macro supports enum-to-enum conversion with similar attribute control:

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

enum TargetEnum {
    Variant1(u32),
    RenamedVariant {
        value: String,
        renamed_field: u32,
    },
    Unit,
}
```

## Type Conversions

The macro intelligently handles various type scenarios:

1. **Direct Mapping**: Fields with identical types are directly copied
2. **Automatic Conversion**: Fields with types that implement `From`/`Into` are automatically converted
3. **Container Types**: Special handling for `Option<T>` and `Vec<T>` with inner type conversion
4. **HashMap Support**: Automatic conversion of HashMap keys and values
5. **Tuple Structs**: Support for conversions between tuple structs
6. **Nested Type Conversions**: Automatically handles nested struct and enum conversions

## Examples

### Basic Struct Conversion

```rust
use derive_into::Convert;

#[derive(Convert)]
#[convert(into(path = "Target"))]
struct Source {
    id: u32,
    name: String,
}

struct Target {
    id: u32,
    name: String,
}

// Usage
let source = Source { id: 1, name: "Example".to_string() };
let target: Target = source.into();
```

### Handling Option and Vec Types

The macro automatically handles conversion of inner types for `Option` and `Vec`:

```rust
use derive_into::Convert;

#[derive(Debug, PartialEq, Default)]
struct Number(u8);

impl From<u8> for Number {
    fn from(n: u8) -> Number {
        Number(n)
    }
}

#[derive(Convert)]
#[convert(into = "Target")]
struct Source {
    // Option's inner type will be converted
    opt_value: Option<u8>,
    // Vec's inner type will be converted
    vec_values: Vec<u8>,
}

struct Target {
    opt_value: Option<Number>,
    vec_values: Vec<Number>,
}
```
### Using UnwrapOrDefault for Options

```rust
use derive_into::Convert;

#[derive(Convert)]
#[convert(try_from(path = "Source"))]
struct Target {
    #[convert(unwrap_or_default)]
    value: u32,
}

struct Source {
    value: Option<u32>,
}

// This will succeed
let source = Source { value: None };
let target: Result<Target, _> = Target::try_from(source);
assert!(target.is_ok());

// This will fail because value is None
let source = Source { value: None };
let target: Result<Target, _> = Target::try_from(source);
assert!(target.is_err());
```

### Using Unwrap for Options

```rust
use derive_into::Convert;

#[derive(Convert)]
#[convert(try_from(path = "Source"))]
struct Target {
    #[convert(unwrap)]
    value: u32,
}

struct Source {
    value: Option<u32>,
}

// This will succeed
let source = Source { value: Some(42) };
let target: Result<Target, _> = Target::try_from(source);
assert!(target.is_ok());

// This will fail because value is None
let source = Source { value: None };
let target: Result<Target, _> = Target::try_from(source);
assert!(target.is_err());
```

### Using Default Values

```rust
use derive_into::Convert;

#[derive(Convert)]
#[convert(into(path = "Target", default))]
struct Source {
    id: u32,
    // No 'extra' field - will use default
}

#[derive(Default)]
struct Target {
    id: u32,
    extra: String, // Will use Default::default()
}
```
<details>

<summary>More examples</summary>

### Tuple Struct Conversion

```rust
use derive_into::Convert;

#[derive(Convert)]
#[convert(into(path = "Target"))]
struct Source(Option<u8>, u8);

struct Target(Option<Number>, Number);
```

### Complex Nested Conversions with Scoped Attributes

```rust
use derive_into::Convert;
use std::collections::HashMap;

#[derive(Convert)]
#[convert(into(path = "ApiProduct", default))]
#[convert(try_from(path = "ApiProduct"))]
struct Product {
    id: String,
    name: NonEmptyString,

    // Vector of complex types with renamed field
    #[convert(rename = "variants")]
    product_variants: Vec<ProductVariant>,

    // HashMap with key/value type conversion
    #[convert(rename = "price_by_region")]
    regional_prices: HashMap<String, f64>,

    // Nested struct with its own conversion
    manufacturer: Manufacturer,

    // Field that will be skipped only during into conversion
    #[convert(into(skip))]
    internal_tracking_code: String,

    // Field that uses default value only during try_from conversion
    #[convert(try_from(default))]
    sku: String,

    // Field that uses custom conversion function
    #[convert(try_from(with_func = conversion_func))]
    product_err: ProductError,
}

// Custom conversion function
fn conversion_func(val: &ApiProduct) -> ProductError {
    ProductError {
        message: if val.name.is_empty() {
            "Name cannot be empty".to_string()
        } else {
            "Valid name".to_string()
        },
    }
}
```

### Enum Conversion with Nested Types

```rust
use derive_into::Convert;

#[derive(Convert)]
#[convert(into(path = "TargetEvent"))]
#[convert(try_from(path = "TargetEvent"))]
enum SourceEvent {
    // Tuple variant with type conversion
    Click(u64),

    // Variant with renamed variant name
    #[convert(rename = "LogoutEvent")]
    Logout {
        username: String,
        timestamp: u64,
    },

    // Variant with nested enum conversion
    UserAction {
        user_id: u64,
        action_type: SourceActionType,
    },
}

enum TargetEvent {
    // Type conversion in tuple variant
    Click(CustomId),

    // Renamed variant
    LogoutEvent {
        username: String,
        timestamp: CustomId,
    },

    // Nested enum conversion
    UserAction {
        user_id: CustomId,
        action_type: TargetActionType,
    },
}
```

### Custom Conversion Functions

```rust
use derive_into::Convert;

#[derive(Convert)]
#[convert(try_from(path = "ApiProduct"))]
struct Product {
    // Field that requires custom conversion
    #[convert(try_from(with_func = validation_function))]
    validated_field: SomeType,
}

// Custom conversion function
fn validation_function(source: &ApiProduct) -> SomeType {
    // Custom conversion/validation logic
    SomeType::new(source.some_field.clone())
}
```
</details>

## License

This project is licensed under the MIT License - see the LICENSE file for details.
