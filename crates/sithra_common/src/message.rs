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
pub struct UserID(pub String);

impl<T: ToString> From<T> for UserID {
    fn from(value: T) -> Self {
        Self(value.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroupID(pub String);

impl <T: ToString> From<T> for GroupID {
    fn from(value: T) -> Self {
        Self(value.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MessageID(pub String);

impl <T: ToString> From<T> for MessageID {
    fn from(value: T) -> Self {
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
                data.id.unwrap_or(data.qq.ok_or("expected id or qq")?),
            )),
            InternalMessage::Poke(_data) => MessageNode::Poke,
            InternalMessage::Share(data) => MessageNode::Share(data.url),
            InternalMessage::Contact(data) => match data.contact_type {
                internal::ContactType::Group => {
                    MessageNode::Contact(ConversationContact::Group(GroupID(data.id)))
                }
                internal::ContactType::QQ => {
                    MessageNode::Contact(ConversationContact::Private(UserID(data.id)))
                }
            },
            InternalMessage::Location(data) => {
                MessageNode::Location(data.lat.parse().unwrap(), data.lon.parse().unwrap())
            }
            InternalMessage::Reply(data) => MessageNode::Reply(MessageID(data.id)),
            InternalMessage::Unknown(data) => MessageNode::Unknown(data),
        })
    }
}