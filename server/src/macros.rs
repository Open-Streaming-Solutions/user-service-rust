#[macro_export]
macro_rules! parse_uuid {
    ($uuid_str:expr) => {
        Uuid::parse_str($uuid_str).map_err(|_| {
            error!("Invalid UUID: {}", $uuid_str);
            $crate::errors::GrpcError::InvalidArgument("Invalid UUID".to_string())
        })?
    };
}