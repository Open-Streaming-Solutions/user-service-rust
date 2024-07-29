use std::sync::Arc;

use async_trait::async_trait;
use log::{error, info};
use tonic::{Request, Response, Status};

use lib_rpc::userpb::user_service_server::UserService;
use lib_rpc::userpb::{
    CreateUserRequest, GetAllUsersRequest, GetAllUsersResponse, GetUserByIdRequest,
    GetUserByIdResponse, GetUserIdByNicknameResponse, GetUserRequest, GetUserResponse,
    UpdateUserRequest, UpdateUserResponse,
};

use crate::app::validation::{validate_user_email, validate_user_name, validate_uuid};
use crate::errors::GrpcError;
use crate::repo::UserRepository;
use crate::types::User;

#[derive(Clone)]
pub struct UserServiceCore<R: UserRepository> {
    pub repository: Arc<R>,
}

#[async_trait]
impl<R: UserRepository + 'static> UserService for UserServiceCore<R> {
    async fn get_user(
        &self, request: Request<GetUserRequest>,
    ) -> Result<Response<GetUserResponse>, Status> {
        info!(
            "Received GetUser request for NickName: \"{}\"",
            request.get_ref().username
        );
        let user_name = request.into_inner().username;

        validate_user_name(&user_name)?;

        match self.repository.get_user_id_by_nickname(&user_name).await {
            Ok(Some(user_id)) => {
                let response = GetUserResponse {
                    uuid: user_id.to_string(),
                };
                Ok(Response::new(response))
            }
            Ok(None) => {
                error!("User with NickName \"{}\" not found", user_name);
                Err(GrpcError::NotFound("User not found".to_string()).into())
            }
            Err(e) => Err(GrpcError::from(e).into()),
        }
    }

    async fn create_user(
        &self, request: Request<CreateUserRequest>,
    ) -> Result<Response<()>, Status> {
        info!(
            "Received CreateUser request for UUID: {}",
            request.get_ref().uuid
        );
        let req = request.into_inner();
        let user_id = validate_uuid(&req.uuid)?;
        validate_user_name(&req.username)?;
        validate_user_email(&req.email)?;

        if self
            .repository
            .get_user_id(&user_id)
            .await
            .map_err(GrpcError::from)?
            .is_some()
        {
            error!("User with UUID {} already exists", user_id);
            return Err(
                GrpcError::AlreadyExists("User with this UUID already exists".to_string()).into(),
            );
        }

        let user = User {
            id: user_id,
            username: req.username,
            email: req.email,
        };

        self.repository
            .add_user(user)
            .await
            .map_err(GrpcError::from)?;
        info!("User {} added successfully", req.uuid);

        Ok(Response::new(()))
    }

    async fn get_user_data_by_id(
        &self, request: Request<GetUserByIdRequest>,
    ) -> Result<Response<GetUserByIdResponse>, Status> {
        info!(
            "Received GetUserData request for UUID: {}",
            request.get_ref().uuid
        );
        let user_uuid = request.into_inner().uuid;
        let user_id = validate_uuid(&user_uuid)?;
        if let Some(user) = self
            .repository
            .get_user(&user_id)
            .await
            .map_err(GrpcError::from)?
        {
            let reply = GetUserByIdResponse {
                username: user.username,
                email: user.email,
            };
            info!("User data retrieved for UUID: {}", user_uuid);
            Ok(Response::new(reply))
        } else {
            error!("User with UUID {} not found", user_uuid);
            Err(GrpcError::NotFound("User not found".to_string()).into())
        }
    }

    async fn update_user_data(
        &self, request: Request<UpdateUserRequest>,
    ) -> Result<Response<UpdateUserResponse>, Status> {
        info!(
            "Received UpdateUserData request for UUID: {}",
            request.get_ref().uuid
        );
        let req = request.into_inner();
        let user_id = validate_uuid(&req.uuid)?;
        if !req.username.is_empty() {
            validate_user_name(&req.username)?;
        }
        if !req.email.is_empty() {
            validate_user_email(&req.email)?;
        }

        let mut user = self
            .repository
            .get_user(&user_id)
            .await
            .map_err(GrpcError::from)?
            .ok_or_else(|| {
                error!("User with UUID {} not found", user_id);
                GrpcError::NotFound("User not found".to_string())
            })?;

        if !req.username.is_empty() {
            user.username = req.username;
        }
        if !req.email.is_empty() {
            user.email = req.email;
        }

        self.repository
            .update_user_by_id(&user_id, user)
            .await
            .map_err(GrpcError::from)?;
        info!("User {} updated successfully", req.uuid);

        let reply = UpdateUserResponse {
            message: format!("User {} updated successfully", req.uuid),
        };
        Ok(Response::new(reply))
    }

    async fn get_all_users(
        &self, _request: Request<GetAllUsersRequest>,
    ) -> Result<Response<GetAllUsersResponse>, Status> {
        info!("Received GetAllUsers request");
        let users = self
            .repository
            .get_all_users()
            .await
            .map_err(GrpcError::from)?;
        let response_users: Vec<lib_rpc::userpb::User> = users
            .into_iter()
            .map(|user| lib_rpc::userpb::User {
                uuid: user.id.to_string(),
                username: user.username,
                email: user.email,
            })
            .collect();

        let response = GetAllUsersResponse {
            users: response_users,
        };
        Ok(Response::new(response))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use pretty_assertions::assert_eq;
    use tonic::Request;
    use uuid::Uuid;

    use app::user_service::UserServiceCore;
    use lib_rpc::userpb::user_service_server::UserService;
    use lib_rpc::userpb::{CreateUserRequest, GetUserByIdRequest, UpdateUserRequest};

    use crate::app;
    use crate::repo::internal::InternalRepository;
    use crate::repo::UserRepository;
    use crate::types::User;

    //переделать на проверку кода ответа
    #[tokio::test]
    async fn create_user_success() {
        let repo = Arc::new(InternalRepository::new());

        let service = UserServiceCore {
            repository: repo.clone(),
        };

        let user_id = Uuid::now_v7();
        let request = Request::new(CreateUserRequest {
            uuid: user_id.to_string(),
            username: "New User".to_string(),
            email: "new@example.com".to_string(),
        });

        let response = service.create_user(request).await;
        assert!(response.is_ok(), "Expected Ok response");

        let added_user = repo.get_user(&user_id).await.unwrap();
        assert_eq!(added_user.as_ref().unwrap().username, "New User");
        assert_eq!(added_user.unwrap().email, "new@example.com");
    }

    #[tokio::test]
    async fn put_user_data_invalid_uuid() {
        let repo = Arc::new(InternalRepository::new());

        let service = UserServiceCore {
            repository: repo.clone(),
        };

        let invalid_uuid = "invalid-uuid";
        let request = Request::new(CreateUserRequest {
            uuid: invalid_uuid.to_string(),
            username: "New User".to_string(),
            email: "new@example.com".to_string(),
        });

        let response = service.create_user(request).await;
        assert!(response.is_err());
        let error = response.unwrap_err();
        assert_eq!(error.code(), tonic::Code::InvalidArgument);
        assert_eq!(error.message(), "Invalid UUID");
    }

    #[tokio::test]
    async fn put_user_data_duplicate_uuid() {
        let repo = Arc::new(InternalRepository::new());

        let service = UserServiceCore {
            repository: repo.clone(),
        };

        let user_id = Uuid::now_v7();
        let put_user_request = CreateUserRequest {
            uuid: user_id.to_string(),
            username: "New User".to_string(),
            email: "new@example.com".to_string(),
        };
        let request = Request::new(put_user_request.clone());

        service.create_user(request).await.unwrap();

        let duplicate_request = Request::new(put_user_request);

        let response = service.create_user(duplicate_request).await;
        assert!(response.is_err());
        let error = response.unwrap_err();
        assert_eq!(error.code(), tonic::Code::AlreadyExists);
        assert_eq!(error.message(), "User with this UUID already exists");
    }

    #[tokio::test]
    async fn get_user_data_success() {
        let repo = Arc::new(InternalRepository::new());
        let user_id = Uuid::now_v7();
        let user = User {
            id: user_id,
            username: "Test User".to_string(),
            email: "test@example.com".to_string(),
        };
        repo.add_user(user).await.unwrap();

        let service = UserServiceCore {
            repository: repo.clone(),
        };

        let request = Request::new(GetUserByIdRequest {
            uuid: user_id.to_string(),
        });

        let response = service.get_user_data_by_id(request).await.unwrap();
        let response_data = response.into_inner();

        assert_eq!(response_data.username, "Test User");
        assert_eq!(response_data.email, "test@example.com");
    }

    #[tokio::test]
    async fn update_user_data_success() {
        let repo = Arc::new(InternalRepository::new());
        let user_id = Uuid::now_v7();
        let user = User {
            id: user_id,
            username: "Existing User".to_string(),
            email: "existing@example.com".to_string(),
        };
        repo.add_user(user).await.unwrap();

        let service = UserServiceCore {
            repository: repo.clone(),
        };

        let request = Request::new(UpdateUserRequest {
            uuid: user_id.to_string(),
            username: "Updated User".to_string(),
            email: "updated@example.com".to_string(),
        });

        let response = service.update_user_data(request).await.unwrap();
        let response_data = response.into_inner();

        assert_eq!(
            response_data.message,
            format!("User {} updated successfully", user_id)
        );

        let updated_user = repo.get_user(&user_id).await.unwrap();
        assert_eq!(updated_user.as_ref().unwrap().username, "Updated User");
        assert_eq!(updated_user.unwrap().email, "updated@example.com");
    }
    /*
        #[tokio::test]
        async fn get_user_id_by_nickname() {
            let repository = Arc::new(InternalRepository::new());
            let service = UserServiceCore {
                repository: repository.clone(),
            };

            let user = User {
                id: Uuid::now_v7(),
                name: "test_user".to_string(),
                email: "test_user@example.com".to_string(),
            };

            repository.add_user(user.clone()).await.unwrap();

            let request = Request::new(GetUserIdByNicknameRequest {
                username: "test_user".to_string(),
            });

            let response = service.get_user(request).await;
            assert!(response.is_ok(), "Expected OK response");
            let user_uuid = response.unwrap().into_inner().user_uuid;
            assert_eq!(user_uuid, user.id.to_string(), "User UUID does not match");

            let empty_request = Request::new(GetUserIdByNicknameRequest {
                username: "".to_string(),
            });

            let empty_response = service.get_user_id_by_nickname(empty_request).await;
            assert!(empty_response.is_err(), "Expected error response");
            let status = empty_response.err().unwrap();
            assert_eq!(
                status.code(),
                tonic::Code::InvalidArgument,
                "Expected InvalidArgument status"
            );
        }
    */
    #[tokio::test]
    async fn update_user_data_invalid_uuid() {
        let repo = Arc::new(InternalRepository::new());

        let service = UserServiceCore {
            repository: repo.clone(),
        };

        let invalid_uuid = "invalid-uuid".to_string();
        let request = Request::new(UpdateUserRequest {
            uuid: invalid_uuid,
            username: "Updated User".to_string(),
            email: "updated@example.com".to_string(),
        });

        let response = service.update_user_data(request).await;
        assert!(response.is_err(), "Expected error response");
        let status = response.err().unwrap();
        assert_eq!(status.code(), tonic::Code::InvalidArgument);
        assert_eq!(status.message(), "Invalid UUID");
    }

    #[tokio::test]
    async fn update_user_data_not_found() {
        let repo = Arc::new(InternalRepository::new());

        let service = UserServiceCore {
            repository: repo.clone(),
        };

        let non_existent_uuid = Uuid::now_v7().to_string();
        let request = Request::new(UpdateUserRequest {
            uuid: non_existent_uuid,
            username: "Updated User".to_string(),
            email: "updated@example.com".to_string(),
        });

        let response = service.update_user_data(request).await;
        assert!(response.is_err(), "Expected error response");
        let status = response.err().unwrap();
        assert_eq!(status.code(), tonic::Code::NotFound);
        assert_eq!(status.message(), "User not found");
    }
}
