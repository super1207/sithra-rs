//! OneBot 标准事件模型
//!
//! 本模块定义了符合 OneBot 标准的事件结构，且进行了一定的抽象。

mod internal;

pub mod event_internal {
    pub use super::internal::*;
}

use std::ops::Deref;

use crate::api::{self, MessageIdResponse};
use crate::message::{ConversationContact, GroupID, MessageNode, UserID};
use crate::traits::*;
use internal::*;
use ioevent::Event;
use ioevent::rpc::*;
use serde::{Deserialize, Serialize};

pub use internal::NotifyEvent;

#[derive(Debug, Serialize, Deserialize, Event)]
pub struct OnebotEvent {
    /// 事件发生时间戳（UNIX 秒级时间戳）
    pub time: u64,

    /// 关联的机器人用户ID
    pub self_id: u64,

    /// 详细的事件类型分类
    pub kind: EventKind,
}
impl From<InternalOnebotEvent> for (OnebotEvent, Option<Vec<&'static str>>) {
    fn from(value: InternalOnebotEvent) -> Self {
        match value.kind {
            InternalOnebotEventKind::Message(message_event) => {
                let (message_detail, errors) = message_event.into();
                (
                    OnebotEvent {
                        time: value.time,
                        self_id: value.self_id,
                        kind: EventKind::Message(message_detail),
                    },
                    Some(errors),
                )
            }
            InternalOnebotEventKind::Notice(notice_event) => (
                OnebotEvent {
                    time: value.time,
                    self_id: value.self_id,
                    kind: EventKind::Notice(notice_event.into()),
                },
                None,
            ),
            InternalOnebotEventKind::Request(request_event) => (
                OnebotEvent {
                    time: value.time,
                    self_id: value.self_id,
                    kind: EventKind::Request(request_event.into()),
                },
                None,
            ),
            InternalOnebotEventKind::Meta(meta_event) => (
                OnebotEvent {
                    time: value.time,
                    self_id: value.self_id,
                    kind: EventKind::Meta(meta_event.into()),
                },
                None,
            ),
            InternalOnebotEventKind::Unknown(inner_value) => (
                OnebotEvent {
                    time: value.time,
                    self_id: value.self_id,
                    kind: EventKind::Unknown(inner_value),
                },
                None,
            ),
        }
    }
}

/// 所有可能的事件类型分类
#[derive(Debug, Serialize, Deserialize, Event)]
pub enum EventKind {
    /// 消息事件（私聊/群聊）
    Message(MessageEvent),

    /// 通知事件（群组变动/戳一戳等）
    Notice(NoticeEvent),

    /// 请求事件（好友/群组邀请）
    Request(RequestEvnet),

    /// 元事件（心跳/生命周期）
    Meta(MetaEvnet),

    /// 未知类型的备用变体
    Unknown(serde_json::Value),
}

/// 通用的消息事件基础结构
#[derive(Debug, Serialize, Deserialize)]
pub struct MessageCommon {
    /// 消息子类型分类
    pub sub_type: String,

    /// 唯一消息标识符
    pub message_id: i32,

    /// 消息发送者用户ID
    pub user_id: u64,

    /// 结构化的消息内容片段
    pub message: Vec<crate::message::MessageNode>,

    /// 未经处理的原始消息内容
    pub raw_message: String,

    /// 消息字体标识符
    pub font: i32,
}

/// 详细的消息类型分类
#[derive(Debug, Serialize, Deserialize, Event)]
pub enum MessageEvent {
    /// 私聊消息事件
    Private {
        /// 通用消息字段
        base: MessageCommon,

        /// 发送者
        sender: PrivateSender,
    },

    /// 群组消息事件
    Group {
        /// 通用消息字段
        base: MessageCommon,

        /// 消息来源群ID
        group_id: u64,

        /// 匿名发送者（如启用匿名时）
        anonymous: Option<Anonymous>,

        /// 发送者
        sender: GroupSender,
    },
}
impl From<internal::InternalMessageEvent> for (MessageEvent, Vec<&'static str>) {
    fn from(value: internal::InternalMessageEvent) -> Self {
        match value {
            InternalMessageEvent::Private(private_message) => {
                let mut errors = Vec::new();
                let internal::InternalPrivateMessage {
                    sub_type,
                    message_id,
                    user_id,
                    message,
                    raw_message,
                    font,
                    sender,
                } = private_message;
                let message = message
                    .into_iter()
                    .map(|x| x.try_into())
                    .collect_error(&mut errors)
                    .collect();
                let common = MessageCommon {
                    sub_type,
                    message_id,
                    user_id,
                    message,
                    raw_message,
                    font,
                };
                (
                    MessageEvent::Private {
                        base: common,
                        sender: sender.into(),
                    },
                    errors,
                )
            }
            InternalMessageEvent::Group(group_message) => {
                let mut errors = Vec::new();
                let internal::InternalGroupMessage {
                    sub_type,
                    message_id,
                    group_id,
                    user_id,
                    anonymous,
                    message,
                    raw_message,
                    font,
                    sender,
                } = group_message;
                let message = message
                    .into_iter()
                    .map(|x| x.try_into())
                    .collect_error(&mut errors)
                    .collect();
                let common = MessageCommon {
                    sub_type,
                    message_id,
                    user_id,
                    message,
                    raw_message,
                    font,
                };
                (
                    MessageEvent::Group {
                        base: common,
                        group_id,
                        anonymous: anonymous.map(|x| x.into()),
                        sender: sender.into(),
                    },
                    errors,
                )
            }
        }
    }
}

