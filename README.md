# derive_convert

Derive macro that generates From/TryFrom/Into/TryInto methods.

## Example

```rust
#[derive(Convert)]
#[convert(into = "B")]
#[convert(from = "B")]
pub struct A {
    pub normal: Option<u8>,

    // auto into of inner
    pub opt: Option<u8>,
    // auto into of inner
    pub vec: Vec<u8>,

    #[convert(rename = "renamed_field")]
    pub old_name: u16,
}

pub struct B {
    normal: u8,
    opt: Option<Number>,
    vec: Vec<Number>,
    renamed_field: Number,
}
```
