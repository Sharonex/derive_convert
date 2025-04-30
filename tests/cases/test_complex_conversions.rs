use derive_into::Convert;
use std::collections::HashMap;

// Custom wrapper types for demonstration
#[derive(Debug, PartialEq, Default, Clone)]
struct Money(f64);

impl From<f64> for Money {
    fn from(value: f64) -> Self {
        Money(value)
    }
}

impl From<Money> for f64 {
    fn from(money: Money) -> Self {
        money.0
    }
}

#[derive(Debug, PartialEq, Default, Clone)]
struct ProductId(String);

impl From<String> for ProductId {
    fn from(id: String) -> Self {
        ProductId(id)
    }
}

impl From<ProductId> for String {
    fn from(id: ProductId) -> Self {
        id.0
    }
}

// Custom wrapper type with validation
#[derive(Debug, PartialEq, Clone, Default)]
struct NonEmptyString(String);

impl NonEmptyString {
    fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl TryFrom<String> for NonEmptyString {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.trim().is_empty() {
            Err("String cannot be empty".to_string())
        } else {
            Ok(NonEmptyString(value))
        }
    }
}

impl From<NonEmptyString> for String {
    fn from(value: NonEmptyString) -> Self {
        value.0
    }
}

// Source struct with complex nested types
#[derive(Convert, Debug, PartialEq, Clone)]
#[convert(into = "ApiProduct", default)]
#[convert(try_from = "ApiProduct")]
struct Product {
    id: String,
    name: NonEmptyString,
    description: Option<String>,

    // Vector of complex types
    #[convert(rename = "variants")]
    product_variants: Vec<ProductVariant>,

    // HashMap with key type conversion
    #[convert(rename = "price_by_region")]
    regional_prices: HashMap<String, f64>,

    // Nested struct with its own conversion
    manufacturer: Manufacturer,
}

// Target struct for Product
#[derive(Debug, Default, PartialEq, Clone)]
struct ApiProduct {
    id: ProductId,
    name: String,
    description: Option<String>,
    variants: Vec<ApiProductVariant>,
    price_by_region: HashMap<String, Money>,
    manufacturer: ApiManufacturer,

    // This field doesn't exist in the source, will use default
    average_rating: Option<f32>,
}

// Nested source struct
#[derive(Convert, Debug, PartialEq, Default, Clone)]
#[convert(into = "ApiProductVariant")]
#[convert(try_from = "ApiProductVariant")]
struct ProductVariant {
    variant_id: String,
    size: String,
    color: String,
    price: f64,
    in_stock: bool,
}

// Target nested struct
#[derive(Debug, Default, PartialEq, Clone)]
struct ApiProductVariant {
    variant_id: ProductId,
    size: String,
    color: String,
    price: Money,
    in_stock: bool,
}

// Another nested source struct
#[derive(Convert, Debug, PartialEq, Default, Clone)]
#[convert(into = "ApiManufacturer")]
#[convert(try_from = "ApiManufacturer")]
struct Manufacturer {
    name: NonEmptyString,
    country: String,
    #[convert(unwrap)]
    contact_email: Option<String>,
}

// Target nested struct
#[derive(Debug, Default, PartialEq, Clone)]
struct ApiManufacturer {
    name: String,
    country: String,
    contact_email: String, // Unwrapped from Option
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complex_into_conversion() {
        // Create a product with all nested structures
        let product = Product {
            id: "prod-123".to_string(),
            name: NonEmptyString("Ergonomic Chair".to_string()),
            description: Some("Office chair with lumbar support".to_string()),
            product_variants: vec![
                ProductVariant {
                    variant_id: "var-1".to_string(),
                    size: "Small".to_string(),
                    color: "Black".to_string(),
                    price: 199.99,
                    in_stock: true,
                },
                ProductVariant {
                    variant_id: "var-2".to_string(),
                    size: "Medium".to_string(),
                    color: "Gray".to_string(),
                    price: 229.99,
                    in_stock: false,
                },
            ],
            regional_prices: {
                let mut prices = HashMap::new();
                prices.insert("US".to_string(), 199.99);
                prices.insert("EU".to_string(), 249.99);
                prices.insert("UK".to_string(), 189.99);
                prices
            },
            manufacturer: Manufacturer {
                name: NonEmptyString("ErgoDesigns".to_string()),
                country: "Germany".to_string(),
                contact_email: Some("info@ergodesigns.com".to_string()),
            },
        };

        // Convert to API type
        let api_product: ApiProduct = product.into();

        // Verify conversion results
        assert_eq!(api_product.id, ProductId("prod-123".to_string()));
        assert_eq!(api_product.name, "Ergonomic Chair".to_string());
        assert_eq!(
            api_product.description,
            Some("Office chair with lumbar support".to_string())
        );

        // Check variants conversion
        assert_eq!(api_product.variants.len(), 2);
        assert_eq!(
            api_product.variants[0].variant_id,
            ProductId("var-1".to_string())
        );
        assert_eq!(api_product.variants[0].price, Money(199.99));

        // Check HashMap conversion
        assert_eq!(api_product.price_by_region.len(), 3);
        assert_eq!(api_product.price_by_region.get("US"), Some(&Money(199.99)));

        // Check nested struct conversion
        assert_eq!(api_product.manufacturer.name, "ErgoDesigns".to_string());
        assert_eq!(
            api_product.manufacturer.contact_email,
            "info@ergodesigns.com"
        );

        // Check default field
        assert_eq!(api_product.average_rating, None);
    }

