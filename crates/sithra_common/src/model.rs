use std::{hash::Hash, str::FromStr};

use micromap::Map;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use crate::error::BotError;

/// 键值对
pub type KV = Map<String, String, 3>;
/// 短向量
pub type SVec<T> = SmallVec<[T; 3]>;

macro_rules! impl_id_traits {
    ($type:ty) => {
        impl ToString for $type {
            fn to_string(&self) -> String {
                self.0.clone()
            }
        }
        impl FromStr for $type {
            type Err = BotError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(Self::new(s))
            }
        }
        impl From<&str> for $type {
            fn from(s: &str) -> Self {
                Self::new(s)
            }
        }
        impl From<u64> for $type {
            fn from(s: u64) -> Self {
                Self::new(s.to_string())
            }
        }
        impl From<u32> for $type {
            fn from(s: u32) -> Self {
                Self::new(s.to_string())
            }
        }
        impl From<u16> for $type {
            fn from(s: u16) -> Self {
                Self::new(s.to_string())
            }
        }
        impl From<i64> for $type {
            fn from(s: i64) -> Self {
                Self::new(s.to_string())
            }
        }
        impl From<i32> for $type {
            fn from(s: i32) -> Self {
                Self::new(s.to_string())
            }
        }
        impl From<i16> for $type {
            fn from(s: i16) -> Self {
                Self::new(s.to_string())
            }
        }
    };
}

/// 用户 ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct UserId(String);
impl UserId {
    pub fn new(id: impl ToString) -> Self {
        Self(id.to_string())
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
impl_id_traits!(UserId);

/// 消息 ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct MessageId(String);
impl MessageId {
    pub fn new(id: impl ToString) -> Self {
        Self(id.to_string())
    }
}
impl_id_traits!(MessageId);

/// 频道 ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Channel(String, ChannelType);
impl Channel {
    pub fn new(id: impl ToString, channel_type: ChannelType) -> Self {
        Self(id.to_string(), channel_type)
    }
    pub fn channel_type(&self) -> &ChannelType {
        &self.1
    }
    pub fn id(&self) -> &String {
        &self.0
    }
    pub fn group_id(&self) -> Option<&String> {
        match self.1 {
            ChannelType::Group => Some(&self.0),
            _ => None,
        }
    }
    pub fn private_id(&self) -> Option<&String> {
        match self.1 {
            ChannelType::Private => Some(&self.0),
            _ => None,
        }
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
/// 用户模型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct User {
    pub uid: UserId,
    avatar: Option<String>,
    name: String,
    nick: Option<String>,
}
impl User {
    pub fn new(uid: impl Into<UserId>, name: impl ToString, nick: Option<String>, avatar: Option<String>) -> Self {
        Self {
            uid: uid.into(),
            name: name.to_string(),
            nick,
            avatar,
        }
    }
    pub fn builder(uid: UserId, name: String) -> UserBuilder {
        UserBuilder::new(uid, name)
    }
    /// 获取用户名
    pub fn call_name(&self) -> String {
        self.nick.clone().unwrap_or_else(|| self.name.clone())
    }
    /// 创建一个空用户(主要用于 QQ 官方机器人这类无法获取用户信息的机器人)
    pub fn empty() -> Self {
        Self {
            uid: UserId::new("".to_string()),
            name: "".to_string(),
            nick: None,
            avatar: None,
        }
    }
    /// 判断用户是否为空
    pub fn is_empty(&self) -> bool {
        self.uid.is_empty() && self.name.is_empty()
    }
}
/// 用户构建器
pub struct UserBuilder {
    id: UserId,
    name: String,
    nick: Option<String>,
    avatar: Option<String>,
}
impl UserBuilder {
    pub fn new(uid: impl Into<UserId>, name: String) -> Self {
        Self {
            id: uid.into(),
            name,
            nick: None,
            avatar: None,
        }
    }
    pub fn id(mut self, id: impl Into<UserId>) -> Self {
        self.id = id.into();
        self
    }
    pub fn name(mut self, name: impl ToString) -> Self {
        self.name = name.to_string();
        self
    }
    pub fn nick(mut self, nick: impl ToString) -> Self {
        self.nick = Some(nick.to_string());
        self
    }
    pub fn avatar(mut self, avatar: impl ToString) -> Self {
        self.avatar = Some(avatar.to_string());
        self
    }
    pub fn build(self) -> User {
        User::new(self.id, self.name, self.nick, self.avatar)
    }
}
/// 通用 ID
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct GenericId(KV);
impl GenericId {
    /// 创建一个通用 ID
    pub fn new(kv: KV) -> Self {
        Self(kv)
    }
    /// 获取一个键值
    pub fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(|v| v.as_str())
    }
    /// 获取一个键值，如果键值不存在，则返回默认值
    pub fn get_or_default<'a>(&'a self, key: &str, default: &'a str) -> &'a str {
        self.get(key).unwrap_or(default)
    }
    /// 创建一个空 ID
    pub fn empty() -> Self {
        Self(KV::new())
    }
}
impl Default for GenericId {
    fn default() -> Self {
        Self::empty()
    }
}
pub trait EnsureGenericId
where
    Self: Sized + Into<GenericId>,
{
    type Error: Default;
    fn ensure_generic_id(id: &GenericId) -> Result<Self, Self::Error>;
}
