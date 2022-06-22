use deadqueue::unlimited::Queue;
use fy_base::api::upload_api::{NotifyCarQueueItem, NotifyFaceQueueItem};

//-----------------------
pub type FaceQueue = Queue<NotifyFaceQueueItem>;
pub type CarQueue = Queue<NotifyCarQueueItem>;
