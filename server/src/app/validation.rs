use crate::errors::GrpcError;
use log::error;
use uuid::Uuid;

pub fn validate_uuid(uuid_str: &str) -> Result<Uuid, GrpcError> {
    Uuid::parse_str(uuid_str).map_err(|_| {
        error!("Invalid UUID: {}", uuid_str);
        GrpcError::InvalidArgument("Invalid UUID".to_string())
    })
}

pub fn validate_user_name(name: &str) -> Result<(), GrpcError> {
    if name.is_empty() {
        error!("User name cannot be empty");
        return Err(GrpcError::InvalidArgument(
            "User name cannot be empty".to_string(),
        ));
    }
    Ok(())
}

pub fn validate_user_email(email: &str) -> Result<(), GrpcError> {
    if email.is_empty() {
        error!("User email cannot be empty");
        return Err(GrpcError::InvalidArgument(
            "User email cannot be empty".to_string(),
        ));
        //Доделать
    }
    //Доделать
    Ok(())
}
