pub mod response {
    use ioevent::rpc::*;
    use serde::{Deserialize, Serialize};

    use crate::{
        event::{Role, Sex},
        message::{GroupID, MessageID, MessageNode, UserID},
        traits::*,
    };

    /// 消息发送响应数据
    #[derive(Debug, Serialize, Deserialize, ProcedureCall)]
    pub struct MessageIdResponse {
        /// 消息ID（用于撤回等功能）
        pub message_id: MessageID,
    }
    impl ProcedureCallResponse for MessageIdResponse {}

    #[derive(Debug, Serialize, Deserialize, ProcedureCall)]
    /// 消息详情响应数据
    pub struct MessageDetail {
        /// 消息发送时间戳
        pub time: i64,
        /// 消息类型
        pub message_type: MessageType,
        /// 消息ID
        pub message_id: i32,
        /// 消息真实ID
        pub real_id: i32,
        /// 发送者信息
        pub sender: SenderInfo,
        /// 消息内容（已解析的消息段）
        pub message: Vec<MessageNode>,
    }
    impl ProcedureCallResponse for MessageDetail {}
    /// 消息类型
    #[derive(Debug, Serialize, Deserialize)]
    pub enum MessageType {
        /// 私聊
        Private,
        /// 群聊
        Group,
        /// 未知
        Unknown,
    }

    impl From<String> for MessageType {
        fn from(value: String) -> Self {
            match value.as_str() {
                "private" => MessageType::Private,
                "group" => MessageType::Group,
                _ => MessageType::Unknown,
            }
        }
    }

    /// 发送者信息结构
    #[derive(Debug, Serialize, Deserialize)]
    pub struct SenderInfo {
        /// 用户QQ号
        pub user_id: i64,
        /// 昵称
        pub nickname: String,
        /// 群角色（仅群消息有效）
        pub role: Role,
        /// 群名片（仅群消息有效）
        pub card: Option<String>,
    }

    /// 登录信息响应数据
    #[derive(Debug, Serialize, Deserialize)]
    pub struct LoginInfo {
        /// 当前登录的QQ号
        pub user_id: UserID,
        /// 当前登录的昵称
        pub nickname: String,
    }

    /// 陌生人信息响应数据
    #[derive(Debug, Serialize, Deserialize, ProcedureCall)]
    pub struct StrangerInfo {
        /// QQ号
        pub user_id: i64,
        /// 昵称
        pub nickname: String,
        /// 性别
        pub sex: Sex,
        /// 年龄
        pub age: i32,
        /// 地区信息
        pub area: Option<String>,
    }
    impl ProcedureCallResponse for StrangerInfo {}

    /// 群信息响应数据
    #[derive(Debug, Serialize, Deserialize, ProcedureCall)]
    pub struct GroupInfo {
        /// 群号
        pub group_id: GroupID,
        /// 群名称
        pub group_name: String,
        /// 当前成员数量
        pub member_count: Option<i32>,
        /// 最大成员数
        pub max_member_count: Option<i32>,
    }
    impl ProcedureCallResponse for GroupInfo {}

    /// 群成员信息响应数据
    #[derive(Debug, Serialize, Deserialize, ProcedureCall)]
    pub struct GroupMemberInfo {
        /// 群号
        pub group_id: GroupID,
        /// 用户QQ号
        pub user_id: UserID,
        /// 用户昵称
        pub nickname: String,
        /// 群名片
        pub card: Option<String>,
        /// 性别
        pub sex: Option<String>,
        /// 年龄
        pub age: Option<i32>,
        /// 地区
        pub area: Option<String>,
        /// 加群时间戳
        pub join_time: i64,
        /// 最后发言时间
        pub last_sent_time: i64,
        /// 成员等级
        pub level: Option<String>,
        /// 角色
        pub role: Role,
        /// 专属头衔
        pub title: Option<String>,
    }
    impl ProcedureCallResponse for GroupMemberInfo {}

