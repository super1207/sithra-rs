use std::ops::Deref;

use ioevent::Event;
use serde::{Deserialize, Deserializer, Serialize};

use crate::message::*;
use crate::model::*;

#[derive(Debug, Clone, Deserialize, Serialize, Event)]
pub struct MessageReceived<M: Message> {
    gid: GenericId,
    channel: Channel,
    user: User,
    #[serde(deserialize_with = "deserialize_message")]
    message: M,
}
impl<M: Message> MessageReceived<M> {
    /// 创建一个消息接收事件
    pub fn new(gid: Option<GenericId>, channel: Channel, user: User, message: M) -> Self {
        Self {
            gid: gid.unwrap_or_default(),
            channel,
            user,
            message,
        }
    }
    /// 获取聊天 ID
    pub fn channel(&self) -> &Channel {
        &self.channel
    }
    /// 获取用户
    pub fn user(&self) -> Option<&User> {
        if self.user.is_empty() {
            None
        } else {
            Some(&self.user)
        }
    }
    /// 获取用户(可能为空，必须自行判断)
    pub unsafe fn fetch_user(&self) -> &User {
        &self.user
    }
    /// 获取消息
    pub fn message(&self) -> &M {
        &self.message
    }
    /// 获取聊天 ID
    pub fn gid<T: EnsureGenericId>(&self) -> Result<T, T::Error> {
        T::ensure_generic_id(&self.gid)
    }
}
impl<M: Message> Deref for MessageReceived<M> {
    type Target = M;

    fn deref(&self) -> &Self::Target {
        &self.message
    }
}
pub fn deserialize_message<'de, D, M>(deserializer: D) -> Result<M, D::Error>
where
    D: Deserializer<'de>,
    M: Message,
{
    let raw = MessageRaw::deserialize(deserializer)?;
    Ok(M::from_raw(raw))
}
