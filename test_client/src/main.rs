/*
cargo run -- -a put -i 0189a30a-60c7-7135-b683-7d7f3783d4b7 -n test1 -e test@test.ru
cargo run -- -a put -i 0189a30a-60c7-7136-b98e-9c2d4f2734f1 -n test2 -e test@test.ru
cargo run -- -a put -i 0189a30a-60c7-7137-b1a3-8a6a3d9076fa -n test3 -e test@test.ru
cargo run -- -a put -i 0189a30a-60c7-7138-8a68-2d4c5b617a98 -n test4 -e test@test.ru
cargo run -- -a put -i 0189a30a-60c7-7139-b187-2e7e3b297efb -n test5 -e test@test.ru

cargo run -- -a get -i 0189a30a-60c7-7135-b683-7d7f3783d4b7
cargo run -- -a get -i 0189a30a-60c7-7136-b98e-9c2d4f2734f1
cargo run -- -a get -i 0189a30a-60c7-7137-b1a3-8a6a3d9076fa
cargo run -- -a get -i 0189a30a-60c7-7138-8a68-2d4c5b617a98
cargo run -- -a get -i 0189a30a-60c7-7139-b187-2e7e3b297efb

cargo run -- -a update -i 0189a30a-60c7-7135-b683-7d7f3783d4b7 -e "mod1@test.ru"
cargo run -- -a update -i 0189a30a-60c7-7136-b98e-9c2d4f2734f1 -e "mod2@test.ru"
cargo run -- -a update -i 0189a30a-60c7-7137-b1a3-8a6a3d9076fa -e "mod3@test.ru"
cargo run -- -a update -i 0189a30a-60c7-7138-8a68-2d4c5b617a98 -e "mod4@test.ru"
cargo run -- -a update -i 0189a30a-60c7-7139-b187-2e7e3b297efb -e "mod5@test.ru"

cargo run -- -a get-all

*/

/*
0189a30a-60c7-7135-b683-7d7f3783d4b7
0189a30a-60c7-7136-b98e-9c2d4f2734f1
0189a30a-60c7-7137-b1a3-8a6a3d9076fa
0189a30a-60c7-7138-8a68-2d4c5b617a98
0189a30a-60c7-7139-b187-2e7e3b297efb

*/

use tonic::transport::Channel;
use uuid::Uuid;
use user_service_rpc::rpc::{GetAllUsersRequest, GetUserRequest, PutUserRequest, UpdateUserRequest};
use user_service_rpc::rpc::user_service_client::UserServiceClient;

use clap::{Parser, ValueEnum};
#[derive(Debug, ValueEnum, Clone)]
enum Actions {
    Get,
    Put,
    Update,
    GetAll,
}

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about = "Типо клиент")]
struct Args {
    #[arg(short = 'a', long, value_enum)]
    action: Actions,

    #[arg(short = 'i', long)]
    user_uuid: Option<String>,

    #[arg(short = 'n', long, default_value_t = String::new())]
    user_name: String,

    #[arg(short = 'e', long, default_value_t = String::new())]
    user_email: String,
}

async fn put_user_data(client: &mut UserServiceClient<Channel>, user_uuid: &Uuid, user_name: &str, user_email: &str) -> Result<(), Box<dyn std::error::Error>> {
    let request = tonic::Request::new(PutUserRequest {
        user_uuid: user_uuid.to_string(),
        user_name: user_name.to_string(),
        user_email: user_email.to_string(),
    });

    let response = client.put_user_data(request).await?;
    println!("PutUserData={:?}", response);

    Ok(())
}

async fn get_user_data(client: &mut UserServiceClient<Channel>, user_uuid: &Uuid) -> Result<(), Box<dyn std::error::Error>> {
    let request = tonic::Request::new(GetUserRequest {
        user_uuid: user_uuid.to_string(),
    });

    let response = client.get_user_data(request).await?;
    println!("GetUserData={:?}", response);

    Ok(())
}

async fn update_user_data(client: &mut UserServiceClient<Channel>, user_uuid: &Uuid, user_name: Option<&str>, user_email: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let request = tonic::Request::new(UpdateUserRequest {
        user_uuid: user_uuid.to_string(),
        user_name: user_name.unwrap_or_default().to_string(),
        user_email: user_email.unwrap_or_default().to_string(),
    });

    let response = client.update_user_data(request).await?;
    println!("UpdateUserData={:?}", response);

    Ok(())
}

async fn get_all_users(client: &mut UserServiceClient<Channel>) -> Result<(), Box<dyn std::error::Error>> {
    let request = tonic::Request::new(GetAllUsersRequest {});
    let response = client.get_all_users(request).await?;
    println!("GetAllUsers={:?}", response);

    let users = response.into_inner().users;

    for user in users {
        println!("UUID: {}, Name: {}, Email: {}", user.user_uuid, user.user_name, user.user_email);
    }


    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Парсинг аргументов командной строки
    let args = Args::parse();

    // Создание клиента gRPC
    let mut client = UserServiceClient::connect("http://127.0.0.1:8080").await?;

    match args.action {
        Actions::Put | Actions::Get | Actions::Update => {
            let user_uuid_str = args.user_uuid.as_deref().ok_or("user_uuid is required for this action")?;
            let user_uuid = Uuid::parse_str(user_uuid_str)?;

            match args.action {
                Actions::Put => {
                    put_user_data(&mut client, &user_uuid, &args.user_name, &args.user_email).await?;
                },
                Actions::Get => {
                    get_user_data(&mut client, &user_uuid).await?;
                },
                Actions::Update => {
                    update_user_data(&mut client, &user_uuid, Some(&args.user_name), Some(&args.user_email)).await?;
                },
                _ => unreachable!(),
            }
        },
        Actions::GetAll => {
            get_all_users(&mut client).await?;
        },
    }

    Ok(())
}
