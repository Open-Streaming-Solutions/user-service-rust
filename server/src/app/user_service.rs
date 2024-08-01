use std::sync::Arc;

use async_trait::async_trait;
use log::{error, info};
use tonic::{Request, Response, Status};

use lib_rpc::userpb::user_service_server::UserService;
use lib_rpc::userpb::{
    CreateUserRequest, GetAllUsersRequest, GetAllUsersResponse, GetUserByIdRequest,
    GetUserByIdResponse, GetUserRequest, GetUserResponse,
    UpdateUserRequest, UpdateUserResponse,
};
use prost_types::Option;
use crate::app::validation::{validate_user_email, validate_user_name, validate_uuid};
use crate::errors::{AppError, RepoError};
use crate::repo::UserRepository;
use crate::types::User;

#[derive(Clone)]
pub struct UserServiceCore<R: UserRepository> {
    repository: Arc<R>,
}

impl<R: UserRepository> UserServiceCore<R> {
    pub async fn new(repository: Arc<R>) -> Self {
        UserServiceCore { repository }
    }
}
//ToDo Попробовать убрать static
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
        validate_user_name(&user_name).await?;

        self.repository.get_user_id_by_username(&user_name).await
            .map_err(|e| {
                error!("Failed to get user ID by nickname: {:?}", e);
                Status::internal("Internal server error")
            })
            .and_then(|opt| match opt {
                Some(user_id) => Ok(Response::new(GetUserResponse { uuid: user_id.to_string() })),
                None => {
                    error!("User with NickName \"{}\" not found", user_name);
                    Err(Status::not_found("User not found"))
                }
            })
    }

    async fn create_user(
        &self, request: Request<CreateUserRequest>,
    ) -> Result<Response<()>, Status> {
        info!(
        "Received CreateUser request for UUID: {}",
        request.get_ref().uuid
    );
        let req = request.into_inner();
        let user_id = validate_uuid(&req.uuid).await?;
        validate_user_name(&req.username).await?;
        validate_user_email(&req.email).await?;

        if self.repository.get_user_id(&user_id).await.map_err(|e| {
            error!("Failed to get user ID: {:?}", e);
            Status::internal("Internal server error")
        })?.is_some() {
            error!("User with UUID {} already exists", user_id);
            return Err(Status::already_exists("User with this UUID already exists"));
        }

        let user = User::new(user_id, req.username, req.email).await;
        self.repository.add_user(user).await.map_err(|e| {
            error!("Failed to add user: {:?}", e);
            match e {
                RepoError::AlreadyExists(msg) => Status::already_exists(msg),
                RepoError::DbError(..) => Status::internal("Internal Server Error"),
                _ => Status::internal("Internal server error"),
            }
        })?;
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
        let user_id = validate_uuid(&user_uuid).await?;

        self.repository.get_user(&user_id).await
            .map_err(|e| {
                error!("Failed to get user: {:?}", e);
                Status::internal("Internal server error")
            })
            .and_then(|opt| match opt {
                Some(user) => {
                    let reply = GetUserByIdResponse {
                        username: user.get_username().to_string(),
                        email: user.get_email().to_string(),
                    };
                    info!("User data retrieved for UUID: {}", user_uuid);
                    Ok(Response::new(reply))
                },
                None => {
                    error!("User with UUID {} not found", user_uuid);
                    Err(Status::not_found("User not found"))
                }
            })
    }

    async fn update_user_data(
        &self, request: Request<UpdateUserRequest>,
    ) -> Result<Response<UpdateUserResponse>, Status> {
        info!(
        "Received UpdateUserData request for UUID: {}",
        request.get_ref().uuid
    );
        let req = request.into_inner();
        let user_id = validate_uuid(&req.uuid).await?;

        let mut user = self.repository.get_user(&user_id).await
            .map_err(|e| {
                error!("Failed to get user: {:?}", e);
                Status::internal("Internal server error")
            })?
            .ok_or_else(|| {
                error!("User with UUID {} not found", user_id);
                Status::not_found("User not found")
            })?;

        let mut updated = false;

        // Проверка и обновление username
        if let Some(new_username) = req.username {
            if user.get_username() != new_username {
                validate_user_name(&new_username).await?;
                user.set_username(new_username);
                updated = true;
            }
        }

        // Проверка и обновление email
        if let Some(new_email) = req.email {
            if user.get_email() != new_email {
                validate_user_email(&new_email).await?;
                user.set_email(new_email);
                updated = true;
            }
        }

        if updated {
            self.repository.update_user_by_id(&user_id, user).await.map_err(|e| {
                error!("Failed to update user: {:?}", e);
                Status::internal("Internal server error")
            })?;
            info!("User {} updated successfully", req.uuid);

            let reply = UpdateUserResponse {
                message: format!("User {} updated successfully", req.uuid),
            };
            Ok(Response::new(reply))
        } else {
            info!("No changes detected for user {}", req.uuid);
            let reply = UpdateUserResponse {
                message: format!("No changes detected for user {}", req.uuid),
            };
            Ok(Response::new(reply))
        }
    }

    async fn get_all_users(
        &self, _request: Request<GetAllUsersRequest>,
    ) -> Result<Response<GetAllUsersResponse>, Status> {
        info!("Received GetAllUsers request");
        let users = self.repository.get_all_users().await.map_err(|e| {
            error!("Failed to get all users: {:?}", e);
            Status::internal("Internal server error")
        })?;
        let response_users: Vec<lib_rpc::userpb::User> = users.into_iter().map(|user| {
            lib_rpc::userpb::User {
                uuid: user.get_id().to_string(),
                username: user.get_username().to_string(),
                email: user.get_email().to_string(),
            }
        }).collect();

        let response = GetAllUsersResponse {
            users: response_users,
        };
        Ok(Response::new(response))
    }
}