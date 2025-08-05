// #[cfg(test)]
// pub mod test {
//     use futures::executor::block_on;
//     use streamies::TryStreamies;
//     use tagstudio_db::models::entry::Entry;
//     use tracing_test::traced_test;

//     use crate::models::tag_providers::pixiv::PixivTagProvider;
//     use crate::models::tag_providers::pixiv::import_tags::PIXIV_IMAGE_DATA_IMPORT;
//     use crate::models::tag_providers::pixiv::import_tags::PIXIV_TAG_DATA_IMPORT;
//     use crate::tests::data::get_test_library;

//     #[tokio::test]
//     #[traced_test]
//     pub async fn import_pixiv_tags() {
//         let lib = get_test_library().await;

//         PixivTagProvider::load_data(
//             &mut *lib.db.get().await.unwrap(),
//             "https://www.pixiv.net/en/artworks/125028544",
//         )
//         .await;

//         let entry = Entry::find_by_id(&mut *lib.db.get().await.unwrap(), 1)
//             .await
//             .unwrap()
//             .unwrap();

//         let tags = entry
//             .get_tags(&mut *lib.db.get().await.unwrap())
//             .await
//             .unwrap();

//         assert!(tags.iter().any(|tag| tag.name == PIXIV_IMAGE_DATA_IMPORT));

//         assert!(tags.iter().any(|tag| {
//             if tag.name == "OC" {
//                 let conn = &mut *block_on(lib.db.get()).unwrap();
//                 let parents = block_on(tag.get_parents(conn).try_collect_vec()).unwrap();
//                 return parents.iter().any(|p| p.name == PIXIV_TAG_DATA_IMPORT);
//             }
//             false
//         }));
//     }
// }