/// 消息事件的扁平化结构
#[derive(Debug, Serialize, Deserialize)]
pub struct MessageDetailFlatten {
    /// 消息内容
    pub message: Vec<MessageNode>,
    /// 消息来源
    pub contact: ConversationContact,
}
impl MessageDetailFlatten {
    pub async fn reply<T>(
        &self,
        state: &ioevent::State<T>,
        message: Vec<MessageNode>,
    ) -> Result<MessageIdResponse, crate::error::BotError>
    where
        T: ioevent::rpc::ProcedureCallWright + Clone,
    {
        match &self.contact {
            ConversationContact::Private(user_id) => Ok(state
                .call(&api::SendPrivateMsgParams::new(user_id.clone(), message)?)
                .await??),
            ConversationContact::Group(group_id) => Ok(state
                .call(&api::SendGroupMsgParams::new(group_id.clone(), message)?)
                .await??),
        }
    }
}
impl Deref for MessageDetailFlatten {
    type Target = Vec<MessageNode>;

    fn deref(&self) -> &Self::Target {
        &self.message
    }
}
impl MessageEvent {
    pub fn flatten(self) -> MessageDetailFlatten {
        match self {
            MessageEvent::Private {
                ref sender, base, ..
            } => MessageDetailFlatten {
                message: base.message,
                contact: ConversationContact::Private(UserID(sender.user_id.to_string())),
            },
            MessageEvent::Group { base, group_id, .. } => MessageDetailFlatten {
                message: base.message,
                contact: ConversationContact::Group(GroupID(group_id.to_string())),
            },
        }
    }
}

impl MessageEvent {
    pub fn downcast(self) -> (Vec<MessageNode>, ConversationContact) {
        match self {
            MessageEvent::Private {
                ref sender, base, ..
            } => (
                base.message,
                ConversationContact::Private(UserID(sender.user_id.to_string())),
            ),
            MessageEvent::Group {
                ref sender, base, ..
            } => (
                base.message,
                ConversationContact::Group(GroupID(sender.user_id.to_string())),
            ),
        }
    }
}

/// 群组通知事件详细类型
#[derive(Debug, Serialize, Deserialize, Event)]
pub enum NoticeEvent {
    /// 群文件上传通知
    GroupUpload {
        /// 群ID
        group_id: u64,
        /// 上传者ID
        user_id: u64,
        /// 文件详细信息
        file: FileInfo,
    },

    /// 群管理员变动通知
    GroupAdmin {
        /// 群ID
        group_id: u64,
        /// 操作者ID
        user_id: u64,
        /// 变动子类型（set/unset）
        sub_type: String,
    },

    /// 群成员变动通知（退群/踢人）
    GroupChange {
        /// 群组ID
        group_id: u64,
        /// 操作者ID（可能是机器人自身）
        _id: u64,
        /// 被操作用户ID
        user_id: u64,
        /// 变动子类型
        sub_type: GroupChangeType,
    },
    /// 好友添加请求
    FriendAdd {
        /// 请求者ID
        user_id: u64,
    },
    Notify(internal::NotifyEvent),
    /// 群聊撤回消息
    GroupRecall {
        /// 群ID
        group_id: u64,
        /// 操作者ID
        user_id: u64,
    },
    /// 私聊撤回消息
    PrivateRecall {
        /// 操作者ID
        user_id: u64,
    },
    /// 不常用
    Other(internal::InternalNoticeEvent),
}

/// 群成员变动子类型
#[derive(Debug, Serialize, Deserialize)]
pub enum GroupChangeType {
    /// 进入
    Approve,
    /// 被邀请
    Invite,
    /// 离开
    Leave,
    /// 踢出
    Kick,
    /// 嘻嘻, 被踢了
    KickMe,
    /// 未知
    Unknown,
}
impl From<String> for GroupChangeType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "approve" => GroupChangeType::Approve,
            "invite" => GroupChangeType::Invite,
            "leave" => GroupChangeType::Leave,
            "kick" => GroupChangeType::Kick,
            "kick_me" => GroupChangeType::KickMe,
            _ => GroupChangeType::Unknown,
        }
    }
}

