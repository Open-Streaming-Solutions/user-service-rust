use core::*;
use std::sync::Arc;
use tonic::Request;
use uuid::Uuid;
use user_service_rpc::rpc::{GetUserByIdRequest, GetUserIdByNicknameRequest, PutUserRequest, UpdateUserRequest};
use user_service_rpc::rpc::user_service_server::UserService;
use crate::app::structs::User;
use crate::adapters::repo::InternalRepository;
use crate::adapters::UserRepository;
use crate::app::user_service::core::UserServiceCore;

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

        assert_eq!(response_data.message, format!("User {} added successfully", user_id));

        let added_user = repo.get_user(&user_id).await.unwrap();
        assert_eq!(added_user.user_name, "New User");
        assert_eq!(added_user.user_email, "new@example.com");
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
    async fn get_user_data_success() {
        let repo = Arc::new(InternalRepository::new());
        let user_id = Uuid::now_v7();
        let user = User {
            id: user_id,
            user_name: "Test User".to_string(),
            user_email: "test@example.com".to_string(),
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
            user_name: "Existing User".to_string(),
            user_email: "existing@example.com".to_string(),
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

        assert_eq!(response_data.message, format!("User {} updated successfully", user_id));

        let updated_user = repo.get_user(&user_id).await.unwrap();
        assert_eq!(updated_user.user_name, "Updated User");
        assert_eq!(updated_user.user_email, "updated@example.com");
    }
#[tokio::test]
async fn get_user_id_by_nickname() {
    let repository = Arc::new(InternalRepository::new());
    let service = UserServiceCore {
        repository: repository.clone(),
    };

    let user = User {
        id: Uuid::now_v7(),
        user_name: "test_user".to_string(),
        user_email: "test_user@example.com".to_string(),
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
    assert_eq!(status.code(), tonic::Code::InvalidArgument, "Expected InvalidArgument status");
}
