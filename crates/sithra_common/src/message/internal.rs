use super::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "lowercase")]
pub enum InternalMessage {
    Text(TextData),
    Image(MediaData),
    Record(MediaData),
    At(AtData),
    Poke(PokeData),
    Share(ShareData),
    Contact(ContactData),
    Location(LocationData),
    Reply(ReplyData),
    #[serde(untagged)]
    Unknown(UnknownMessage),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct UnknownMessage {
    #[serde(rename = "type")]
    pub r#type: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TextData {
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MediaData {
    pub file: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AtData {
    pub id: Option<String>,
    pub qq: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PokeData {
    #[serde(rename = "type")]
    pub poke_type: String,
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ShareData {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContactData {
    #[serde(rename = "type")]
    pub contact_type: ContactType,
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LocationData {
    pub lat: String,
    pub lon: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReplyData {
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ContactType {
    QQ,
    Group,
}

impl From<MessageNode> for InternalMessage {
    fn from(node: MessageNode) -> InternalMessage {
        match node {
            MessageNode::Text(text) => InternalMessage::Text(TextData { text }),
            MessageNode::Image(url) => InternalMessage::Image(MediaData { file: url }),
            MessageNode::Record(url) => InternalMessage::Record(MediaData { file: url }),
            MessageNode::At(qq) => InternalMessage::At(AtData {
                qq: Some(qq.0.to_string()),
                id: Some(qq.0.to_string()),
            }),
            MessageNode::Poke => InternalMessage::Poke(PokeData {
                poke_type: "poke".to_string(),
                id: "-1".to_string(),
            }),
            MessageNode::Share(url) => InternalMessage::Share(ShareData { url }),
            MessageNode::Contact(contact_type) => InternalMessage::Contact(match contact_type {
                ConversationContact::Group(id) => ContactData {
                    contact_type: ContactType::Group,
                    id: id.0.to_string(),
                },
                ConversationContact::Private(id) => ContactData {
                    contact_type: ContactType::QQ,
                    id: id.0.to_string(),
                },
            }),
            MessageNode::Location(lat, lon) => InternalMessage::Location(LocationData {
                lat: lat.to_string(),
                lon: lon.to_string(),
            }),
            MessageNode::Reply(id) => InternalMessage::Reply(ReplyData {
                id: id.0.to_string(),
            }),
            MessageNode::Unknown(data) => InternalMessage::Unknown(data),
        }
    }
}
