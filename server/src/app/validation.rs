use crate::errors::{AppError};
use log::trace;
use uuid::Uuid;
use regex::Regex;
use tonic::Status;

pub async fn validate_uuid(uuid_str: &str) -> Result<Uuid, Status> {
    Uuid::parse_str(uuid_str).map_err(|_| {
        trace!("Invalid UUID: {}", uuid_str);
        Status::invalid_argument("Invalid UUID")
    })
}

pub async fn validate_user_name(name: &str) -> Result<(), Status> {
    if name.is_empty() {
        trace!("User name cannot be empty");
        return  Err(Status::invalid_argument("User name cannot be empty"))
    }
    Ok(())
}

pub async fn validate_user_email(email: &str) -> Result<(), Status> {
    if email.is_empty() {
        trace!("User email cannot be empty");
        return Err(Status::invalid_argument("User email cannot be empty"));
    }
    let email_regex = Regex::new(r"^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+$").unwrap();

    if !email_regex.is_match(email) {
        trace!("Invalid email format: {}", email);
        return Err(Status::invalid_argument("Invalid email format"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_validate_uuid() {
        let valid_uuid = Uuid::now_v7().to_string();
        let result = validate_uuid(&valid_uuid).await;
        assert!(result.is_ok());

        let invalid_uuid = "invalid-uuid";
        let result = validate_uuid(invalid_uuid).await;
        assert!(result.is_err());
        assert_eq!(result.err().unwrap().to_string(), "status: InvalidArgument, message: \"Invalid UUID\", details: [], metadata: MetadataMap { headers: {} }");
    }

    #[tokio::test]
    async fn test_validate_user_name() {
        let valid_name = "testuser";
        let result = validate_user_name(valid_name).await;
        assert!(result.is_ok());

        let invalid_name = "";
        let result = validate_user_name(invalid_name).await;
        assert!(result.is_err());
        assert_eq!(result.err().unwrap().to_string(), "status: InvalidArgument, message: \"User name cannot be empty\", details: [], metadata: MetadataMap { headers: {} }");
    }

    #[tokio::test]
    async fn test_validate_user_email() {
        let valid_email = "testuser@example.com";
        let result = validate_user_email(valid_email).await;
        assert!(result.is_ok());

        let invalid_email = "";
        let result = validate_user_email(invalid_email).await;
        assert!(result.is_err());
        assert_eq!(result.err().unwrap().to_string(), "status: InvalidArgument, message: \"User email cannot be empty\", details: [], metadata: MetadataMap { headers: {} }");

        let invalid_email = "invalid-email";
        let result = validate_user_email(invalid_email).await;
        assert!(result.is_err());
        assert_eq!(result.err().unwrap().to_string(), "status: InvalidArgument, message: \"Invalid email format\", details: [], metadata: MetadataMap { headers: {} }");
    }
}