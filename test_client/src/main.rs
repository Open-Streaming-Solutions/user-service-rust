use clap::{Parser, ValueEnum};
use futures::future::join_all;
use lib_rpc::userpb::user_service_client::UserServiceClient;
use lib_rpc::userpb::{
    CreateUserRequest, GetAllUsersRequest, GetUserByIdRequest, GetUserRequest, UpdateUserRequest,
};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use tokio::task;
use tonic::transport::Channel;
use uuid::Uuid;

#[derive(Debug, ValueEnum, Clone)]
enum Actions {
    GetUserDataById,
    CreateUser,
    Update,
    GetAll,
    GetUserIdByNickname,
    Generate,
}

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about = "Типо клиент")]
struct Args {
    #[arg(long = "host", default_value = "User-Service-Server")]
    target_host: String,
    #[arg(short = 'p', long = "port", default_value_t = 8080)]
    target_port: u16,

    #[arg(short = 'a', long, value_enum)]
    action: Actions,

    #[arg(short = 'i', long)]
    uuid: Option<String>,

    #[arg(short = 'n', long, default_value_t = String::new())]
    username: String,

    #[arg(short = 'e', long, default_value_t = String::new())]
    email: String,

    #[arg(short = 'g', long, default_value_t = 1)]
    generate_count: usize,
}

async fn create_user(
    client: &mut UserServiceClient<Channel>, user_uuid: &Uuid, user_name: &str, user_email: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let request = tonic::Request::new(CreateUserRequest {
        uuid: user_uuid.to_string(),
        username: user_name.to_string(),
        email: user_email.to_string(),
    });

    let response = client.create_user(request).await?;
    println!("CreateUser={:?}", response);

    Ok(())
}

async fn get_user_data_by_id(
    client: &mut UserServiceClient<Channel>, user_uuid: &Uuid,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let request = tonic::Request::new(GetUserByIdRequest {
        uuid: user_uuid.to_string(),
    });

    let response = client.get_user_data_by_id(request).await?;
    println!("GetUserDataById={:?}", response);

    Ok(())
}

async fn get_user(
    client: &mut UserServiceClient<Channel>, username: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let request = tonic::Request::new(GetUserRequest { username });

    let response = client.get_user(request).await?;
    println!("GetUserIdByNickname={:?}", response);

    Ok(())
}

async fn update_user_data(
    client: &mut UserServiceClient<Channel>, user_uuid: &Uuid, user_name: Option<&str>,
    user_email: Option<&str>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let request = tonic::Request::new(UpdateUserRequest {
        uuid: user_uuid.to_string(),
        username: user_name.unwrap_or_default().to_string(),
        email: user_email.unwrap_or_default().to_string(),
    });

    let response = client.update_user_data(request).await?;
    println!("UpdateUserData={:?}", response);

    Ok(())
}

async fn get_all_users(
    client: &mut UserServiceClient<Channel>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let request = tonic::Request::new(GetAllUsersRequest {});
    let response = client.get_all_users(request).await?;
    println!("GetAllUsers={:?}", response);

    let users = response.into_inner().users;

    for user in users {
        println!(
            "UUID: {}, Name: {}, Email: {}",
            user.uuid, user.username, user.email
        );
    }

    Ok(())
}

fn generate_random_user() -> (Uuid, String, String) {
    let user_id = Uuid::now_v7();
    let user_name: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect();
    let user_email = format!("{}@example.com", user_name);
    (user_id, user_name, user_email)
}

async fn generate_users(
    client: UserServiceClient<Channel>, count: usize,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut futures = Vec::new();
    for _ in 0..count {
        let (user_id, user_name, user_email) = generate_random_user();
        let mut client = client.clone();
        futures.push(task::spawn(async move {
            create_user(&mut client, &user_id, &user_name, &user_email).await
        }));
    }
    let results = join_all(futures).await;
    for result in results {
        result??;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let addr = format!("http://{}:{}", args.target_host, args.target_port);

    let client = UserServiceClient::connect(addr.clone()).await?;

    match args.action {
        Actions::CreateUser | Actions::GetUserDataById | Actions::Update => {
            let user_uuid_str = args
                .uuid
                .as_deref()
                .ok_or("user_uuid is required for this action")?;
            let user_uuid = Uuid::parse_str(user_uuid_str)?;

            match args.action {
                Actions::CreateUser => {
                    create_user(&mut client.clone(), &user_uuid, &args.username, &args.email)
                        .await
                        .unwrap();
                }
                Actions::GetUserDataById => {
                    get_user_data_by_id(&mut client.clone(), &user_uuid)
                        .await
                        .unwrap();
                }
                Actions::Update => {
                    update_user_data(
                        &mut client.clone(),
                        &user_uuid,
                        Some(&args.username),
                        Some(&args.email),
                    )
                    .await
                    .unwrap();
                }
                _ => unreachable!(),
            }
        }
        Actions::GetAll => {
            get_all_users(&mut client.clone()).await.unwrap();
        }
        Actions::GetUserIdByNickname => {
            get_user(&mut client.clone(), args.username).await.unwrap();
        }
        Actions::Generate => {
            generate_users(client, args.generate_count).await.unwrap();
        }
    }

    Ok(())
}
