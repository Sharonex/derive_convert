# derive-convert

A Rust derive macro for easily creating conversions between structs and enums.

## Features

- Automate conversions between similar data structures
- Support for struct-to-struct, tuple struct, and enum conversions
- Field renaming capabilities
- Automatic handling of wrapped types with `From`/`Into` implementations
- Special handling for `Option` and `Vec` types
- Support for both infallible (`From`) and fallible (`TryFrom`) conversions
- Fine-grained control with field-level attributes

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
derive-convert = "0.1.0"
```

## Quick Start

```rust
use derive_convert::Convert;

// Source struct with conversion attributes
#[derive(Convert)]
#[convert(into = "Destination")] // Generate Into<Destination> implementation
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

## Conversion Types

The macro supports the following conversion types:

| Attribute | Description |
|-----------|-------------|
| `#[convert(into = "Type")]` | Implements `Into<Type>` for the struct/enum |
| `#[convert(try_from = "Type")]` | Implements `TryFrom<Type>` for the struct/enum |
| `#[convert(from = "Type")]` | Implements `From<Type>` for the struct/enum |

## Struct-Level Attributes

| Attribute | Description |
|-----------|-------------|
| `#[convert(into = "Type")]` | Generate an `Into<Type>` implementation |
| `#[convert(try_from = "Type")]` | Generate a `TryFrom<Type>` implementation |
| `#[convert(from = "Type")]` | Generate a `From<Type>` implementation |
| `#[convert(default)]` | Use `Default::default()` for fields not explicitly mapped |

## Field-Level Attributes

| Attribute | Description |
|-----------|-------------|
| `#[convert(rename = "new_name")]` | Map this field to a differently named field in the target type |
| `#[convert(unwrap)]` | Automatically unwrap an `Option` value (fails in `try_from` if `None`) |
| `#[convert(skip)]` | Skip this field during conversion (target must provide a default) |

## Enum Conversion

The macro supports enum-to-enum conversion with similar attribute control:

```rust
#[derive(Convert)]
#[convert(into = "TargetEnum")]
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
        renamed_field: u8,
    },
    Unit,
}
```

## Type Conversions

The macro intelligently handles various type scenarios:

1. **Direct Mapping**: Fields with identical types are directly copied
2. **Automatic Conversion**: Fields with types that implement `From`/`Into` are automatically converted
3. **Container Types**: Special handling for `Option<T>` and `Vec<T>` with inner type conversion
4. **Tuple Structs**: Support for conversions between tuple structs

## Examples

### Basic Struct Conversion

```rust
use derive_convert::Convert;

#[derive(Convert)]
#[convert(into = "Target")]
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
use derive_convert::Convert;

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

### Using Unwrap for Options

```rust
use derive_convert::Convert;

#[derive(Convert)]
#[convert(try_from = "Source")]
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
use derive_convert::Convert;

#[derive(Convert)]
#[convert(into = "Target", default)]
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

## License

This project is licensed under the MIT License - see the LICENSE file for details.
