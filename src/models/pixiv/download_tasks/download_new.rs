use futures::channel::mpsc::Receiver;
use futures::channel::mpsc::Sender;
use futures::stream;

/// A task that download new
pub struct PixivDownloadNewTask {
    sender: Sender<i64>,
    reciever: Receiver<i64>
}

// impl PixivDownloadNewTask {
//     pub fn create_stream() {
//         stream::empty::<i64>()
//             .;
//     }
// }