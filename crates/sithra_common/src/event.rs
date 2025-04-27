use std::ops::Deref;

use ioevent::Event;
use ioevent::rpc::ProcedureCall;
use serde::Deserializer;
use serde::{Deserialize, Serialize};

use crate::api::{DeleteMessage, SendMessage, SithraCall, SithraCallRequest};
use crate::message::*;
use crate::model::*;

pub trait SithraEvent {
    fn generic_id<'a>(&'a self) -> &'a GenericId;
    fn build_request<C>(&self, call: C) -> SithraCall<C>
    where
        C: ProcedureCall + SithraCallRequest,
    {
        SithraCall::new(self.generic_id().clone(), call)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Event)]
pub struct MessageEvent<M: Message> {
    generic_id: GenericId,
    channel: Channel,
    user: User,
    #[serde(deserialize_with = "deserialize_message")]
    #[serde(serialize_with = "serialize_message::<_, M>")]
    message: M,
}
impl<M: Message> SithraEvent for MessageEvent<M> {
    fn generic_id<'a>(&'a self) -> &'a GenericId {
        &self.generic_id
    }
}
impl<M: Message> MessageEvent<M> {
    /// 创建一个消息接收事件
    pub fn new<T: EnsureGenericId>(gid: T, channel: Channel, user: User, message: M) -> Self {
        Self {
            generic_id: gid.into(),
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
    pub fn get_generic_id<T: EnsureGenericId>(&self) -> Result<T, T::Error> {
        T::ensure_generic_id(&self.generic_id)
    }
    /// 构造回复请求体
    pub fn build_reply(&self, message: M) -> SithraCall<SendMessage> {
        self.build_request(SendMessage::new(message, self.channel().clone()))
    }
    /// 构造删除请求体(消息ID不为空时)
    pub fn build_delete(&self) -> Option<SithraCall<DeleteMessage>> {
        self.message()
            .id()
            .map(|id| self.build_request(DeleteMessage::new(id)))
    }
}
impl<M: Message> Deref for MessageEvent<M> {
    type Target = M;

    fn deref(&self) -> &Self::Target {
        &self.message
    }
}
/// 消息操作事件
#[derive(Debug, Clone, Deserialize, Serialize, Event)]
pub struct MessageActionEvent<M: Message> {
    generic_id: GenericId,
    channel: Channel,
    user: User,
    message_id: MessageId,
    #[serde(deserialize_with = "deserialize_message_action_type")]
    action: MessageActionType<M>,
}
impl<M: Message> SithraEvent for MessageActionEvent<M> {
    fn generic_id<'a>(&'a self) -> &'a GenericId {
        &self.generic_id
    }
}
impl<M: Message> MessageActionEvent<M> {
    /// 获取变动消息(变动后)
    pub fn message(&self) -> Option<&M> {
        self.action.message()
    }
    /// 获取变动消息ID(变动前)
    pub fn message_id(&self) -> &MessageId {
        &self.message_id
    }
    /// 获取操作者用户信息
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
    /// 获取聊天信息
    pub fn channel(&self) -> &Channel {
        &self.channel
    }
    /// 获取特殊 ID
    pub fn get_generic_id<T: EnsureGenericId>(&self) -> Result<T, T::Error> {
        T::ensure_generic_id(&self.generic_id)
    }
}
/// 消息操作类型
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum MessageActionType<M: Message> {
    /// 删除
    Delete,
    /// 编辑(新消息)
    Edit(
        #[serde(serialize_with = "serialize_message::<_, M>")]
        #[serde(deserialize_with = "deserialize_message")]
        M,
    ),
}
/// 反序列化消息操作类型
pub fn deserialize_message_action_type<'de, D, M>(
    deserializer: D,
) -> Result<MessageActionType<M>, D::Error>
where
    D: Deserializer<'de>,
    M: Message,
{
    let raw = MessageActionType::<M>::deserialize(deserializer)?;
    Ok(raw)
}
impl<M: Message> MessageActionType<M> {
    /// 获取变动消息(变动后)
    pub fn message(&self) -> Option<&M> {
        match self {
            Self::Edit(message) => Some(message),
            _ => None,
        }
    }
}
/// 群用户变动事件
#[derive(Debug, Clone, Deserialize, Serialize, Event)]
pub struct UserActionEvent {
    generic_id: GenericId,
    channel: Channel,
    user: User,
    action: UserActionType,
}
impl SithraEvent for UserActionEvent {
    fn generic_id<'a>(&'a self) -> &'a GenericId {
        &self.generic_id
    }
}
impl UserActionEvent {
    /// 获取操作者
    pub fn user(&self) -> Option<&User> {
        if self.user.is_empty() {
            None
        } else {
            Some(&self.user)
        }
    }
    /// 获取被操作者
    pub fn target(&self) -> Option<&User> {
        self.action.target()
    }
    /// 获取聊天信息
    pub fn channel(&self) -> &Channel {
        &self.channel
    }
    /// 获取特殊 ID
    pub fn get_generic_id<T: EnsureGenericId>(&self) -> Result<T, T::Error> {
        T::ensure_generic_id(&self.generic_id)
    }
}
/// 群用户变动类型
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum UserActionType {
    /// 加入群聊(用户)
    JoinGroup(User),
    /// 离开群聊(用户)
    LeaveGroup(User),
    /// 被禁止(用户)
    Ban(User),
    /// 被解除禁止(用户)
    Unban(User),
}
impl UserActionType {
    /// 获取目标用户
    pub fn target(&self) -> Option<&User> {
        match self {
            Self::JoinGroup(user) => Some(user),
            Self::LeaveGroup(user) => Some(user),
            Self::Ban(user) => Some(user),
            Self::Unban(user) => Some(user),
        }
    }
}
/// 聊天申请
#[derive(Debug, Clone, Deserialize, Serialize, Event)]
pub struct ChannelRequestEvent {
    generic_id: GenericId,
    channel: Channel,
}
impl SithraEvent for ChannelRequestEvent {
    fn generic_id<'a>(&'a self) -> &'a GenericId {
        &self.generic_id
    }
}
impl ChannelRequestEvent {
    /// 获取聊天信息
    pub fn channel(&self) -> &Channel {
        &self.channel
    }
    /// 获取特殊 ID
    pub fn get_generic_id<T: EnsureGenericId>(&self) -> Result<T, T::Error> {
        T::ensure_generic_id(&self.generic_id)
    }
}
