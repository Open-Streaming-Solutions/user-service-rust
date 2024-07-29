//use crate::adapters::repo::InternalRepository;
use crate::adapters::UserRepository;
use async_trait::async_trait;
use log::{error, info};
use std::sync::Arc;
use tonic::{Request, Response, Status};
use lib_rpc::rpc::user_service_server::UserService;
use lib_rpc::rpc::{
    GetAllUsersRequest, GetAllUsersResponse, GetUserByIdRequest, GetUserByIdResponse,
    GetUserIdByNicknameRequest, GetUserIdByNicknameResponse, PutUserRequest, PutUserResponse,
    UpdateUserRequest, UpdateUserResponse,
};
use uuid::Uuid;
use crate::types::User;

//use crate::adapters::internal_repo::InternalRepository;
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
    async fn get_user_data_by_id(
        &self, request: Request<GetUserByIdRequest>,
    ) -> Result<Response<GetUserByIdResponse>, Status> {
        info!(
            "Received GetUserData request for UUID: {}",
            request.get_ref().user_uuid
        );
        let user_uuid = request.into_inner().user_uuid;
        let user_id = Uuid::parse_str(&user_uuid).map_err(|_| {
            error!("Invalid UUID: {}", user_uuid);
            Status::invalid_argument("Invalid UUID")
        })?;
        if let Some(user) = self.repository.get_user(&user_id).await {
            let reply = GetUserByIdResponse {
                user_name: user.name,
                user_email: user.email,
            };
            info!("User data retrieved for UUID: {}", user_uuid);
            Ok(Response::new(reply))
        } else {
            error!("User with UUID {} not found", user_uuid);
            Err(Status::not_found("User not found"))
        }
    }
    async fn get_user_id_by_nickname(
        &self, request: Request<GetUserIdByNicknameRequest>,
    ) -> Result<Response<GetUserIdByNicknameResponse>, Status> {
        info!(
            "Received GetUserIdByNickname request for NickName: \"{}\"",
            request.get_ref().user_name
        );
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
            }
            None => {
                error!("User with NickName \"{}\" not found", user_name);
                Err(Status::not_found("User not found"))
            }
        }
    }

    async fn put_user_data(
        &self, request: Request<PutUserRequest>,
    ) -> Result<Response<PutUserResponse>, Status> {
        info!(
            "Received PutUserData request for UUID: {}",
            request.get_ref().user_uuid
        );
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
            name: req.user_name,
            email: req.user_email,
        };

        self.repository.add_user(user).await;
        info!("User {} added successfully", req.user_uuid);

        let reply = PutUserResponse {
            message: format!("User {} added successfully", req.user_uuid),
        };
        Ok(Response::new(reply))
    }

    async fn update_user_data(
        &self, request: Request<UpdateUserRequest>,
    ) -> Result<Response<UpdateUserResponse>, Status> {
        info!(
            "Received UpdateUserData request for UUID: {}",
            request.get_ref().user_uuid
        );
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
            user.name = req.user_name;
        }
        if !req.user_email.is_empty() {
            user.email = req.user_email;
        }

        self.repository.update_user_by_id(&user_id, user).await;
        info!("User {} updated successfully", req.user_uuid);

        let reply = UpdateUserResponse {
            message: format!("User {} updated successfully", req.user_uuid),
        };
        Ok(Response::new(reply))
    }

    async fn get_all_users(
        &self, _request: Request<GetAllUsersRequest>,
    ) -> Result<Response<GetAllUsersResponse>, Status> {
        info!("Received GetAllUsers request");
        let users = self.repository.get_all_users().await;
        let response_users: Vec<lib_rpc::rpc::User> = users
            .into_iter()
            .map(|user| lib_rpc::rpc::User {
                user_uuid: user.id.to_string(),
                user_name: user.name,
                user_email: user.email,
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

    use lib_rpc::rpc::user_service_server::UserService;
    use lib_rpc::rpc::{
        GetUserByIdRequest, GetUserIdByNicknameRequest, PutUserRequest, UpdateUserRequest,
    };


    use crate::adapters::UserRepository;
    use crate::types::User;
    use app::user_service::UserServiceCore;
    use crate::app;
    use crate::internal_repo::InternalRepository;

    #[tokio::test]
    async fn put_user_data_success() {
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

        assert_eq!(
            response_data.message,
            format!("User {} added successfully", user_id)
        );

        let added_user = repo.get_user(&user_id).await.unwrap();
        assert_eq!(added_user.name, "New User");
        assert_eq!(added_user.email, "new@example.com");
    }

    #[tokio::test]
    async fn put_user_data_invalid_uuid() {
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
    async fn put_user_data_duplicate_uuid() {
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

        service.put_user_data(request).await.unwrap();

        let duplicate_request = Request::new(put_user_request);

        let response = service.put_user_data(duplicate_request).await;
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
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        };
        repo.add_user(user).await;

        let service = UserServiceCore {
            repository: repo.clone(),
        };

        let request = Request::new(GetUserByIdRequest {
            user_uuid: user_id.to_string(),
        });

        let response = service.get_user_data_by_id(request).await.unwrap();
        let response_data = response.into_inner();

        assert_eq!(response_data.user_name, "Test User");
        assert_eq!(response_data.user_email, "test@example.com");
    }

    #[tokio::test]
    async fn update_user_data_success() {
        let repo = Arc::new(InternalRepository::new());
        let user_id = Uuid::now_v7();
        let user = User {
            id: user_id,
            name: "Existing User".to_string(),
            email: "existing@example.com".to_string(),
        };
        repo.add_user(user).await;

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

        assert_eq!(
            response_data.message,
            format!("User {} updated successfully", user_id)
        );

        let updated_user = repo.get_user(&user_id).await.unwrap();
        assert_eq!(updated_user.name, "Updated User");
        assert_eq!(updated_user.email, "updated@example.com");
    }

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

        repository.add_user(user.clone()).await;

        let request = Request::new(GetUserIdByNicknameRequest {
            user_name: "test_user".to_string(),
        });

        let response = service.get_user_id_by_nickname(request).await;
        assert!(response.is_ok(), "Expected OK response");
        let user_uuid = response.unwrap().into_inner().user_uuid;
        assert_eq!(user_uuid, user.id.to_string(), "User UUID does not match");

        let empty_request = Request::new(GetUserIdByNicknameRequest {
            user_name: "".to_string(),
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


}