    /// 群成员列表响应数据（JSON数组包装）
    #[derive(Debug, Serialize, Deserialize, ProcedureCall)]
    pub struct GroupMemberList(pub Vec<GroupMemberInfo>);
    impl ProcedureCallResponse for GroupMemberList {}

    /// 运行状态响应
    #[derive(Debug, Serialize, Deserialize, ProcedureCall)]
    pub struct StatusInfo {
        /// 是否在线（null表示未知）
        pub online: Option<bool>,
        /// 状态是否正常
        pub good: bool,
    }
    impl ProcedureCallResponse for StatusInfo {}
    /// 版本信息响应
    #[derive(Debug, Serialize, Deserialize, ProcedureCall)]
    pub struct VersionInfo {
        /// 实现名称（如go-cqhttp）
        pub app_name: String,
        /// 实现版本
        pub app_version: String,
        /// 协议版本（如v11）
        pub protocol_version: String,
    }
    impl ProcedureCallResponse for VersionInfo {}
    impl From<api_internal::MessageIdResponse> for MessageIdResponse {
        fn from(i: api_internal::MessageIdResponse) -> Self {
            Self {
                message_id: i.message_id.into(),
            }
        }
    }

    impl From<api_internal::SenderInfo> for SenderInfo {
        fn from(i: api_internal::SenderInfo) -> Self {
            Self {
                user_id: i.user_id,
                nickname: i.nickname,
                role: i.role.into(),
                card: i.card,
            }
        }
    }

    impl From<api_internal::LoginInfo> for LoginInfo {
        fn from(i: api_internal::LoginInfo) -> Self {
            Self {
                user_id: i.user_id.into(),
                nickname: i.nickname,
            }
        }
    }

    impl From<api_internal::StrangerInfo> for StrangerInfo {
        fn from(i: api_internal::StrangerInfo) -> Self {
            Self {
                user_id: i.user_id,
                nickname: i.nickname,
                sex: i.sex.into(),
                age: i.age,
                area: i.area,
            }
        }
    }

    impl From<api_internal::GroupInfo> for GroupInfo {
        fn from(i: api_internal::GroupInfo) -> Self {
            Self {
                group_id: i.group_id.into(),
                group_name: i.group_name,
                member_count: i.member_count,
                max_member_count: i.max_member_count,
            }
        }
    }

    impl From<api_internal::GroupMemberInfo> for GroupMemberInfo {
        fn from(i: api_internal::GroupMemberInfo) -> Self {
            Self {
                group_id: i.group_id.into(),
                user_id: i.user_id.into(),
                nickname: i.nickname,
                card: i.card,
                sex: i.sex.map(|x| x.into()),
                age: i.age,
                area: i.area,
                join_time: i.join_time,
                last_sent_time: i.last_sent_time,
                level: i.level,
                role: i.role.into(),
                title: i.title,
            }
        }
    }

    impl From<Vec<api_internal::GroupMemberInfo>> for GroupMemberList {
        fn from(i: Vec<api_internal::GroupMemberInfo>) -> Self {
            Self(i.into_iter().map(|x| x.into()).collect())
        }
    }

    impl From<api_internal::StatusInfo> for StatusInfo {
        fn from(i: api_internal::StatusInfo) -> Self {
            Self {
                online: i.online,
                good: i.good,
            }
        }
    }

    impl From<api_internal::VersionInfo> for VersionInfo {
        fn from(i: api_internal::VersionInfo) -> Self {
            Self {
                app_name: i.app_name,
                app_version: i.app_version,
                protocol_version: i.protocol_version,
            }
        }
    }

    impl From<api_internal::MessageDetail> for (MessageDetail, Vec<&'static str>) {
        fn from(i: api_internal::MessageDetail) -> Self {
            let mut errors = Vec::new();
            let message = i
                .message
                .into_iter()
                .map(|x| x.try_into())
                .collect_error(&mut errors)
                .collect();
            (
                MessageDetail {
                    time: i.time,
                    message_type: i.message_type.into(),
                    message_id: i.message_id,
                    real_id: i.real_id,
                    sender: i.sender.into(),
                    message,
                },
                errors,
            )
        }
    }

