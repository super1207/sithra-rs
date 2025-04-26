use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::BotError;

/// 用户 ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct UserId(String);
impl UserId {
    pub fn new(id: String) -> Self {
        Self(id)
    }
}
impl ToString for UserId {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}
/// 消息 ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct MessageId(String);
impl MessageId {
    pub fn new(id: String) -> Self {
        Self(id)
    }
}
impl ToString for MessageId {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

/// 频道 ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Channel(String, ChannelType);
impl Channel {
    pub fn new(id: String, channel_type: ChannelType) -> Self {
        Self(id, channel_type)
    }
    pub fn channel_type(&self) -> &ChannelType {
        &self.1
    }
    pub fn id(&self) -> &String {
        &self.0
    }
}
impl ToString for Channel {
    fn to_string(&self) -> String {
        format!("{}#{}", self.1.to_string(), self.0)
    }
}

/// 频道类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum ChannelType {
    /// 私聊
    #[serde(rename = "private")]
    Private,
    /// 群聊
    #[serde(rename = "group")]
    Group,
}
impl ToString for ChannelType {
    fn to_string(&self) -> String {
        match self {
            Self::Private => "private".to_string(),
            Self::Group => "group".to_string(),
        }
    }
}
impl FromStr for ChannelType {
    type Err = BotError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "private" => Self::Private,
            "group" => Self::Group,
            _ => return Err(BotError::InvalidChannelType),
        })
    }
}
