use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
struct RawEvent {
    time:    u64,
    self_id: u64,
    #[serde(flatten)]
    ty:      PostType,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", tag = "post_type")]
pub enum PostType {
    Message {
        #[serde(flatten)]
        kind: MessageEventKind,
    },
    Notice,
    Request,
    MetaEvent,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", tag = "message_type")]
pub enum MessageEventKind {
    Private,
    Group,
}
