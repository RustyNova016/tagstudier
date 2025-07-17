use futures::channel::mpsc::unbounded;
use futures::SinkExt;
use futures::StreamExt;
use futures::TryStream;
use futures::TryStreamExt;
use streamies::Streamies;

pub struct Tag {
    name: String
}

// impl Tag {
//     pub async  fn get_tag_with_parents(&self, conn: &mut sqlx::SqliteConnection) -> impl TryStream<Ok = Tag, Error = crate::Error> {
//         let (tag_sender, tag_channel) = unbounded::<Tag>();

//         tag_channel.unique_by(|tag| tag.name).map(async |tag| {
//             let stream = sqlx::query_as("SELECT * FROM TAG").fetch(conn);

//             for new_tag in stream.try_next().await {

//             }
//         })
//     }
// }