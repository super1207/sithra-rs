use crate::{
    message::{Message, MessageRaw},
    model::{Channel, GenericId, MessageId},
};
use ioevent::{error::TryFromEventError, rpc::*};
use serde::{Deserialize, Serialize};

pub use base::*;
mod base {
    use super::*;
    /// 基础请求
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SithraCall<C> {
        generic_id: GenericId,
        call: C,
    }
    pub trait SithraCallRequest {
        type RESPONSE: ProcedureCallResponse;
    }
    impl<C: ProcedureCall> SithraCall<C> {
        /// 创建一个基础请求
        pub fn new(generic_id: GenericId, call: C) -> Self {
            Self { generic_id, call }
        }
        /// 匹配通用ID
        pub fn match_generic_id(&self, generic_id: &GenericId) -> bool {
            &self.generic_id == generic_id
        }
    }
    impl<C: ProcedureCall + SithraCallRequest> ProcedureCall for SithraCall<C> {
        fn path() -> String {
            format!("SithraCall<{}>", C::path())
        }
    }
    impl<C: ProcedureCall + SithraCallRequest> TryFrom<ProcedureCallData> for SithraCall<C> {
        type Error = TryFromEventError;
        fn try_from(value: ProcedureCallData) -> Result<Self, Self::Error> {
            Ok(value.payload.deserialized()?)
        }
    }
    impl<C: ProcedureCall + SithraCallRequest> ProcedureCallRequest for SithraCall<C> {
        type RESPONSE = C::RESPONSE;
    }
}

/* ------------------------------------------------------------ */

/// 发送消息
#[derive(Debug, Clone, Serialize, Deserialize, ProcedureCall)]
pub struct SendMessage {
    message: MessageRaw,
    channel: Channel,
}
impl SendMessage {
    pub fn new<M: Message>(message: M, channel: Channel) -> Self {
        Self {
            message: message.into_raw(),
            channel,
        }
    }
}
impl SithraCallRequest for SendMessage {
    type RESPONSE = SendMessageResponse;
}
#[derive(Debug, Clone, Serialize, Deserialize, ProcedureCall)]
pub struct SendMessageResponse {
    pub message_id: Option<MessageId>,
}
impl ProcedureCallResponse for SendMessageResponse {}

/// 删除消息
#[derive(Debug, Clone, Serialize, Deserialize, ProcedureCall)]
pub struct DeleteMessage {
    message_id: MessageId,
}
impl DeleteMessage {
    pub fn new(message_id: MessageId) -> Self {
        Self { message_id }
    }
}
impl SithraCallRequest for DeleteMessage {
    type RESPONSE = ();
}