use std::ops::Deref;

use ioevent::Event;
use serde::{Deserialize, Serialize};

use crate::message::*;
use crate::model::*;

#[derive(Debug, Clone, Deserialize, Serialize, Event)]
pub struct MessageReceived<M: Message> {
    generic_id: Option<GenericId>,
    channel: Channel,
    user: User,
    #[serde(deserialize_with = "deserialize_message")]
    message: M,
}
impl<M: Message> MessageReceived<M> {
    /// 创建一个消息接收事件
    pub fn new<T: EnsureGenericId>(
        gid: Option<T>,
        channel: Channel,
        user: User,
        message: M,
    ) -> Self {
        Self {
            generic_id: gid.map(Into::into),
            channel,
            user,
            message,
        }
    }
    /// 获取聊天信息
    pub fn channel(&self) -> &Channel {
        &self.channel
    }
    /// 获取用户信息
    pub fn user(&self) -> Option<&User> {
        if self.user.is_empty() {
            None
        } else {
            Some(&self.user)
        }
    }
    /// 获取用户(可能为空，必须自行判断)
    pub fn fetch_user(&self) -> &User {
        &self.user
    }
    /// 获取消息
    pub fn message(&self) -> &M {
        &self.message
    }
    /// 获取特殊 ID
    pub fn generic_id<T: EnsureGenericId>(&self) -> Result<T, T::Error> {
        T::ensure_generic_id(self.generic_id.as_ref().ok_or(T::Error::default())?)
    }
}
impl<M: Message> Deref for MessageReceived<M> {
    type Target = M;

    fn deref(&self) -> &Self::Target {
        &self.message
    }
}
