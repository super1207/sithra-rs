mod internal;

pub mod message_internal {
    pub use super::internal::*;
}

use internal::*;
use serde::{Deserialize, Serialize};

pub struct SMessage {
    pub messages: Vec<MessageNode>,
    pub conversation: ConversationContact,
    pub id: MessageID,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserID(pub u64);

impl From<u64> for UserID {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<i64> for UserID {
    fn from(value: i64) -> Self {
        Self(value as u64)
    }
}

impl From<UserID> for u64 {
    fn from(value: UserID) -> Self {
        value.0
    }
}

impl From<UserID> for i64 {
    fn from(value: UserID) -> Self {
        value.0 as i64
    }
}

impl From<UserID> for String {
    fn from(value: UserID) -> Self {
        value.0.to_string()
    }
}

impl From<String> for UserID {
    fn from(value: String) -> Self {
        Self(value.parse().unwrap())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroupID(pub u64);

impl From<u64> for GroupID {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<i64> for GroupID {
    fn from(value: i64) -> Self {
        Self(value as u64)
    }
}

impl From<GroupID> for u64 {
    fn from(value: GroupID) -> Self {
        value.0
    }
}

impl From<GroupID> for i64 {
    fn from(value: GroupID) -> Self {
        value.0 as i64
    }
}

impl From<GroupID> for String {
    fn from(value: GroupID) -> Self {
        value.0.to_string()
    }
}

impl From<String> for GroupID {
    fn from(value: String) -> Self {
        Self(value.parse().unwrap())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MessageID(pub String);

impl ToString for MessageID {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl From<String> for MessageID {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&String> for MessageID {
    fn from(value: &String) -> Self {
        Self(value.clone())
    }
}

impl From<i32> for MessageID {
    fn from(value: i32) -> Self {
        Self(value.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MessageNode {
    /// 文本 (内容)
    Text(String),
    /// 图片 (URL)
    Image(String),
    /// 提及 (QQ)
    At(UserID),
    /// 戳一戳
    Poke,
    /// 分享 (URL)
    Share(String),
    /// 语音 (URL)
    Record(String),
    /// 推荐好友/群 (类型)
    Contact(ConversationContact),
    /// 位置 (纬度, 经度)
    Location(f64, f64),
    /// 回复 (消息ID)
    Reply(MessageID),
    /// 未知
    Unknown(UnknownMessage),
}

impl ToString for MessageNode {
    fn to_string(&self) -> String {
        match self {
            MessageNode::Text(text) => text.clone(),
            MessageNode::Image(url) => format!("![]({})", url),
            MessageNode::At(user_id) => format!("@{}", user_id.0),
            MessageNode::Poke => "戳一戳".to_string(),
            MessageNode::Share(url) => format!("[分享]({})", url),
            MessageNode::Record(url) => format!("[语音]({})", url),
            MessageNode::Contact(contact) => match contact {
                ConversationContact::Private(user_id) => format!("[推荐好友]({})", user_id.0),
                ConversationContact::Group(group_id) => format!("[推荐群]({})", group_id.0),
            },
            MessageNode::Location(lat, lon) => format!("[位置]({},{})", lat, lon),
            MessageNode::Reply(message_id) => format!("[引用]({})", message_id.to_string()),
            MessageNode::Unknown(unknown) => format!("{:?}", unknown),
        }
    }
}

pub trait MessageExt {
    fn to_string(&self) -> String;
    fn trim_start_matches(self, prefix: &str) -> Self;
    fn starts_with(&self, prefix: &str) -> bool;
}

impl MessageExt for Vec<MessageNode> {
    fn to_string(&self) -> String {
        self.iter()
            .map(|node| node.to_string())
            .collect::<Vec<String>>()
            .join(" ")
    }
    fn trim_start_matches(self, prefix: &str) -> Self {
        self.into_iter()
            .enumerate()
            .map(|(i, node)| {
                if i != 0 {
                    return node;
                }
                match node {
                    MessageNode::Text(text) => {
                        MessageNode::Text(text.trim_start_matches(prefix).to_string())
                    }
                    _ => node,
                }
            })
            .collect()
    }
    fn starts_with(&self, prefix: &str) -> bool {
        match self.first() {
            Some(MessageNode::Text(text)) => text.starts_with(prefix),
            _ => false,
        }
    }
}

pub mod command {
    use std::ops::Deref;

    use super::*;
    pub struct Text(pub String);
    impl Deref for Text {
        type Target = String;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    pub struct Image(pub String);
    pub struct At(pub UserID);
    impl Deref for At {
        type Target = UserID;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    pub struct Poke;
    pub struct Share(pub String);
    pub struct Record(pub String);
    pub struct Contact(pub ConversationContact);
    impl Deref for Contact {
        type Target = ConversationContact;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    pub struct Location(pub f64, pub f64);
    pub struct Reply(pub MessageID);
    impl Deref for Reply {
        type Target = MessageID;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConversationContact {
    Private(UserID),
    Group(GroupID),
}

impl TryFrom<InternalMessage> for MessageNode {
    type Error = &'static str;
    fn try_from(msg: InternalMessage) -> Result<MessageNode, &'static str> {
        Ok(match msg {
            InternalMessage::Text(data) => MessageNode::Text(data.text),
            InternalMessage::Image(data) => MessageNode::Image(data.file),
            InternalMessage::Record(data) => MessageNode::Record(data.file),
            InternalMessage::At(data) => MessageNode::At(UserID(
                data.id
                    .unwrap_or(data.qq.ok_or("expected id or qq")?)
                    .parse()
                    .map_err(|_| "id must be a number")?,
            )),
            InternalMessage::Poke(_data) => MessageNode::Poke,
            InternalMessage::Share(data) => MessageNode::Share(data.url),
            InternalMessage::Contact(data) => match data.contact_type {
                internal::ContactType::Group => MessageNode::Contact(ConversationContact::Group(
                    GroupID(data.id.parse().map_err(|_| "id must be a number")?),
                )),
                internal::ContactType::QQ => MessageNode::Contact(ConversationContact::Private(
                    UserID(data.id.parse().map_err(|_| "id must be a number")?),
                )),
            },
            InternalMessage::Location(data) => MessageNode::Location(
                data.lat.parse().map_err(|_| "lat must be a number")?,
                data.lon.parse().map_err(|_| "lon must be a number")?,
            ),
            InternalMessage::Reply(data) => MessageNode::Reply(MessageID(data.id)),
            InternalMessage::Unknown(data) => MessageNode::Unknown(data),
        })
    }
}