    impl From<api_internal::GroupMemberList> for GroupMemberList {
        fn from(i: api_internal::GroupMemberList) -> Self {
            Self(i.0.into_iter().map(|x| x.into()).collect())
        }
    }

    pub mod api_internal {
        use crate::message::message_internal::InternalMessage;
        use serde::{Deserialize, Serialize};

        /// API响应基础结构
        #[derive(Debug, Serialize, Deserialize)]
        pub struct ApiResponse {
            /// 响应状态
            pub status: Option<String>,
            /// 返回码
            pub retcode: Option<i32>,
            /// 响应数据
            pub data: Option<ApiResponseKind>,
            /// 与请求对应的echo标识
            pub echo: String,
        }

        #[derive(Debug, Serialize, Deserialize)]
        pub struct OnlyEcho {
            pub echo: String,
        }

        #[derive(Debug, Serialize, Deserialize)]
        #[serde(untagged)]
        pub enum ApiResponseKind {
            MessageIdResponse(MessageIdResponse),
            MessageDetail(MessageDetail),
            StrangerInfo(StrangerInfo),
            GroupInfo(GroupInfo),
            GroupMemberList(GroupMemberList),
            StatusInfo(StatusInfo),
            VersionInfo(VersionInfo),
            LoginInfo(LoginInfo),
            GroupMemberInfo(GroupMemberInfo),
            Unknown(serde_json::Value),
        }

        /// 消息发送响应数据
        #[derive(Debug, Serialize, Deserialize)]
        pub struct MessageIdResponse {
            /// 消息ID（用于撤回等功能）
            pub message_id: i32,
        }

        /// 消息详情响应数据
        #[derive(Debug, Serialize, Deserialize)]
        pub struct MessageDetail {
            /// 消息发送时间戳
            pub time: i64,
            /// 消息类型（private/group）
            pub message_type: String,
            /// 消息ID
            pub message_id: i32,
            /// 消息真实ID
            pub real_id: i32,
            /// 发送者信息
            pub sender: SenderInfo,
            /// 消息内容（已解析的消息段）
            pub message: Vec<InternalMessage>,
        }

        /// 发送者信息结构
        #[derive(Debug, Serialize, Deserialize)]
        pub struct SenderInfo {
            /// 用户QQ号
            pub user_id: i64,
            /// 昵称
            pub nickname: String,
            /// 群角色（仅群消息有效）
            #[serde(skip_serializing_if = "Option::is_none")]
            pub role: Option<String>,
            /// 群名片（仅群消息有效）
            #[serde(skip_serializing_if = "Option::is_none")]
            pub card: Option<String>,
        }

        /// 登录信息响应数据
        #[derive(Debug, Serialize, Deserialize)]
        pub struct LoginInfo {
            /// 当前登录的QQ号
            pub user_id: i64,
            /// 当前登录的昵称
            pub nickname: String,
        }

        /// 陌生人信息响应数据
        #[derive(Debug, Serialize, Deserialize)]
        pub struct StrangerInfo {
            /// QQ号
            pub user_id: i64,
            /// 昵称
            pub nickname: String,
            /// 性别（male/female/unknown）
            pub sex: Option<String>,
            /// 年龄
            pub age: i32,
            /// 地区信息
            #[serde(skip_serializing_if = "Option::is_none")]
            pub area: Option<String>,
        }

        /// 群信息响应数据
        #[derive(Debug, Serialize, Deserialize)]
        pub struct GroupInfo {
            /// 群号
            pub group_id: i64,
            /// 群名称
            pub group_name: String,
            /// 当前成员数量
            pub member_count: Option<i32>,
            /// 最大成员数
            pub max_member_count: Option<i32>,
        }

