use std::sync::Arc;
use tonic::{Request, Response, Status};
use uuid::Uuid;
use log::{error, info};
use user_service_rpc::rpc::{PutUserRequest,
                            PutUserResponse,
                            GetUserRequest,
                            GetUserResponse,
                            UpdateUserRequest,
                            UpdateUserResponse,
                            GetAllUsersRequest,
                            GetAllUsersResponse};
use user_service_rpc::rpc::user_service_server::UserService;
use crate::adapters::repo::InternalRepository;
use crate::app::structs::User;

#[derive(Clone)]
pub struct UserServiceCore {
    pub repository: Arc<InternalRepository>,
}

#[tonic::async_trait]
impl UserService for UserServiceCore {
    async fn get_user_data(&self, request: Request<GetUserRequest>) -> Result<Response<GetUserResponse>, Status> {
        info!("Received GetUserData request for UUID: {}", request.get_ref().user_uuid);
        get_user_data(&self.repository, request).await
    }

    async fn put_user_data(&self, request: Request<PutUserRequest>) -> Result<Response<PutUserResponse>, Status> {
        info!("Received PutUserData request for UUID: {}", request.get_ref().user_uuid);
        put_user_data(&self.repository, request).await
    }

    async fn update_user_data(&self, request: Request<UpdateUserRequest>) -> Result<Response<UpdateUserResponse>, Status> {
        info!("Received UpdateUserData request for UUID: {}", request.get_ref().user_uuid);
        update_user_data(&self.repository, request).await
    }
    async fn get_all_users(&self, _request: Request<GetAllUsersRequest>) -> Result<Response<GetAllUsersResponse>, Status> {
        info!("Received GetAllUsers request");
        get_all_users(&self.repository).await
    }
}

impl Default for UserServiceCore {
    fn default() -> Self {
        UserServiceCore {
            repository: Arc::new(InternalRepository::new()),
        }
    }
}

pub async fn get_user_data(repository: &InternalRepository, request: Request<GetUserRequest>) -> Result<Response<GetUserResponse>, Status> {
    let user_uuid = request.into_inner().user_uuid;
    let user_id = Uuid::parse_str(&user_uuid).map_err(|_| {
        error!("Invalid UUID: {}", user_uuid);
        Status::invalid_argument("Invalid UUID")
    })?;
    if let Some(user) = repository.get_user(&user_id) {
        let reply = GetUserResponse {
            user_name: user.user_name,
            user_email: user.user_email,
        };
        info!("User data retrieved for UUID: {}", user_uuid);
        Ok(Response::new(reply))
    } else {
        error!("User with UUID {} not found", user_uuid);
        Err(Status::not_found("User not found"))
    }
}

pub async fn put_user_data(repository: &InternalRepository, request: Request<PutUserRequest>) -> Result<Response<PutUserResponse>, Status> {
    let req = request.into_inner();
    let user_id = Uuid::parse_str(&req.user_uuid).map_err(|_| {
        error!("Invalid UUID: {}", req.user_uuid);
        Status::invalid_argument("Invalid UUID")
    })?;

    // Проверка на существование UUID
    if repository.get_user_id(&user_id).is_some() {
        error!("User with UUID {} already exists", user_id);
        return Err(Status::already_exists("User with this UUID already exists"));
    }

    let user = User {
        id: user_id,
        user_name: req.user_name,
        user_email: req.user_email,
    };

    repository.add_user(user);
    info!("User {} added successfully", req.user_uuid);

    let reply = PutUserResponse {
        message: format!("User {} added successfully", req.user_uuid),
    };
    Ok(Response::new(reply))
}

pub async fn update_user_data(repository: &InternalRepository, request: Request<UpdateUserRequest>) -> Result<Response<UpdateUserResponse>, Status> {
    let req = request.into_inner();
    let user_id = Uuid::parse_str(&req.user_uuid).map_err(|_| {
        error!("Invalid UUID: {}", req.user_uuid);
        Status::invalid_argument("Invalid UUID")
    })?;

    let mut user = repository.get_user(&user_id).ok_or_else(|| {
        error!("User with UUID {} not found", user_id);
        Status::not_found("User not found")
    })?;

    if !req.user_name.is_empty() {
        user.user_name = req.user_name;
    }
    if !req.user_email.is_empty() {
        user.user_email = req.user_email;
    }

    repository.add_user(user);
    info!("User {} updated successfully", req.user_uuid);

    let reply = UpdateUserResponse {
        message: format!("User {} updated successfully", req.user_uuid),
    };
    Ok(Response::new(reply))
}

