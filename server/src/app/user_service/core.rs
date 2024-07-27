use std::sync::Arc;
use tonic::{Request, Response, Status};
use uuid::Uuid;
use log::{error, info};
use user_service_rpc::rpc::{PutUserRequest, PutUserResponse, GetUserByIdRequest, UpdateUserRequest, UpdateUserResponse, GetAllUsersRequest, GetAllUsersResponse, GetUserByIdResponse, GetUserIdByNicknameRequest, GetUserIdByNicknameResponse};
use user_service_rpc::rpc::user_service_server::UserService;
use crate::adapters::repo::InternalRepository;
use crate::app::structs::User;
use async_trait::async_trait;
use crate::adapters::UserRepository;

/*
Читается вот так:
Структура, С любым типом R Который должен реализовывать трейт UserRepository
*/
#[derive(Clone)]
pub struct UserServiceCore<R: UserRepository> {
    pub repository: Arc<R>,
}

/*
Читается вот так:
Реализация UserService (в котором находится описание gRPC) для структуры UserServiceCore с обобщенным типом R где
тип R должен реализовывать трейт UserRepository и иметь время жизни 'static,
это гарантирует, что R не содержит временных ссылок (ссылок на данные с ограниченным временем жизни)
и безопасен для передачи между потоками.

Возможно это можно сделать по-другому, но это первое что пришло в голову, если я хочу и DbRepository и Internalrepository использовать.
*/
#[async_trait]
impl<R: UserRepository + 'static> UserService for UserServiceCore<R> {
    async fn get_user_data_by_id(&self, request: Request<GetUserByIdRequest>) -> Result<Response<GetUserByIdResponse>, Status> {
        info!("Received GetUserData request for UUID: {}", request.get_ref().user_uuid);
        let user_uuid = request.into_inner().user_uuid;
        let user_id = Uuid::parse_str(&user_uuid).map_err(|_| {
            error!("Invalid UUID: {}", user_uuid);
            Status::invalid_argument("Invalid UUID")
        })?;
        if let Some(user) = self.repository.get_user(&user_id).await {
            let reply = GetUserByIdResponse {
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
    async fn get_user_id_by_nickname(&self, request: Request<GetUserIdByNicknameRequest>)
                                     -> Result<Response<GetUserIdByNicknameResponse>, Status> {
        info!("Received GetUserIdByNickname request for NickName: \"{}\"", request.get_ref().user_name);
        let user_name = request.into_inner().user_name;

        if user_name.is_empty() {
            error!("Received empty user_name in GetUserIdByNickname request");
            return Err(Status::invalid_argument("user_name cannot be empty"));
        }

        match self.repository.get_user_id_by_nickname(&user_name).await {
            Some(user_id) => {
                let response = GetUserIdByNicknameResponse {
                    user_uuid: user_id.to_string(),
                };
                Ok(Response::new(response))
            },
            None => {
                error!("User with NickName \"{}\" not found", user_name);
                Err(Status::not_found("User not found"))
            }
        }
    }

    async fn put_user_data(&self, request: Request<PutUserRequest>) -> Result<Response<PutUserResponse>, Status> {
        info!("Received PutUserData request for UUID: {}", request.get_ref().user_uuid);
        let req = request.into_inner();
        let user_id = Uuid::parse_str(&req.user_uuid).map_err(|_| {
            error!("Invalid UUID: {}", req.user_uuid);
            Status::invalid_argument("Invalid UUID")
        })?;

        // Проверка на существование UUID
        if self.repository.get_user_id(&user_id).await.is_some() {
            error!("User with UUID {} already exists", user_id);
            return Err(Status::already_exists("User with this UUID already exists"));
        }

        let user = User {
            id: user_id,
            user_name: req.user_name,
            user_email: req.user_email,
        };

        self.repository.add_user(user).await;
        info!("User {} added successfully", req.user_uuid);

        let reply = PutUserResponse {
            message: format!("User {} added successfully", req.user_uuid),
        };
        Ok(Response::new(reply))
    }

    async fn update_user_data(&self, request: Request<UpdateUserRequest>) -> Result<Response<UpdateUserResponse>, Status> {
        info!("Received UpdateUserData request for UUID: {}", request.get_ref().user_uuid);
        let req = request.into_inner();
        let user_id = Uuid::parse_str(&req.user_uuid).map_err(|_| {
            error!("Invalid UUID: {}", req.user_uuid);
            Status::invalid_argument("Invalid UUID")
        })?;

        let mut user = self.repository.get_user(&user_id).await.ok_or_else(|| {
            error!("User with UUID {} not found", user_id);
            Status::not_found("User not found")
        })?;

        if !req.user_name.is_empty() {
            user.user_name = req.user_name;
        }
        if !req.user_email.is_empty() {
            user.user_email = req.user_email;
        }

        self.repository.add_user(user).await;
        info!("User {} updated successfully", req.user_uuid);

        let reply = UpdateUserResponse {
            message: format!("User {} updated successfully", req.user_uuid),
        };
        Ok(Response::new(reply))
    }

    async fn get_all_users(&self, _request: Request<GetAllUsersRequest>) -> Result<Response<GetAllUsersResponse>, Status> {
        info!("Received GetAllUsers request");
        let users = self.repository.get_all_users().await;
        let response_users: Vec<user_service_rpc::rpc::User> = users.into_iter().map(|user| user_service_rpc::rpc::User {
            user_uuid: user.id.to_string(),
            user_name: user.user_name,
            user_email: user.user_email,
        }).collect();

        let response = GetAllUsersResponse { users: response_users };
        Ok(Response::new(response))
    }
}

impl Default for UserServiceCore<InternalRepository> {
    fn default() -> Self {
        UserServiceCore {
            repository: Arc::new(InternalRepository::new()),
        }
    }
}

impl Default for UserServiceCore<DbRepository> {
    fn default() -> Self {
        UserServiceCore {
            repository: Arc::new(InternalRepository::new()),
        }
    }
}
