cargo sqlx prepare --check
if [ $? -ne 0 ] 
then
    cargo sqlx prepare
    git add ./.sqlx
fi