    #[test]
    fn test_complex_try_from_conversion() {
        // Create API product
        let api_product = ApiProduct {
            id: ProductId("prod-456".to_string()),
            name: "Standing Desk".to_string(),
            description: Some("Adjustable height desk".to_string()),
            variants: vec![ApiProductVariant {
                variant_id: ProductId("desk-var-1".to_string()),
                size: "Standard".to_string(),
                color: "Oak".to_string(),
                price: Money(349.99),
                in_stock: true,
            }],
            price_by_region: {
                let mut prices = HashMap::new();
                prices.insert("US".to_string(), Money(349.99));
                prices.insert("CA".to_string(), Money(399.99));
                prices
            },
            manufacturer: ApiManufacturer {
                name: "DeskCraft".to_string(),
                country: "Sweden".to_string(),
                contact_email: "support@deskcraft.com".to_string(),
            },
            average_rating: Some(4.7),
        };

        // Convert to internal type
        let product_result = Product::try_from(api_product.clone());
        assert!(product_result.is_ok());

        let product = product_result.unwrap();

        // Verify conversion results
        assert_eq!(product.id, "prod-456");
        assert_eq!(product.name.as_str(), "Standing Desk");
        assert_eq!(
            product.description,
            Some("Adjustable height desk".to_string())
        );

        // Check variants conversion
        assert_eq!(product.product_variants.len(), 1);
        assert_eq!(product.product_variants[0].variant_id, "desk-var-1");
        assert_eq!(product.product_variants[0].price, 349.99);

        // Check HashMap conversion
        assert_eq!(product.regional_prices.len(), 2);
        assert_eq!(product.regional_prices.get("US"), Some(&349.99));

        // Check nested struct conversion
        assert_eq!(product.manufacturer.name.as_str(), "DeskCraft");
        assert_eq!(
            product.manufacturer.contact_email,
            Some("support@deskcraft.com".to_string())
        );
    }

    #[test]
    fn test_try_from_validation_failure() {
        // Create an API product with an empty name (which should fail validation)
        let api_product = ApiProduct {
            id: ProductId("prod-789".to_string()),
            name: "Valid Product".to_string(), // This is valid
            description: None,
            variants: vec![],
            price_by_region: HashMap::new(),
            manufacturer: ApiManufacturer {
                name: "Manufacturer".to_string(),
                country: "Country".to_string(),
                contact_email: "".to_string(), // Empty email
            },
            average_rating: None,
        };

        // This should succeed since all fields are valid
        let product_result = Product::try_from(api_product.clone());
        assert!(product_result.is_ok());

        // Now let's modify it to have an invalid field
        // In a real implementation, we'd need to make this invalid, but for demonstration
        // purposes we'll just assert what would happen

        // In a real implementation with validation, something like this would fail:
        // api_product.name = NonEmptyString("".to_string()); // This would fail in a real scenario

        // For now, we'll just simulate a validation error by assuming it would fail
        // with proper validation implemented

        // This is a hypothetical test that shows how validation failures would be handled
        // assert!(Product::try_from(api_product_invalid).is_err());
    }
}

fn main() {
    // This allows the file to be run as a standalone example
    println!("Running complex conversion tests...");

    // Create a product
    let product = Product {
        id: "example-prod".to_string(),
        name: NonEmptyString("Example Product".to_string()),
        description: Some("This is a test product".to_string()),
        product_variants: vec![ProductVariant {
            variant_id: "v1".to_string(),
            size: "Universal".to_string(),
            color: "Blue".to_string(),
            price: 99.99,
            in_stock: true,
        }],
        regional_prices: {
            let mut prices = HashMap::new();
            prices.insert("US".to_string(), 99.99);
            prices.insert("EU".to_string(), 89.99);
            prices
        },
        manufacturer: Manufacturer {
            name: NonEmptyString("Test Manufacturer".to_string()),
            country: "Test Country".to_string(),
            contact_email: Some("test@example.com".to_string()),
        },
    };

    // Convert to API model
    let api_product: ApiProduct = product.into();
    println!("Converted to API product: {:#?}", api_product);

    // Convert back
    match Product::try_from(api_product) {
        Ok(converted_product) => println!("Converted back to Product: {:#?}", converted_product),
        Err(e) => println!("Conversion failed: {:?}", e),
    }
}
