

// pub async fn get_test_library() -> Library {
//     let lib = get_empty_library().await;

//     sqlx::query!("
//         PRAGMA foreign_keys=OFF;
//         BEGIN TRANSACTION;

//         INSERT INTO folders VALUES(0,'/tmp/','uuid');

//         INSERT INTO entries VALUES(1,0,'alephria_p1.png','alephria_p1.png','.png',NULL,NULL,NULL);

//         INSERT INTO text_fields VALUES('https://www.pixiv.net/en/artworks/125028544', 1, 'URL', 1, 0);

//         COMMIT;
//     ").execute(&mut *lib.db.get().await.unwrap()).await.unwrap();

//     lib
// }
