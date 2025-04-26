/* old version
use crate::client::*;
use ioevent::error::CallSubscribeError;
use ioevent::rpc::*;
use log::error;
use rand::RngCore; */
/* old version
use sithra_common::api::data::request;
use sithra_common::api::api_internal::ApiResponseKind;
use sithra_common::api::*;
use sithra_common::error::ApiError; */
/* old version
use std::time::Duration;
use tokio::sync::oneshot;
use tokio::time::timeout; */
/* old version
macro_rules! api_procedure {
    ($name:ident, $request:ty, $response:ident) => {
        #[procedure]
        pub async fn $name(state: State<ClientState>, call: $request) -> Result {
            let (sender, receiver) = oneshot::channel();
            let echo = ECHO_GENERATOR.lock().await.next_u64();
            let request = ApiRequest::new(echo.to_string(), call.into());
            state.state.api_shooters.insert(echo.to_string(), sender);
            state.state.api_sender.send(request).map_err(|e| {
                CallSubscribeError::Other(format!(
                    "{} 发送请求失败: {}",
                    stringify!($name),
                    e.to_string()
                ))
            })?;
            let response = timeout(Duration::from_secs(10), receiver)
                .await
                .map_err(|e| {
                    CallSubscribeError::Other(format!(
                        "{} 接收响应超时: {}",
                        stringify!($name),
                        e.to_string()
                    ))
                })?;
            match response {
                Ok(Ok(response)) => {
                    log::debug!("{} 接收响应: {:?}", stringify!($name), response);
                    if let Some(ApiResponseKind::$response(response)) = response.data {
                        Ok(Ok(response.into()))
                    } else {
                        Ok(Err(ApiError::ResponseEmpty))
                    }
                }
                Ok(Err(e)) => Ok(Err(e)),
                Err(e) => Err(CallSubscribeError::Other(e.to_string())),
            }
        }
    };
}
macro_rules! api_procedure_unit {
    ($name:ident, $request:ty) => {
        #[procedure]
        pub async fn $name(state: State<ClientState>, call: $request) -> Result {
            let (sender, receiver) = oneshot::channel();
            let echo = ECHO_GENERATOR.lock().await.next_u64();
            let request = ApiRequest::new(echo.to_string(), call.into());
            state.state.api_shooters.insert(echo.to_string(), sender);
            state.state.api_sender.send(request).map_err(|e| {
                CallSubscribeError::Other(format!(
                    "{} 发送请求失败: {}",
                    stringify!($name),
                    e.to_string()
                ))
            })?;
            let result = timeout(Duration::from_secs(10), receiver)
                .await
                .map_err(|e| {
                    CallSubscribeError::Other(format!(
                        "{} 接收响应超时: {}",
                        stringify!($name),
                        e.to_string()
                    ))
                })?;
            match result {
                Ok(Ok(_)) => Ok(Ok(())),
                Ok(Err(e)) => Ok(Err(e)),
                Err(e) => Err(CallSubscribeError::Other(e.to_string())),
            }
        }
    };
}

api_procedure!(
    api_send_private_msg,
    request::SendPrivateMsgParams,
    MessageIdResponse
);
api_procedure!(
    api_send_group_msg,
    request::SendGroupMsgParams,
    MessageIdResponse
);
api_procedure_unit!(api_delete_msg, request::DeleteMsgParams);
api_procedure_unit!(api_set_group_kick, request::SetGroupKickParams);
api_procedure_unit!(api_set_group_ban, request::SetGroupBanParams);
api_procedure_unit!(api_set_group_admin, request::SetGroupAdminParams);
api_procedure_unit!(api_set_group_card, request::SetGroupCardParams);
api_procedure_unit!(api_set_group_leave, request::SetGroupLeaveParams);
api_procedure_unit!(
    api_set_friend_add_request,
    request::SetFriendAddRequestParams
);
api_procedure_unit!(api_set_group_add_request, request::SetGroupAddRequestParams);
api_procedure!(
    api_get_stranger_info,
    request::GetStrangerInfoParams,
    StrangerInfo
);
api_procedure!(api_get_group_info, request::GetGroupInfoParams, GroupInfo);
api_procedure!(
    api_get_group_member_info,
    request::GetGroupMemberInfoParams,
    GroupMemberInfo
);
api_procedure!(
    api_get_group_member_list,
    request::GetGroupMemberListParams,
    GroupMemberList
);
api_procedure!(
    api_create_forward_msg,
    request::CreateForwardMsgParams,
    ForwardIdResponse
);
#[procedure]
pub async fn api_get_msg(state: State<ClientState>, call: request::GetMsgParams) -> Result {
    let (sender, receiver) = oneshot::channel();
    let echo = ECHO_GENERATOR.lock().await.next_u64();
    let request = ApiRequest::new(echo.to_string(), call.into());
    state.state.api_shooters.insert(echo.to_string(), sender);
    state
        .state
        .api_sender
        .send(request)
        .map_err(|e| CallSubscribeError::Other(e.to_string()))?;
    let response = timeout(Duration::from_secs(10), receiver)
        .await
        .map_err(|e| {
            CallSubscribeError::Other(format!("api_get_msg接收响应超时: {}", e.to_string()))
        })?;
    match response {
        Ok(Err(e)) => Ok(Err(e)),
        Ok(Ok(response)) => {
            log::debug!("api_get_msg 接收响应: {:?}", response);
            if let Some(ApiResponseKind::MessageDetail(response)) = response.data {
                let (message, errors) = response.into();
                for error in errors {
                    error!("消息解析错误: {:?}", error);
                }
                Ok(Ok(message))
            } else {
                Ok(Err(ApiError::ResponseEmpty))
            }
        }
        Err(e) => Err(CallSubscribeError::Other(e.to_string())),
    }
}
 */