        /// 群成员信息响应数据
        #[derive(Debug, Serialize, Deserialize)]
        pub struct GroupMemberInfo {
            /// 群号
            pub group_id: i64,
            /// 用户QQ号
            pub user_id: i64,
            /// 用户昵称
            pub nickname: String,
            /// 群名片
            pub card: Option<String>,
            /// 性别
            pub sex: Option<String>,
            /// 年龄
            pub age: Option<i32>,
            /// 地区
            pub area: Option<String>,
            /// 加群时间戳
            pub join_time: i64,
            /// 最后发言时间
            pub last_sent_time: i64,
            /// 成员等级
            pub level: Option<String>,
            /// 角色（owner/admin/member）
            pub role: Option<String>,
            /// 专属头衔
            #[serde(skip_serializing_if = "Option::is_none")]
            pub title: Option<String>,
        }

        /// 群成员列表响应数据（JSON数组包装）
        #[derive(Debug, Serialize, Deserialize)]
        pub struct GroupMemberList(pub Vec<GroupMemberInfo>);

        /// 运行状态响应
        #[derive(Debug, Serialize, Deserialize)]
        pub struct StatusInfo {
            /// 是否在线（null表示未知）
            pub online: Option<bool>,
            /// 状态是否正常
            pub good: bool,
        }

        /// 版本信息响应
        #[derive(Debug, Serialize, Deserialize)]
        pub struct VersionInfo {
            /// 实现名称（如go-cqhttp）
            pub app_name: String,
            /// 实现版本
            pub app_version: String,
            /// 协议版本（如v11）
            pub protocol_version: String,
        }
    }
}

pub mod request {
    use super::response::*;
    use std::num::ParseIntError;

    use ioevent::rpc::*;
    use serde::{Deserialize, Serialize};

    use super::error::ApiError;
    use crate::message::{GroupID, MessageID, MessageNode, UserID, message_internal::InternalMessage};

    #[derive(Debug, Serialize, Deserialize, ProcedureCall)]
    pub struct NoneRequest;
    impl ProcedureCallRequest for NoneRequest {
        type RESPONSE = Result<(), ApiError>;
    }
    /// API请求包装结构
    ///
    /// # 字段说明
    /// - `echo`: 请求标识符，用于匹配异步响应
    /// - `kind`: 具体的API请求类型
    #[derive(Debug, Serialize, Deserialize)]
    pub struct ApiRequest {
        pub echo: String,
        #[serde(flatten)]
        pub kind: ApiRequestKind,
    }
    impl ApiRequest {
        pub fn new(echo: String, kind: ApiRequestKind) -> Self {
            Self { echo, kind }
        }
    }
    #[derive(Debug, Serialize, Deserialize)]
    #[serde(tag = "action", content = "params")]
    pub enum ApiRequestKind {
        #[serde(rename = "send_private_msg")]
        SendPrivateMsg(SendPrivateMsgParams),
        #[serde(rename = "send_group_msg")]
        SendGroupMsg(SendGroupMsgParams),
        #[serde(rename = "delete_msg")]
        DeleteMsg(DeleteMsgParams),
        #[serde(rename = "get_msg")]
        GetMsg(GetMsgParams),
        #[serde(rename = "set_group_kick")]
        SetGroupKick(SetGroupKickParams),
        #[serde(rename = "set_group_ban")]
        SetGroupBan(SetGroupBanParams),
        #[serde(rename = "set_group_admin")]
        SetGroupAdmin(SetGroupAdminParams),
        #[serde(rename = "set_group_card")]
        SetGroupCard(SetGroupCardParams),
        #[serde(rename = "set_group_leave")]
        SetGroupLeave(SetGroupLeaveParams),
        #[serde(rename = "set_friend_add_request")]
        SetFriendAddRequest(SetFriendAddRequestParams),
        #[serde(rename = "set_group_add_request")]
        SetGroupAddRequest(SetGroupAddRequestParams),
        #[serde(rename = "get_stranger_info")]
        GetStrangerInfo(GetStrangerInfoParams),
        #[serde(rename = "get_group_info")]
        GetGroupInfo(GetGroupInfoParams),
        #[serde(rename = "get_group_member_info")]
        GetGroupMemberInfo(GetGroupMemberInfoParams),
        #[serde(rename = "get_group_member_list")]
        GetGroupMemberList(GetGroupMemberListParams),
    }

