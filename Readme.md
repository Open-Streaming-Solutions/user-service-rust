
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

cargo run -- -a update -i 0189a30a-60c7-7135-b683-7d7f3783d4b7 -e "mod1@test.ru"

cargo run -- -a update -i 0189a30a-60c7-7136-b98e-9c2d4f2734f1 -e "mod2@test.ru"

cargo run -- -a update -i 0189a30a-60c7-7137-b1a3-8a6a3d9076fa -e "mod3@test.ru"

cargo run -- -a update -i 0189a30a-60c7-7138-8a68-2d4c5b617a98 -e "mod4@test.ru"

cargo run -- -a update -i 0189a30a-60c7-7139-b187-2e7e3b297efb -e "mod5@test.ru"