/// 请求事件详细类型
#[derive(Debug, Serialize, Deserialize, Event)]
pub enum RequestEvnet {
    /// 好友添加请求
    Friend {
        /// 请求者用户ID
        user_id: u64,
        /// 验证信息
        comment: String,
        /// 请求标识（用于处理请求）
        flag: String,
    },

    /// 群组加入请求
    Group {
        /// 目标群组ID
        group_id: u64,
        /// 申请者用户ID
        user_id: u64,
        /// 验证信息
        comment: String,
        /// 请求标识（用于处理请求）
        flag: String,
        /// 请求子类型（add/invite等）
        sub_type: String,
    },
}

/// 元事件详细类型
#[derive(Debug, Serialize, Deserialize, Event)]
pub enum MetaEvnet {
    /// 机器人生命周期事件
    Lifecycle {
        /// 生命周期子类型（enable/disable等）
        sub_type: LifecycleType,
    },

    /* /// 心跳状态事件
    Heartbeat {
        /// 心跳间隔（毫秒）
        interval: u64,
    }, */
    /// 未知元事件类型
    Unknown,
}

/// 机器人生命周期子类型
#[derive(Debug, Serialize, Deserialize)]
pub enum LifecycleType {
    /// 启用
    Enable,
    /// 禁用
    Disable,
    /// 连接
    Connect,
    /// 未知
    Unknown,
}

impl From<String> for LifecycleType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "enable" => LifecycleType::Enable,
            "disable" => LifecycleType::Disable,
            "connect" => LifecycleType::Connect,
            _ => LifecycleType::Unknown,
        }
    }
}

/// 性别
#[derive(Debug, Serialize, Deserialize)]
pub enum Sex {
    /// 自我认知男性
    Male,
    /// 自我认知女性
    Female,
    /// 其他多元化性别
    Unknown,
}

impl From<Option<String>> for Sex {
    fn from(value: Option<String>) -> Self {
        match value {
            Some(value) => match value.as_str() {
                "male" => Sex::Male,
                "female" => Sex::Female,
                _ => Sex::Unknown,
            },
            None => Sex::Unknown,
        }
    }
}