    /// 发送私聊消息参数
    #[derive(Debug, Serialize, Deserialize, ProcedureCall)]
    pub struct SendPrivateMsgParams {
        user_id: i64,
        message: Vec<InternalMessage>,
        auto_escape: bool,
    }
    impl ProcedureCallRequest for SendPrivateMsgParams {
        type RESPONSE = Result<MessageIdResponse, ApiError>;
    }
    impl SendPrivateMsgParams {
        pub fn new(user_id: UserID, message: Vec<MessageNode>) -> Result<Self, ApiError> {
            let message = message.into_iter().map(|x| x.into()).collect();
            Ok(Self {
                user_id: user_id.into(),
                message,
                auto_escape: false,
            })
        }
    }

    /// 发送群消息参数（结构同私聊）
    #[derive(Debug, Serialize, Deserialize, ProcedureCall)]
    pub struct SendGroupMsgParams {
        group_id: i64,
        message: Vec<InternalMessage>,
        auto_escape: bool,
    }
    impl ProcedureCallRequest for SendGroupMsgParams {
        type RESPONSE = Result<MessageIdResponse, ApiError>;
    }
    impl SendGroupMsgParams {
        /// 创建发送群消息参数
        ///
        /// # 参数
        /// - `group_id`: 目标群号
        /// - `message`: 消息内容
        pub fn new(group_id: GroupID, message: Vec<MessageNode>) -> Result<Self, ApiError> {
            let message = message.into_iter().map(|x| x.into()).collect();
            Ok(Self {
                group_id: group_id.into(),
                message,
                auto_escape: false,
            })
        }
    }

    /// 消息撤回参数
    #[derive(Debug, Serialize, Deserialize, ProcedureCall)]
    pub struct DeleteMsgParams {
        message_id: i32,
    }
    impl ProcedureCallRequest for DeleteMsgParams {
        type RESPONSE = Result<(), ApiError>;
    }
    impl DeleteMsgParams {
        /// 创建消息撤回参数
        ///
        /// # 参数
        /// - `message_id`: 要撤回的消息ID
        pub fn new(message_id: MessageID) -> Result<Self, ParseIntError> {
            Ok(Self {
                message_id: message_id.0.parse()?,
            })
        }
    }

    /// 获取消息参数
    #[derive(Debug, Serialize, Deserialize, ProcedureCall)]
    pub struct GetMsgParams {
        message_id: i32,
    }
    impl ProcedureCallRequest for GetMsgParams {
        type RESPONSE = Result<MessageDetail, ApiError>;
    }
    impl GetMsgParams {
        /// 创建获取消息参数
        ///
        /// # 参数
        /// - `message_id`: 目标消息ID
        pub fn new(message_id: MessageID) -> Result<Self, ParseIntError> {
            Ok(Self {
                message_id: message_id.0.parse()?,
            })
        }
    }

    /// 群组踢人参数
    #[derive(Debug, Serialize, Deserialize, ProcedureCall)]
    pub struct SetGroupKickParams {
        group_id: i64,
        user_id: i64,
        reject_add_request: bool,
    }
    impl ProcedureCallRequest for SetGroupKickParams {
        type RESPONSE = Result<(), ApiError>;
    }
    impl SetGroupKickParams {
        /// 创建群组踢人参数
        ///
        /// # 参数
        /// - `group_id`: 目标群号
        /// - `user_id`: 被踢用户QQ号
        /// - `reject_add_request`: 是否拒绝后续加群
        pub fn new(
            group_id: GroupID,
            user_id: UserID,
            reject_add_request: bool,
        ) -> Result<Self, ParseIntError> {
            Ok(Self {
                group_id: group_id.into(),
                user_id: user_id.into(),
                reject_add_request,
            })
        }
    }

