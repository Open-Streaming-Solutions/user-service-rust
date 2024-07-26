
Для ручного клиента:

cargo run -- -a put -i 0189a30a-60c7-7135-b683-7d7f3783d4b7 -n test1 -e test@test.ru

cargo run -- -a put -i 0189a30a-60c7-7136-b98e-9c2d4f2734f1 -n test2 -e test@test.ru

cargo run -- -a get-user-data-by-id -i "0189a30a-60c7-7135-b683-7d7f3783d4b7"

cargo run -- -a get-user-data-by-id -i "0189a30a-60c7-7136-b98e-9c2d4f2734f1"

cargo run -- -a get-user-id-by-nickname -n test1

cargo run -- -a get-user-id-by-nickname -n test2

cargo run -- -a update -i 0189a30a-60c7-7135-b683-7d7f3783d4b7 -e "mod1@test.ru"

cargo run -- -a update -i 0189a30a-60c7-7136-b98e-9c2d4f2734f1 -e "mod2@test.ru"

cargo run -- -a get-all



Для grpcurl (В powershell в теле json должны быть экранированы ""  \"Нечто\"):

grpcurl -plaintext -import-path ../rpc -proto user-service.proto -d '{"user_uuid": "0189a30a-60c7-7135-b683-7d7f3783d4b7", "user_name": "test1", "user_email": "test@test.ru"}' localhost:8080 rpc.UserService/PutUserData

grpcurl -plaintext -import-path ../rpc -proto user-service.proto -d '{"user_uuid": "0189a30a-60c7-7136-b98e-9c2d4f2734f1", "user_name": "test2", "user_email": "test@test.ru"}' localhost:8080 rpc.UserService/PutUserData


grpcurl -plaintext -import-path ../rpc -proto user-service.proto -d '{"user_uuid": "0189a30a-60c7-7135-b683-7d7f3783d4b7"}' localhost:8080 rpc.UserService/GetUserDataById

grpcurl -plaintext -import-path ../rpc -proto user-service.proto -d '{"user_uuid": "0189a30a-60c7-7136-b98e-9c2d4f2734f1"}' localhost:8080 rpc.UserService/GetUserDataById


grpcurl -plaintext -import-path ../rpc -proto user-service.proto -d '{"user_name": "test1"}' localhost:8080 rpc.UserService/GetUserIdByNickname

grpcurl -plaintext -import-path ../rpc -proto user-service.proto -d '{"user_name": "test2"}' localhost:8080 rpc.UserService/GetUserIdByNickname


grpcurl -plaintext -import-path ../rpc -proto user-service.proto -d '{"user_uuid": "0189a30a-60c7-7135-b683-7d7f3783d4b7", "user_email": "mod1@test.ru"}' localhost:8080 rpc.UserService/UpdateUserData

grpcurl -plaintext -import-path ../rpc -proto user-service.proto -d '{"user_uuid": "0189a30a-60c7-7136-b98e-9c2d4f2734f1", "user_email": "mod2@test.ru"}' localhost:8080 rpc.UserService/UpdateUserData


grpcurl -plaintext -import-path ../rpc -proto user-service.proto -d '{}' localhost:8080 rpc.UserService/GetAllUsers
