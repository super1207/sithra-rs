use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct InternalOnebotEvent {
    pub time: u64,
    pub self_id: u64,
    #[serde(flatten)]
    pub kind: InternalOnebotEventKind,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "post_type")]
pub enum InternalOnebotEventKind {
    #[serde(rename = "message")]
    Message(MessageEvent),
    #[serde(rename = "notice")]
    Notice(NoticeEvent),
    #[serde(rename = "request")]
    Request(RequestEvent),
    #[serde(rename = "meta_event")]
    Meta(MetaEvent),
    #[serde(untagged)]
    Unknown(serde_json::Value),
}

/* 消息事件 */
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "message_type")]
pub enum MessageEvent {
    #[serde(rename = "private")]
    Private(PrivateMessage),
    #[serde(rename = "group")]
    Group(GroupMessage),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrivateMessage {
    pub sub_type: String,
    pub message_id: i32,
    pub user_id: u64,
    pub message: Vec<crate::message::internal::InternalMessage>,
    pub raw_message: String,
    pub font: i32,
    pub sender: PrivateSender,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrivateSender {
    pub user_id: u64,
    pub nickname: Option<String>,
    pub sex: Option<String>,
    pub age: Option<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GroupMessage {
    pub sub_type: String,
    pub message_id: i32,
    pub group_id: u64,
    pub user_id: u64,
    pub anonymous: Option<Anonymous>,
    pub message: Vec<crate::message::internal::InternalMessage>,
    pub raw_message: String,
    pub font: i32,
    pub sender: GroupSender,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GroupSender {
    pub user_id: u64,
    pub nickname: Option<String>,
    pub card: Option<String>,
    pub sex: Option<String>,
    pub age: Option<i32>,
    pub area: Option<String>,
    pub level: Option<String>,
    pub role: Option<String>,
    pub title: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Anonymous {
    pub id: u64,
    pub name: String,
    pub flag: String,
}

/* 通知事件 */
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "notice_type")]
pub enum NoticeEvent {
    #[serde(rename = "group_upload")]
    GroupUpload(GroupUploadNotice),
    #[serde(rename = "group_admin")]
    GroupAdmin(GroupAdminNotice),
    #[serde(rename = "group_decrease")]
    GroupDecrease(GroupDecreaseNotice),
    #[serde(rename = "group_increase")]
    GroupIncrease(GroupIncreaseNotice),
    #[serde(rename = "group_ban")]
    GroupBan(GroupBanNotice),
    #[serde(rename = "friend_add")]
    FriendAdd(FriendAddNotice),
    #[serde(rename = "group_recall")]
    GroupRecall(GroupRecallNotice),
    #[serde(rename = "friend_recall")]
    FriendRecall(FriendRecallNotice),
    #[serde(rename = "notify")]
    Notify(NotifyEvent),
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GroupUploadNotice {
    pub group_id: u64,
    pub user_id: u64,
    pub file: FileInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileInfo {
    pub id: String,
    pub name: String,
    pub size: u64,
    pub busid: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GroupAdminNotice {
    pub sub_type: String,
    pub group_id: u64,
    pub user_id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GroupDecreaseNotice {
    pub sub_type: String,
    pub group_id: u64,
    pub operator_id: u64,
    pub user_id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GroupIncreaseNotice {
    pub sub_type: String,
    pub group_id: u64,
    pub operator_id: u64,
    pub user_id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GroupBanNotice {
    pub sub_type: String,
    pub group_id: u64,
    pub operator_id: u64,
    pub user_id: u64,
    pub duration: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FriendAddNotice {
    pub user_id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GroupRecallNotice {
    pub group_id: u64,
    pub user_id: u64,
    pub operator_id: u64,
    pub message_id: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FriendRecallNotice {
    pub user_id: u64,
    pub message_id: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "sub_type")]
pub enum NotifyEvent {
    #[serde(rename = "poke")]
    Poke {
        group_id: u64,
        user_id: u64,
        target_id: u64,
    },
    #[serde(rename = "lucky_king")]
    LuckyKing {
        group_id: u64,
        user_id: u64,
        target_id: u64,
    },
    #[serde(rename = "honor")]
    Honor {
        group_id: u64,
        honor_type: String,
        user_id: u64,
    },
    #[serde(other)]
    Unknown,
}
/* 请求事件 */
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "request_type")]
pub enum RequestEvent {
    #[serde(rename = "friend")]
    Friend(FriendRequest),
    #[serde(rename = "group")]
    Group(GroupRequest),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FriendRequest {
    pub user_id: u64,
    pub comment: String,
    pub flag: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GroupRequest {
    pub sub_type: String,
    pub group_id: u64,
    pub user_id: u64,
    pub comment: String,
    pub flag: String,
}

/* 元事件 */
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "meta_event_type")]
pub enum MetaEvent {
    #[serde(rename = "lifecycle")]
    Lifecycle { sub_type: String },
    #[serde(rename = "heartbeat")]
    // TODO: 其他未知元事件类型
    #[serde(other)]
    Unknown,
}