    /// 群组禁言参数
    #[derive(Debug, Serialize, Deserialize, ProcedureCall)]
    pub struct SetGroupBanParams {
        group_id: i64,
        user_id: i64,
        duration: i32,
    }
    impl ProcedureCallRequest for SetGroupBanParams {
        type RESPONSE = Result<(), ApiError>;
    }
    impl SetGroupBanParams {
        /// 创建群组禁言参数
        ///
        /// # 参数
        /// - `group_id`: 目标群号
        /// - `user_id`: 被禁言用户QQ号
        /// - `duration`: 禁言时长（秒）
        pub fn new(
            group_id: GroupID,
            user_id: UserID,
            duration: i32,
        ) -> Result<Self, ParseIntError> {
            Ok(Self {
                group_id: group_id.into(),
                user_id: user_id.into(),
                duration,
            })
        }
    }

    /// 设置管理员参数
    #[derive(Debug, Serialize, Deserialize, ProcedureCall)]
    pub struct SetGroupAdminParams {
        group_id: i64,
        user_id: i64,
        enable: bool,
    }
    impl ProcedureCallRequest for SetGroupAdminParams {
        type RESPONSE = Result<(), ApiError>;
    }
    impl SetGroupAdminParams {
        /// 创建设置管理员参数
        ///
        /// # 参数
        /// - `group_id`: 目标群号
        /// - `user_id`: 用户QQ号
        /// - `enable`: 是否设置为管理员
        pub fn new(
            group_id: GroupID,
            user_id: UserID,
            enable: bool,
        ) -> Result<Self, ParseIntError> {
            Ok(Self {
                group_id: group_id.into(),
                user_id: user_id.into(),
                enable,
            })
        }
    }

    /// 群名片设置参数
    #[derive(Debug, Serialize, Deserialize, ProcedureCall)]
    pub struct SetGroupCardParams {
        group_id: i64,
        user_id: i64,
        card: String,
    }
    impl ProcedureCallRequest for SetGroupCardParams {
        type RESPONSE = Result<(), ApiError>;
    }
    impl SetGroupCardParams {
        /// 创建群名片设置参数
        ///
        /// # 参数
        /// - `group_id`: 目标群号
        /// - `user_id`: 用户QQ号
        /// - `card`: 新群名片
        pub fn new(
            group_id: GroupID,
            user_id: UserID,
            card: String,
        ) -> Result<Self, ParseIntError> {
            Ok(Self {
                group_id: group_id.into(),
                user_id: user_id.into(),
                card,
            })
        }
    }

    /// 退群参数
    #[derive(Debug, Serialize, Deserialize, ProcedureCall)]
    pub struct SetGroupLeaveParams {
        group_id: i64,
        is_dismiss: bool,
    }
    impl ProcedureCallRequest for SetGroupLeaveParams {
        type RESPONSE = Result<(), ApiError>;
    }
    impl SetGroupLeaveParams {
        /// 创建退群参数
        ///
        /// # 参数
        /// - `group_id`: 目标群号
        /// - `is_dismiss`: 是否解散群
        pub fn new(group_id: GroupID, is_dismiss: bool) -> Result<Self, ParseIntError> {
            Ok(Self {
                group_id: group_id.into(),
                is_dismiss,
            })
        }
    }

    /// 好友请求处理参数
    #[derive(Debug, Serialize, Deserialize, ProcedureCall)]
    pub struct SetFriendAddRequestParams {
        flag: String,
        approve: bool,
        remark: String,
    }
    impl ProcedureCallRequest for SetFriendAddRequestParams {
        type RESPONSE = Result<(), ApiError>;
    }
    impl SetFriendAddRequestParams {
        /// 创建好友请求处理参数
        ///
        /// # 参数
        /// - `flag`: 请求标识
        /// - `approve`: 是否同意
        /// - `remark`: 备注信息
        pub fn new(flag: String, approve: bool, remark: String) -> Result<Self, ParseIntError> {
            Ok(Self {
                flag,
                approve,
                remark,
            })
        }
    }