/// 以下为复用结构定义，保持与原始设计一致并进行必要简化 ///
#[derive(Debug, Serialize, Deserialize)]
pub struct PrivateSender {
    /// 用户ID
    pub user_id: u64,
    /// 用户昵称
    pub nickname: Option<String>,
    /// 性别
    pub sex: Sex,
    /// 年龄
    pub age: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GroupSender {
    /// 用户ID
    pub user_id: u64,
    /// 群昵称
    pub nickname: Option<String>,
    /// 群名片
    pub card: Option<String>,
    /// 性别
    pub sex: Sex,
    /// 年龄
    pub age: Option<i32>,
    /// 群角色
    pub role: Role,
}

/// 用户于群聊中的性别
#[derive(Debug, Serialize, Deserialize)]
pub enum Role {
    Owner,
    Admin,
    Member,
    Unknown,
}
impl From<Option<String>> for Role {
    fn from(value: Option<String>) -> Self {
        match value {
            Some(value) => match value.as_str() {
                "owner" => Role::Owner,
                "admin" => Role::Admin,
                "member" => Role::Member,
                _ => Role::Unknown,
            },
            None => Role::Unknown,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Anonymous {
    /// 匿名用户唯一标识
    pub id: u64,
    /// 匿名名称
    pub name: String,
    /// 匿名标识符（用于反匿名化）
    pub flag: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileInfo {
    /// 文件唯一ID
    pub id: String,
    /// 文件名
    pub name: String,
    /// 文件大小（字节）
    pub size: u64,
    /// 文件业务ID
    pub busid: u64,
}

impl From<internal::InternalPrivateSender> for PrivateSender {
    fn from(value: internal::InternalPrivateSender) -> Self {
        PrivateSender {
            user_id: value.user_id,
            nickname: value.nickname,
            sex: value.sex.into(),
            age: value.age,
        }
    }
}

impl From<internal::InternalAnonymous> for Anonymous {
    fn from(value: internal::InternalAnonymous) -> Self {
        Anonymous {
            id: value.id,
            name: value.name,
            flag: value.flag,
        }
    }
}

impl From<internal::InternalGroupSender> for GroupSender {
    fn from(value: internal::InternalGroupSender) -> Self {
        GroupSender {
            user_id: value.user_id,
            nickname: value.nickname,
            card: value.card,
            sex: value.sex.into(),
            age: value.age,
            role: value.role.into(),
        }
    }
}

impl From<InternalNoticeEvent> for NoticeEvent {
    fn from(value: InternalNoticeEvent) -> Self {
        match value {
            InternalNoticeEvent::GroupUpload(n) => n.into(),
            InternalNoticeEvent::GroupAdmin(n) => n.into(),
            InternalNoticeEvent::GroupDecrease(n) => n.into(),
            InternalNoticeEvent::GroupIncrease(n) => n.into(),
            InternalNoticeEvent::GroupBan(n) => n.into(),
            InternalNoticeEvent::FriendAdd(n) => n.into(),
            InternalNoticeEvent::GroupRecall(n) => n.into(),
            InternalNoticeEvent::FriendRecall(n) => n.into(),
            InternalNoticeEvent::Notify(n) => NoticeEvent::Notify(n),
            _ => NoticeEvent::Other(value),
        }
    }
}

impl From<InternalGroupUploadNotice> for NoticeEvent {
    fn from(value: InternalGroupUploadNotice) -> Self {
        NoticeEvent::GroupUpload {
            group_id: value.group_id,
            user_id: value.user_id,
            file: FileInfo {
                id: value.file.id,
                name: value.file.name,
                size: value.file.size,
                busid: value.file.busid,
            },
        }
    }
}

impl From<InternalGroupAdminNotice> for NoticeEvent {
    fn from(value: InternalGroupAdminNotice) -> Self {
        NoticeEvent::GroupAdmin {
            group_id: value.group_id,
            user_id: value.user_id,
            sub_type: value.sub_type,
        }
    }
}

impl From<InternalGroupDecreaseNotice> for NoticeEvent {
    fn from(value: InternalGroupDecreaseNotice) -> Self {
        NoticeEvent::GroupChange {
            group_id: value.group_id,
            user_id: value.user_id,
            _id: value.operator_id,
            sub_type: value.sub_type.into(),
        }
    }
}

impl From<InternalGroupIncreaseNotice> for NoticeEvent {
    fn from(value: InternalGroupIncreaseNotice) -> Self {
        NoticeEvent::GroupChange {
            group_id: value.group_id,
            user_id: value.user_id,
            _id: value.operator_id,
            sub_type: value.sub_type.into(),
        }
    }
}

impl From<InternalGroupBanNotice> for NoticeEvent {
    fn from(value: InternalGroupBanNotice) -> Self {
        NoticeEvent::GroupChange {
            group_id: value.group_id,
            user_id: value.user_id,
            _id: value.operator_id,
            sub_type: value.sub_type.into(),
        }
    }
}

impl From<InternalFriendAddNotice> for NoticeEvent {
    fn from(value: InternalFriendAddNotice) -> Self {
        NoticeEvent::FriendAdd {
            user_id: value.user_id,
        }
    }
}

impl From<InternalGroupRecallNotice> for NoticeEvent {
    fn from(value: InternalGroupRecallNotice) -> Self {
        NoticeEvent::GroupRecall {
            group_id: value.group_id,
            user_id: value.user_id,
        }
    }
}

impl From<InternalFriendRecallNotice> for NoticeEvent {
    fn from(value: InternalFriendRecallNotice) -> Self {
        NoticeEvent::PrivateRecall {
            user_id: value.user_id,
        }
    }
}

impl From<InternalRequestEvent> for RequestEvnet {
    fn from(value: InternalRequestEvent) -> Self {
        match value {
            InternalRequestEvent::Friend(f) => f.into(),
            InternalRequestEvent::Group(g) => g.into(),
        }
    }
}

impl From<InternalFriendRequest> for RequestEvnet {
    fn from(value: InternalFriendRequest) -> Self {
        RequestEvnet::Friend {
            user_id: value.user_id,
            comment: value.comment,
            flag: value.flag,
        }
    }
}

impl From<InternalGroupRequest> for RequestEvnet {
    fn from(value: InternalGroupRequest) -> Self {
        RequestEvnet::Group {
            group_id: value.group_id,
            user_id: value.user_id,
            comment: value.comment,
            flag: value.flag,
            sub_type: value.sub_type,
        }
    }
}

impl From<InternalMetaEvent> for MetaEvnet {
    fn from(value: InternalMetaEvent) -> Self {
        match value {
            InternalMetaEvent::Lifecycle { sub_type } => MetaEvnet::Lifecycle {
                sub_type: sub_type.into(),
            },
            _ => MetaEvnet::Unknown,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct CommonFields {
    time: u64,
    self_id: u64,
}
