use crate::errors::GrpcError;
use log::{error, trace};
use uuid::Uuid;
use regex::Regex;

pub fn validate_uuid(uuid_str: &str) -> Result<Uuid, GrpcError> {
    Uuid::parse_str(uuid_str).map_err(|_| {
        trace!("Invalid UUID: {}", uuid_str);
        GrpcError::InvalidArgument("Invalid UUID".to_string())
    })
}

pub fn validate_user_name(name: &str) -> Result<(), GrpcError> {
    if name.is_empty() {
        trace!("User name cannot be empty");
        return Err(GrpcError::InvalidArgument(
            "User name cannot be empty".to_string(),
        ));
    }
    Ok(())
}

pub fn validate_user_email(email: &str) -> Result<(), GrpcError> {
    if email.is_empty() {
        trace!("User email cannot be empty");
        return Err(GrpcError::InvalidArgument(
            "User email cannot be empty".to_string(),
        ));
    }
    let email_regex = Regex::new(r"^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+$")
        .map_err(|_| GrpcError::Internal("Failed to compile regex".to_string()))?;

    if !email_regex.is_match(email) {
        trace!("Invalid email format: {}", email);
        return Err(GrpcError::InvalidArgument("Invalid email format".to_string()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use crate::errors::GrpcError;

    #[test]
    fn test_validate_uuid() {
        let valid_uuid = Uuid::now_v7().to_string();
        let result = validate_uuid(&valid_uuid);
        assert!(result.is_ok());

        let invalid_uuid = "invalid-uuid";
        let result = validate_uuid(invalid_uuid);
        assert!(result.is_err());
        assert_eq!(result.err().unwrap().to_string(), "Invalid argument: Invalid UUID");
    }

    #[test]
    fn test_validate_user_name() {
        let valid_name = "testuser";
        let result = validate_user_name(valid_name);
        assert!(result.is_ok());

        let invalid_name = "";
        let result = validate_user_name(invalid_name);
        assert!(result.is_err());
        assert_eq!(result.err().unwrap().to_string(), "Invalid argument: User name cannot be empty");
    }

    #[test]
    fn test_validate_user_email() {
        let valid_email = "testuser@example.com";
        let result = validate_user_email(valid_email);
        assert!(result.is_ok());

        let invalid_email = "";
        let result = validate_user_email(invalid_email);
        assert!(result.is_err());
        assert_eq!(result.err().unwrap().to_string(), "Invalid argument: User email cannot be empty");

        let invalid_email = "invalid-email";
        let result = validate_user_email(invalid_email);
        assert!(result.is_err());
        assert_eq!(result.err().unwrap().to_string(), "Invalid argument: Invalid email format");
    }
}