    /// 加群请求处理参数
    #[derive(Debug, Serialize, Deserialize, ProcedureCall)]
    pub struct SetGroupAddRequestParams {
        pub flag: String,
        pub sub_type: String,
        pub approve: bool,
        pub reason: String,
    }
    impl ProcedureCallRequest for SetGroupAddRequestParams {
        type RESPONSE = Result<(), ApiError>;
    }
    impl SetGroupAddRequestParams {
        /// 创建加群请求处理参数
        ///
        /// # 参数
        /// - `flag`: 请求标识
        /// - `sub_type`: 请求类型
        /// - `approve`: 是否同意
        /// - `reason`: 拒绝理由
        pub fn new(
            flag: String,
            sub_type: String,
            approve: bool,
            reason: String,
        ) -> Result<Self, ParseIntError> {
            Ok(Self {
                flag,
                sub_type,
                approve,
                reason,
            })
        }
    }

    /// 陌生人信息查询参数
    #[derive(Debug, Serialize, Deserialize, ProcedureCall)]
    pub struct GetStrangerInfoParams {
        user_id: i64,
        no_cache: bool,
    }
    impl ProcedureCallRequest for GetStrangerInfoParams {
        type RESPONSE = Result<StrangerInfo, ApiError>;
    }
    impl GetStrangerInfoParams {
        /// 创建陌生人信息查询参数
        ///
        /// # 参数
        /// - `user_id`: 目标QQ号
        /// - `no_cache`: 是否不使用缓存
        pub fn new(user_id: u64, no_cache: bool) -> Self {
            Self {
                user_id: user_id as i64,
                no_cache,
            }
        }
    }

    /// 群信息查询参数
    #[derive(Debug, Serialize, Deserialize, ProcedureCall)]
    pub struct GetGroupInfoParams {
        group_id: i64,
        no_cache: bool,
    }
    impl ProcedureCallRequest for GetGroupInfoParams {
        type RESPONSE = Result<GroupInfo, ApiError>;
    }
    impl GetGroupInfoParams {
        /// 创建群信息查询参数
        ///
        /// # 参数
        /// - `group_id`: 目标群号
        /// - `no_cache`: 是否不使用缓存
        pub fn new(group_id: GroupID, no_cache: bool) -> Result<Self, ParseIntError> {
            Ok(Self {
                group_id: group_id.into(),
                no_cache,
            })
        }
    }

    /// 群成员信息查询参数
    #[derive(Debug, Serialize, Deserialize, ProcedureCall)]
    pub struct GetGroupMemberInfoParams {
        group_id: i64,
        user_id: i64,
        no_cache: bool,
    }
    impl ProcedureCallRequest for GetGroupMemberInfoParams {
        type RESPONSE = Result<GroupMemberInfo, ApiError>;
    }
    impl GetGroupMemberInfoParams {
        /// 创建群成员信息查询参数
        ///
        /// # 参数
        /// - `group_id`: 目标群号
        /// - `user_id`: 成员QQ号
        /// - `no_cache`: 是否不使用缓存
        pub fn new(
            group_id: GroupID,
            user_id: UserID,
            no_cache: bool,
        ) -> Result<Self, ParseIntError> {
            Ok(Self {
                group_id: group_id.into(),
                user_id: user_id.into(),
                no_cache,
            })
        }
    }

    /// 群成员列表查询参数
    #[derive(Debug, Serialize, Deserialize, ProcedureCall)]
    pub struct GetGroupMemberListParams {
        group_id: i64,
    }
    impl ProcedureCallRequest for GetGroupMemberListParams {
        type RESPONSE = Result<GroupMemberList, ApiError>;
    }
    impl GetGroupMemberListParams {
        /// 创建群成员列表查询参数
        ///
        /// # 参数
        /// - `group_id`: 目标群号
        pub fn new(group_id: GroupID) -> Result<Self, ParseIntError> {
            Ok(Self {
                group_id: group_id.into(),
            })
        }
    }

