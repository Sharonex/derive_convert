use derive_into::Convert;

// Custom types to demonstrate type conversion in enums
#[derive(Debug, PartialEq, Clone)]
struct CustomId(u64);

impl From<u64> for CustomId {
    fn from(id: u64) -> Self {
        CustomId(id)
    }
}

impl From<CustomId> for u64 {
    fn from(id: CustomId) -> Self {
        id.0
    }
}

// Source enum with different variant types
#[derive(Convert, Debug, PartialEq, Clone)]
#[convert(into = "TargetEvent")]
#[convert(try_from = "TargetEvent")]
enum SourceEvent {
    // Simple unit variant
    Heartbeat,

    // Tuple variant with simple type
    Click(u64),

    // Tuple variant with multiple values
    MouseMove(u64, u64),

    // Struct variant with renamed field
    Login {
        username: String,
        #[convert(rename = "auth_token")]
        token: String,
        timestamp: u64,
    },

    // Variant with renamed variant name
    #[convert(rename = "LogoutEvent")]
    Logout {
        username: String,
        timestamp: u64,
    },

    // Variant with Option that will be unwrapped
    Message {
        from: String,
        to: String,
        #[convert(unwrap)]
        content: Option<String>,
    },

    // Variant with nested conversion
    UserAction {
        user_id: u64,
        action_type: SourceActionType,
    },
}

// Target enum
#[derive(Debug, PartialEq, Clone)]
enum TargetEvent {
    // Direct match
    Heartbeat,

    // Type conversion in tuple variant
    Click(CustomId),

    // Multiple values in tuple variant
    MouseMove(CustomId, CustomId),

    // Struct variant with renamed field
    Login {
        username: String,
        auth_token: String, // Renamed from 'token'
        timestamp: CustomId,
    },

    // Renamed variant
    LogoutEvent {
        username: String,
        timestamp: CustomId,
    },

    // Value unwrapped from Option
    Message {
        from: String,
        to: String,
        content: String, // Unwrapped from Option
    },

    // Nested enum conversion
    UserAction {
        user_id: CustomId,
        action_type: TargetActionType,
    },
}

// Source nested enum
#[derive(Convert, Debug, PartialEq, Clone)]
#[convert(into = "TargetActionType")]
#[convert(try_from = "TargetActionType")]
enum SourceActionType {
    View,
    Edit,
    Delete,
    #[convert(rename = "CreateNew")]
    Create,
}

// Target nested enum
#[derive(Debug, PartialEq, Clone)]
enum TargetActionType {
    View,
    Edit,
    Delete,
    CreateNew, // Renamed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_variants() {
        // Test unit variant
        let source = SourceEvent::Heartbeat;
        let target: TargetEvent = source.into();
        assert_eq!(target, TargetEvent::Heartbeat);

        // Convert back
        let source_again = SourceEvent::try_from(target).unwrap();
        assert_eq!(source_again, SourceEvent::Heartbeat);

        // Test tuple variant with conversion
        let source = SourceEvent::Click(42);
        let target: TargetEvent = source.into();
        assert_eq!(target, TargetEvent::Click(CustomId(42)));

        // Convert back
        let source_again = SourceEvent::try_from(target).unwrap();
        assert_eq!(source_again, SourceEvent::Click(42));
    }

    #[test]
    fn test_multiple_tuple_elements() {
        let source = SourceEvent::MouseMove(10, 20);
        let target: TargetEvent = source.into();
        assert_eq!(target, TargetEvent::MouseMove(CustomId(10), CustomId(20)));

        // Convert back
        let source_again = SourceEvent::try_from(target).unwrap();
        assert_eq!(source_again, SourceEvent::MouseMove(10, 20));
    }

    #[test]
    fn test_struct_variants_with_rename() {
        let source = SourceEvent::Login {
            username: "user123".to_string(),
            token: "abc123".to_string(),
            timestamp: 1678901234,
        };

        let target: TargetEvent = source.clone().into();

        // Check field conversion and renaming
        match target {
            TargetEvent::Login {
                username,
                auth_token,
                timestamp,
            } => {
                assert_eq!(username, "user123");
                assert_eq!(auth_token, "abc123");
                assert_eq!(timestamp, CustomId(1678901234));
            }
            _ => panic!("Unexpected variant"),
        }

        // Convert back
        let source_again = SourceEvent::try_from(target).unwrap();
        assert_eq!(source_again, source);
    }

    #[test]
    fn test_renamed_variant() {
        let source = SourceEvent::Logout {
            username: "user123".to_string(),
            timestamp: 1678901235,
        };

        let target: TargetEvent = source.clone().into();

        // Check variant renaming
        match target {
            TargetEvent::LogoutEvent {
                username,
                timestamp,
            } => {
                assert_eq!(username, "user123");
                assert_eq!(timestamp, CustomId(1678901235));
            }
            _ => panic!("Unexpected variant"),
        }

        // Convert back
        let source_again = SourceEvent::try_from(target).unwrap();
        assert_eq!(source_again, source);
    }

    #[test]
    fn test_unwrapped_option() {
        let source = SourceEvent::Message {
            from: "alice".to_string(),
            to: "bob".to_string(),
            content: Some("Hello!".to_string()),
        };

        let target: TargetEvent = source.into();

        // Check Option unwrapping
        match target {
            TargetEvent::Message { from, to, content } => {
                assert_eq!(from, "alice");
                assert_eq!(to, "bob");
                assert_eq!(content, "Hello!");
            }
            _ => panic!("Unexpected variant"),
        }

        // Convert back should reconstruct the Option
        let source_again = SourceEvent::try_from(target).unwrap();
        match source_again {
            SourceEvent::Message { from, to, content } => {
                assert_eq!(from, "alice");
                assert_eq!(to, "bob");
                assert_eq!(content, Some("Hello!".to_string()));
            }
            _ => panic!("Unexpected variant"),
        }
    }

    #[test]
    fn test_unwrapped_option_failure() {
        // This should fail during TryFrom since content is None
        let source_with_none = SourceEvent::Message {
            from: "alice".to_string(),
            to: "bob".to_string(),
            content: None,
        };

        // This should fail during try_into due to unwrap on None
        let result: Result<TargetEvent, _> = source_with_none.try_into();
        assert!(result.is_err());
    }

    #[test]
    fn test_nested_enum_conversion() {
        let source = SourceEvent::UserAction {
            user_id: 999,
            action_type: SourceActionType::Create,
        };

        let target: TargetEvent = source.into();

        // Check nested enum conversion with renamed variant
        match target {
            TargetEvent::UserAction {
                user_id,
                action_type,
            } => {
                assert_eq!(user_id, CustomId(999));
                assert_eq!(action_type, TargetActionType::CreateNew);
            }
            _ => panic!("Unexpected variant"),
        }
    }
}

fn main() {
    // This allows the file to be run as a standalone example
    println!("Running enum conversion tests...");

    let source_event = SourceEvent::Login {
        username: "test_user".to_string(),
        token: "test_token".to_string(),
        timestamp: 1234567890,
    };

    let target_event: TargetEvent = source_event.into();
    println!("Converted to target event: {:#?}", target_event);

    // Another example with nested enum
    let source_action = SourceEvent::UserAction {
        user_id: 42,
        action_type: SourceActionType::Create,
    };

    let target_action: TargetEvent = source_action.into();
    println!("Converted action event: {:#?}", target_action);
}
