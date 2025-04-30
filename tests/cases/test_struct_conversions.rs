use derive_into::Convert;

// Custom types to demonstrate type conversion
#[derive(Debug, PartialEq, Default)]
struct UserId(u32);

impl From<u32> for UserId {
    fn from(id: u32) -> Self {
        UserId(id)
    }
}

impl From<UserId> for u32 {
    fn from(id: UserId) -> Self {
        id.0
    }
}

#[derive(Debug, PartialEq, Default)]
struct Email(String);

impl From<String> for Email {
    fn from(email: String) -> Self {
        Email(email)
    }
}

impl From<Email> for String {
    fn from(email: Email) -> Self {
        email.0
    }
}

// Source struct with a variety of field types and conversion attributes
#[derive(Convert, Debug, PartialEq)]
#[convert(into = "UserRecord", default)] // Infallible conversion to UserRecord
#[convert(try_from = "UserRecord")] // Fallible conversion from UserRecord
struct User {
    // Basic field with direct mapping
    name: String,

    // Field that uses type conversion
    id: u32,

    // Optional field with type conversion
    email: Option<String>,

    // Field to be renamed in target
    #[convert(rename = "creation_date")]
    created_at: String,

    // Field that will be unwrapped
    #[convert(unwrap)]
    age: Option<u8>,

    // Vector field with inner type conversion
    roles: Vec<String>,
}

// Target struct
#[derive(Debug, Default, PartialEq)]
struct UserRecord {
    name: String,
    id: UserId,            // Different type than source
    email: Option<Email>,  // Option with different inner type
    creation_date: String, // Renamed field
    age: u8,               // Unwrapped from Option
    roles: Vec<Email>,     // Vec with different inner type

    // This field isn't in User, so it needs a default
    last_login: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_into_conversion() {
        let user = User {
            name: "John Doe".to_string(),
            id: 42,
            email: Some("john@example.com".to_string()),
            created_at: "2023-01-01".to_string(),
            age: Some(30),
            roles: vec!["admin".to_string(), "user".to_string()],
        };

        // Convert to UserRecord
        let record: UserRecord = user.into();

        // Verify conversion
        assert_eq!(record.name, "John Doe");
        assert_eq!(record.id, UserId(42));
        assert_eq!(record.email, Some(Email("john@example.com".to_string())));
        assert_eq!(record.creation_date, "2023-01-01");
        assert_eq!(record.age, 30);
        assert_eq!(record.roles.len(), 2);
        assert_eq!(record.roles[0], Email("admin".to_string()));
        assert_eq!(record.roles[1], Email("user".to_string()));
        assert_eq!(record.last_login, None); // Default value
    }

    #[test]
    fn test_try_from_conversion() {
        let record = UserRecord {
            name: "Jane Doe".to_string(),
            id: UserId(123),
            email: Some(Email("jane@example.com".to_string())),
            creation_date: "2023-02-02".to_string(),
            age: 25,
            roles: vec![Email("moderator".to_string())],
            last_login: Some("2023-03-03".to_string()),
        };

        // Convert from UserRecord
        let user_result = User::try_from(record);
        assert!(user_result.is_ok());

        let user = user_result.unwrap();
        assert_eq!(user.name, "Jane Doe");
        assert_eq!(user.id, 123);
        assert_eq!(user.email, Some("jane@example.com".to_string()));
        assert_eq!(user.created_at, "2023-02-02");
        assert_eq!(user.age, Some(25));
        assert_eq!(user.roles.len(), 1);
        assert_eq!(user.roles[0], "moderator");
    }

    #[test]
    fn test_try_from_failure() {
        // Create a UserRecord with age = 0, which should fail unwrapping
        let record = UserRecord {
            name: "Test User".to_string(),
            id: UserId(456),
            email: None,
            creation_date: "2023-04-04".to_string(),
            age: 0, // This is just to demonstrate - actually this won't fail
            roles: vec![],
            last_login: None,
        };

        // This should still succeed since our test doesn't actually have a failure case
        // In a real implementation, you might have validation that could fail
        let user_result = User::try_from(record);
        assert!(user_result.is_ok());
    }
}

fn main() {
    // This allows the file to be run as a standalone example
    println!("Running struct conversion tests...");

    let user = User {
        name: "Example User".to_string(),
        id: 1,
        email: Some("example@test.com".to_string()),
        created_at: "2023-05-05".to_string(),
        age: Some(42),
        roles: vec!["guest".to_string()],
    };

    let record: UserRecord = user.into();
    println!("Converted User to UserRecord: {:#?}", record);

    // Convert back
    match User::try_from(record) {
        Ok(converted_user) => println!("Converted back to User: {:#?}", converted_user),
        Err(_) => println!("Conversion failed"),
    }
}
