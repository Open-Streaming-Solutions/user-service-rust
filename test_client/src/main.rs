use clap::{Parser, ValueEnum};
use lib_rpc::rpc::user_service_client::UserServiceClient;
use lib_rpc::rpc::{
    GetAllUsersRequest, GetUserByIdRequest, GetUserIdByNicknameRequest, PutUserRequest,
    UpdateUserRequest,
};
use tonic::transport::Channel;
use uuid::Uuid;

#[derive(Debug, ValueEnum, Clone)]
enum Actions {
    GetUserDataById,
    Put,
    Update,
    GetAll,
    GetUserIdByNickname,
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
    user_uuid: Option<String>,

    #[arg(short = 'n', long, default_value_t = String::new())]
    user_name: String,

    #[arg(short = 'e', long, default_value_t = String::new())]
    user_email: String,
}

async fn put_user_data(
    client: &mut UserServiceClient<Channel>, user_uuid: &Uuid, user_name: &str, user_email: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let request = tonic::Request::new(PutUserRequest {
        user_uuid: user_uuid.to_string(),
        user_name: user_name.to_string(),
        user_email: user_email.to_string(),
    });

    let response = client.put_user_data(request).await?;
    println!("PutUserData={:?}", response);

    Ok(())
}

async fn get_user_data_by_id(
    client: &mut UserServiceClient<Channel>, user_uuid: &Uuid,
) -> Result<(), Box<dyn std::error::Error>> {
    let request = tonic::Request::new(GetUserByIdRequest {
        user_uuid: user_uuid.to_string(),
    });

    let response = client.get_user_data_by_id(request).await?;
    println!("GetUserData={:?}", response);

    Ok(())
}

async fn get_user_id_by_nickname(
    client: &mut UserServiceClient<Channel>, user_name: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let request = tonic::Request::new(GetUserIdByNicknameRequest { user_name });

    let response = client.get_user_id_by_nickname(request).await?;
    println!("GetUserIdByNickname={:?}", response);

    Ok(())
}

async fn update_user_data(
    client: &mut UserServiceClient<Channel>, user_uuid: &Uuid, user_name: Option<&str>,
    user_email: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let request = tonic::Request::new(UpdateUserRequest {
        user_uuid: user_uuid.to_string(),
        user_name: user_name.unwrap_or_default().to_string(),
        user_email: user_email.unwrap_or_default().to_string(),
    });

    let response = client.update_user_data(request).await?;
    println!("UpdateUserData={:?}", response);

    Ok(())
}

async fn get_all_users(
    client: &mut UserServiceClient<Channel>,
) -> Result<(), Box<dyn std::error::Error>> {
    let request = tonic::Request::new(GetAllUsersRequest {});
    let response = client.get_all_users(request).await?;
    println!("GetAllUsers={:?}", response);

    let users = response.into_inner().users;

    for user in users {
        println!(
            "UUID: {}, Name: {}, Email: {}",
            user.user_uuid, user.user_name, user.user_email
        );
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let addr = format!("http://{}:{}", args.target_host, args.target_port);

    let mut client = UserServiceClient::connect(addr).await?;

    match args.action {
        Actions::Put | Actions::GetUserDataById | Actions::Update => {
            let user_uuid_str = args
                .user_uuid
                .as_deref()
                .ok_or("user_uuid is required for this action")?;
            let user_uuid = Uuid::parse_str(user_uuid_str)?;

            match args.action {
                Actions::Put => {
                    put_user_data(&mut client, &user_uuid, &args.user_name, &args.user_email)
                        .await?;
                }
                Actions::GetUserDataById => {
                    get_user_data_by_id(&mut client, &user_uuid).await?;
                }
                Actions::Update => {
                    update_user_data(
                        &mut client,
                        &user_uuid,
                        Some(&args.user_name),
                        Some(&args.user_email),
                    )
                    .await?;
                }
                _ => unreachable!(),
            }
        }
        Actions::GetAll => {
            get_all_users(&mut client).await?;
        }
        Actions::GetUserIdByNickname => {
            get_user_id_by_nickname(&mut client, args.user_name).await?;
        }
    }

    Ok(())
}