    impl From<SendPrivateMsgParams> for ApiRequestKind {
        fn from(value: SendPrivateMsgParams) -> Self {
            Self::SendPrivateMsg(value)
        }
    }

    impl From<SendGroupMsgParams> for ApiRequestKind {
        fn from(value: SendGroupMsgParams) -> Self {
            Self::SendGroupMsg(value)
        }
    }

    impl From<DeleteMsgParams> for ApiRequestKind {
        fn from(value: DeleteMsgParams) -> Self {
            Self::DeleteMsg(value)
        }
    }

    impl From<GetMsgParams> for ApiRequestKind {
        fn from(value: GetMsgParams) -> Self {
            Self::GetMsg(value)
        }
    }

    impl From<SetGroupKickParams> for ApiRequestKind {
        fn from(value: SetGroupKickParams) -> Self {
            Self::SetGroupKick(value)
        }
    }

    impl From<SetGroupBanParams> for ApiRequestKind {
        fn from(value: SetGroupBanParams) -> Self {
            Self::SetGroupBan(value)
        }
    }

    impl From<SetGroupAdminParams> for ApiRequestKind {
        fn from(value: SetGroupAdminParams) -> Self {
            Self::SetGroupAdmin(value)
        }
    }

    impl From<SetGroupCardParams> for ApiRequestKind {
        fn from(value: SetGroupCardParams) -> Self {
            Self::SetGroupCard(value)
        }
    }

    impl From<SetGroupLeaveParams> for ApiRequestKind {
        fn from(value: SetGroupLeaveParams) -> Self {
            Self::SetGroupLeave(value)
        }
    }

    impl From<SetFriendAddRequestParams> for ApiRequestKind {
        fn from(value: SetFriendAddRequestParams) -> Self {
            Self::SetFriendAddRequest(value)
        }
    }

    impl From<SetGroupAddRequestParams> for ApiRequestKind {
        fn from(value: SetGroupAddRequestParams) -> Self {
            Self::SetGroupAddRequest(value)
        }
    }

    impl From<GetStrangerInfoParams> for ApiRequestKind {
        fn from(value: GetStrangerInfoParams) -> Self {
            Self::GetStrangerInfo(value)
        }
    }

    impl From<GetGroupInfoParams> for ApiRequestKind {
        fn from(value: GetGroupInfoParams) -> Self {
            Self::GetGroupInfo(value)
        }
    }

    impl From<GetGroupMemberInfoParams> for ApiRequestKind {
        fn from(value: GetGroupMemberInfoParams) -> Self {
            Self::GetGroupMemberInfo(value)
        }
    }

    impl From<GetGroupMemberListParams> for ApiRequestKind {
        fn from(value: GetGroupMemberListParams) -> Self {
            Self::GetGroupMemberList(value)
        }
    }
}

pub mod error {
    use ioevent::{error::CallSubscribeError, rpc::*};
    use serde::{Deserialize, Serialize};
    use thiserror::Error;

    #[derive(Debug, Error, Serialize, Deserialize, ProcedureCall)]
    pub enum ApiError {
        #[error("API请求失败: {0}")]
        RequestFailed(String),
        #[error("API响应解析失败: {0}")]
        ResponseParseFailed(String),
        #[error("API响应为空")]
        ResponseEmpty,
        #[error("API响应状态码错误: {0}")]
        StatusCodeError(u16),
    }
    impl ProcedureCallResponse for ApiError {}
    impl From<ApiError> for CallSubscribeError {
        fn from(value: ApiError) -> Self {
            match value {
                ApiError::RequestFailed(e) => CallSubscribeError::ProcedureCall(e),
                ApiError::ResponseParseFailed(e) => CallSubscribeError::ProcedureCall(e),
                ApiError::ResponseEmpty => {
                    CallSubscribeError::ProcedureCall("API响应为空".to_string())
                }
                ApiError::StatusCodeError(e) => {
                    CallSubscribeError::Other(format!("API响应状态码错误: {}", e))
                }
            }
        }
    }
}
