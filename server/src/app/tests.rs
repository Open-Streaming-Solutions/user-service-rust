use pretty_assertions::assert_eq;
use tonic::Request;
use uuid::Uuid;
use std::sync::Arc;
use crate::types::User;
use crate::app::user_service::UserServiceCore;

use lib_rpc::userpb::user_service_server::UserService;
use lib_rpc::userpb::{CreateUserRequest, GetUserByIdRequest, UpdateUserRequest};

use crate::repo::UserRepository;
use crate::errors::RepoError;
use async_trait::async_trait;
use dashmap::DashMap;

//ToDO Переделать под моки
pub struct InternalRepository {
    storage: Arc<DashMap<Uuid, User>>,
}

impl InternalRepository {
    pub fn new() -> Self {
        InternalRepository {
            storage: Arc::new(DashMap::new()),
        }
    }
}

#[async_trait]
impl UserRepository for InternalRepository {
    async fn add_user(&self, user: User) -> Result<(), RepoError> {
        self.storage.insert(user.get_id(), user);
        Ok(())
    }

    async fn get_all_users(&self) -> Result<Vec<User>, RepoError> {
        let users: Vec<User> = self.storage.iter().map(|kv| kv.value().clone()).collect();
        Ok(users)
    }

    async fn get_user(&self, user_id: &Uuid) -> Result<Option<User>, RepoError> {
        Ok(self.storage.get(user_id).map(|user| user.clone()))
    }

    async fn get_user_id(&self, user_id: &Uuid) -> Result<Option<Uuid>, RepoError> {
        Ok(self.storage.get(user_id).map(|kv| *kv.key()))
    }

    async fn get_user_id_by_username(&self, user_name: &str) -> Result<Option<Uuid>, RepoError> {
        Ok(self
            .storage
            .iter()
            .find(|kv| kv.value().get_username() == user_name)
            .map(|kv| *kv.key()))
    }

    async fn update_user_by_id(
        &self, user_id: &Uuid, updated_user: User,
    ) -> Result<Option<()>, RepoError> {
        if self.storage.contains_key(user_id) {
            self.storage.insert(*user_id, updated_user);
            Ok(Some(()))
        } else {
            Ok(None)
        }
    }

    async fn update_user_by_username(
        &self, nick_name: &str, updated_user: User,
    ) -> Result<Option<()>, RepoError> {
        if let Some(user_id) = self.get_user_id_by_username(nick_name).await? {
            self.update_user_by_id(&user_id, updated_user).await
        } else {
            Ok(None)
        }
    }
}

#[tokio::test]
async fn create_user_success() {
    let repo = Arc::new(InternalRepository::new());

    let user_id = Uuid::now_v7();
    let user = User::new(user_id, "New User".to_string(), "new@example.com".to_string()).await;
    let service = UserServiceCore::new(repo.clone()).await;

    let request = Request::new(CreateUserRequest {
        uuid: user_id.to_string(),
        username: user.get_username().to_string(),
        email: user.get_email().to_string(),
    });

    let response = service.create_user(request).await;
    assert!(response.is_ok(), "Expected Ok response");

    let added_user = repo.get_user(&user_id).await.unwrap();
    assert_eq!(added_user.as_ref().unwrap().get_username(), "New User");
    assert_eq!(added_user.unwrap().get_email(), "new@example.com");
}

#[tokio::test]
async fn create_user_invalid_uuid() {
    let repo = Arc::new(InternalRepository::new());
    let service = UserServiceCore::new(repo.clone()).await;

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
async fn create_user_duplicate_uuid() {
    let repo = Arc::new(InternalRepository::new());

    let user_id = Uuid::now_v7();
    let user = User::new(user_id, "New User".to_string(), "new@example.com".to_string()).await;
    repo.add_user(user.clone()).await.unwrap();
    let service = UserServiceCore::new(repo.clone()).await;

    let request = Request::new(CreateUserRequest {
        uuid: user_id.to_string(),
        username: user.get_username().to_string(),
        email: user.get_email().to_string(),
    });

    let response = service.create_user(request).await;
    assert!(response.is_err());
    let error = response.unwrap_err();
    assert_eq!(error.code(), tonic::Code::AlreadyExists);
    assert_eq!(error.message(), "User with this UUID already exists");
}

#[tokio::test]
async fn get_user_data_success() {
    let repo = Arc::new(InternalRepository::new());
    let user_id = Uuid::now_v7();
    let user = User::new(user_id, "Test User".to_string(), "test@example.com".to_string()).await;
    repo.add_user(user.clone()).await.unwrap();
    let service = UserServiceCore::new(repo.clone()).await;

    let request = Request::new(GetUserByIdRequest {
        uuid: user_id.to_string(),
    });

    let response = service.get_user_data_by_id(request).await.unwrap();
    let response_data = response.into_inner();

    assert_eq!(response_data.username, user.get_username());
    assert_eq!(response_data.email, user.get_email());
}

#[tokio::test]
async fn update_user_data_success() {
    let repo = Arc::new(InternalRepository::new());
    let user_id = Uuid::now_v7();
    let user = User::new(user_id, "Existing User".to_string(), "existing@example.com".to_string()).await;
    repo.add_user(user.clone()).await.unwrap();
    let service = UserServiceCore::new(repo.clone()).await;

    let updated_user = User::new(user_id, "Updated User".to_string(), "updated@example.com".to_string()).await;

    let request = Request::new(UpdateUserRequest {
        uuid: user_id.to_string(),
        username: Some(updated_user.get_username().to_string()),
        email: Some(updated_user.get_email().to_string()),
    });

    let response = service.update_user_data(request).await.unwrap();
    let response_data = response.into_inner();

    assert_eq!(
        response_data.message,
        format!("User {} updated successfully", user_id)
    );

    let updated_user = repo.get_user(&user_id).await.unwrap();
    assert_eq!(updated_user.as_ref().unwrap().get_username(), "Updated User");
    assert_eq!(updated_user.unwrap().get_email(), "updated@example.com");
}

#[tokio::test]
async fn update_user_data_invalid_uuid() {
    let repo = Arc::new(InternalRepository::new());
    let service = UserServiceCore::new(repo.clone()).await;

    let invalid_uuid = "invalid-uuid".to_string();
    let request = Request::new(UpdateUserRequest {
        uuid: invalid_uuid,
        username: Some("Updated User".to_string()),
        email: Some("updated@example.com".to_string()),
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
    let service = UserServiceCore::new(repo.clone()).await;

    let user_id = Uuid::now_v7();

    let non_existent_uuid = user_id.to_string();
    let request = Request::new(UpdateUserRequest {
        uuid: non_existent_uuid,
        username: Some("Updated User".to_string()),
        email: Some("updated@example.com".to_string()),
    });

    let response = service.update_user_data(request).await;
    assert!(response.is_err(), "Expected error response");
    let status = response.err().unwrap();
    assert_eq!(status.code(), tonic::Code::NotFound);
    assert_eq!(status.message(), "User not found");
}