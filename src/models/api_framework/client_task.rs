use futures::channel::mpsc::UnboundedReceiver;
use futures::channel::mpsc::UnboundedSender;

use crate::models::api_framework::api_request::ApiRequest;

pub struct ClientTask {
    reciever: UnboundedReceiver<ApiRequest>,
    sender: UnboundedSender<ApiRequest>,
    //queue: SortedVec<ApiRequest>
}