pub async fn get_all_users(repository: &InternalRepository) -> Result<Response<GetAllUsersResponse>, Status> {
    let users = repository.get_all_users();
    let response_users: Vec<user_service_rpc::rpc::User> = users.into_iter().map(|user| user_service_rpc::rpc::User {
        user_uuid: user.id.to_string(),
        user_name: user.user_name,
        user_email: user.user_email,
    }).collect();

    let response = GetAllUsersResponse { users: response_users };
    Ok(Response::new(response))
}
#[cfg(test)]
mod user_service {
    use super::*;
    use tonic::Request;
    use uuid::Uuid;
    use crate::app::structs::User;
    use crate::adapters::repo::InternalRepository;

    #[tokio::test]
    async fn test_put_user_data_success() {
        let repo = Arc::new(InternalRepository::new());

        let service = UserServiceCore {
            repository: repo.clone(),
        };

        let user_id = Uuid::now_v7();
        let request = Request::new(PutUserRequest {
            user_uuid: user_id.to_string(),
            user_name: "New User".to_string(),
            user_email: "new@example.com".to_string(),
        });

        let response = service.put_user_data(request).await.unwrap();
        let response_data = response.into_inner();

        assert_eq!(response_data.message, format!("User {} added successfully", user_id));

        let added_user = repo.get_user(&user_id).unwrap();
        assert_eq!(added_user.user_name, "New User");
        assert_eq!(added_user.user_email, "new@example.com");
    }

    #[tokio::test]
    async fn test_put_user_data_invalid_uuid() {
        let repo = Arc::new(InternalRepository::new());

        let service = UserServiceCore {
            repository: repo.clone(),
        };

        let invalid_uuid = "invalid-uuid";
        let request = Request::new(PutUserRequest {
            user_uuid: invalid_uuid.to_string(),
            user_name: "New User".to_string(),
            user_email: "new@example.com".to_string(),
        });

        let response = service.put_user_data(request).await;
        assert!(response.is_err());
        let error = response.unwrap_err();
        assert_eq!(error.code(), tonic::Code::InvalidArgument);
        assert_eq!(error.message(), "Invalid UUID");
    }

    #[tokio::test]
    async fn test_put_user_data_duplicate_uuid() {
        let repo = Arc::new(InternalRepository::new());

        let service = UserServiceCore {
            repository: repo.clone(),
        };

        let user_id = Uuid::now_v7();
        let put_user_request = PutUserRequest {
            user_uuid: user_id.to_string(),
            user_name: "New User".to_string(),
            user_email: "new@example.com".to_string(),
        };
        let request = Request::new(put_user_request.clone());

        // Add user first time
        service.put_user_data(request).await.unwrap();

        // Create a new request with the same data
        let duplicate_request = Request::new(put_user_request);

        // Try adding the same user again
        let response = service.put_user_data(duplicate_request).await;
        assert!(response.is_err());
        let error = response.unwrap_err();
        assert_eq!(error.code(), tonic::Code::AlreadyExists);
        assert_eq!(error.message(), "User with this UUID already exists");
    }

    #[tokio::test]
    async fn test_get_user_data_success() {
        let repo = Arc::new(InternalRepository::new());
        let user_id = Uuid::now_v7();
        let user = User {
            id: user_id,
            user_name: "Test User".to_string(),
            user_email: "test@example.com".to_string(),
        };
        repo.add_user(user);

        let service = UserServiceCore {
            repository: repo.clone(),
        };

        let request = Request::new(GetUserRequest {
            user_uuid: user_id.to_string(),
        });

        let response = service.get_user_data(request).await.unwrap();
        let response_data = response.into_inner();

        assert_eq!(response_data.user_name, "Test User");
        assert_eq!(response_data.user_email, "test@example.com");
    }


    #[tokio::test]
    async fn test_update_user_data_success() {
        let repo = Arc::new(InternalRepository::new());
        let user_id = Uuid::now_v7();
        let user = User {
            id: user_id,
            user_name: "Existing User".to_string(),
            user_email: "existing@example.com".to_string(),
        };
        repo.add_user(user);

        let service = UserServiceCore {
            repository: repo.clone(),
        };

        let request = Request::new(UpdateUserRequest {
            user_uuid: user_id.to_string(),
            user_name: "Updated User".to_string(),
            user_email: "updated@example.com".to_string(),
        });

        let response = service.update_user_data(request).await.unwrap();
        let response_data = response.into_inner();

        assert_eq!(response_data.message, format!("User {} updated successfully", user_id));

        let updated_user = repo.get_user(&user_id).unwrap();
        assert_eq!(updated_user.user_name, "Updated User");
        assert_eq!(updated_user.user_email, "updated@example.com");
    }
}
