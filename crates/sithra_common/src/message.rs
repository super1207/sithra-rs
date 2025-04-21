mod internal;

pub mod message_internal {
    pub use super::internal::*;
}

use internal::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ForwardId(pub String);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserId(pub u64);

impl From<u64> for UserId {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<i64> for UserId {
    fn from(value: i64) -> Self {
        Self(value as u64)
    }
}

impl From<UserId> for u64 {
    fn from(value: UserId) -> Self {
        value.0
    }
}

impl From<UserId> for i64 {
    fn from(value: UserId) -> Self {
        value.0 as i64
    }
}

impl From<UserId> for String {
    fn from(value: UserId) -> Self {
        value.0.to_string()
    }
}

impl From<String> for UserId {
    fn from(value: String) -> Self {
        Self(value.parse().unwrap())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroupId(pub u64);

impl From<u64> for GroupId {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<i64> for GroupId {
    fn from(value: i64) -> Self {
        Self(value as u64)
    }
}

impl From<GroupId> for u64 {
    fn from(value: GroupId) -> Self {
        value.0
    }
}

impl From<GroupId> for i64 {
    fn from(value: GroupId) -> Self {
        value.0 as i64
    }
}

impl From<GroupId> for String {
    fn from(value: GroupId) -> Self {
        value.0.to_string()
    }
}

impl From<String> for GroupId {
    fn from(value: String) -> Self {
        Self(value.parse().unwrap())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MessageId(pub String);

impl ToString for MessageId {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl From<String> for MessageId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&String> for MessageId {
    fn from(value: &String) -> Self {
        Self(value.clone())
    }
}

impl From<i32> for MessageId {
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
    At(UserId),
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
    Reply(MessageId),
    /// 合并转发消息
    Forward(Forward),
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
            MessageNode::Reply(message_id) => format!("[引用]({})", message_id.0),
            MessageNode::Forward(forward_node) => format!("[合并转发节点]({})", forward_node.id.0),
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
    pub struct At(pub UserId);
    impl Deref for At {
        type Target = UserId;
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
    pub struct Reply(pub MessageId);
    impl Deref for Reply {
        type Target = MessageId;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForwardMessageNode {
    pub user_id: UserId,
    pub nickname: String,
    pub content: Vec<MessageNode>,
}

impl ForwardMessageNode {
    pub fn new(user_id: UserId, nickname: String, content: Vec<MessageNode>) -> Self {
        Self {
            user_id,
            nickname,
            content,
        }
    }
}

impl From<ForwardMessageNode> for InternalForwardMessage {
    fn from(value: ForwardMessageNode) -> Self {
        InternalForwardMessage::new(
            value.user_id.into(),
            value.nickname,
            value.content.into_iter().map(|node| node.into()).collect(),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConversationContact {
    Private(UserId),
    Group(GroupId),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Forward {
    pub id: ForwardId,
}

impl From<ForwardId> for Forward {
    fn from(value: ForwardId) -> Self {
        Self { id: value }
    }
}

impl TryFrom<InternalMessage> for MessageNode {
    type Error = &'static str;
    fn try_from(msg: InternalMessage) -> Result<MessageNode, &'static str> {
        Ok(match msg {
            InternalMessage::Text(data) => MessageNode::Text(data.text),
            InternalMessage::Image(data) => MessageNode::Image(data.file),
            InternalMessage::Record(data) => MessageNode::Record(data.file),
            InternalMessage::At(data) => MessageNode::At(UserId(
                data.id
                    .unwrap_or(data.qq.ok_or("expected id or qq")?)
                    .parse()
                    .map_err(|_| "id must be a number")?,
            )),
            InternalMessage::Poke(_data) => MessageNode::Poke,
            InternalMessage::Share(data) => MessageNode::Share(data.url),
            InternalMessage::Contact(data) => match data.contact_type {
                internal::ContactType::Group => MessageNode::Contact(ConversationContact::Group(
                    GroupId(data.id.parse().map_err(|_| "id must be a number")?),
                )),
                internal::ContactType::QQ => MessageNode::Contact(ConversationContact::Private(
                    UserId(data.id.parse().map_err(|_| "id must be a number")?),
                )),
            },
            InternalMessage::Location(data) => MessageNode::Location(
                data.lat.parse().map_err(|_| "lat must be a number")?,
                data.lon.parse().map_err(|_| "lon must be a number")?,
            ),
            InternalMessage::Reply(data) => MessageNode::Reply(MessageId(data.id)),
            InternalMessage::Forward(data) => MessageNode::Forward(Forward {
                id: ForwardId(data.id),
            }),
            InternalMessage::Unknown(data) => MessageNode::Unknown(data),
        })
    }
}
