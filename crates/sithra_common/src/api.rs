use crate::{
    message::{Message, MessageRaw},
    model::{Channel, GenericId, MessageId},
};
use ioevent::{error::TryFromEventError, rpc::*};
use serde::{Deserialize, Serialize};

pub use base::*;
mod base {
    use std::ops::{Deref, DerefMut};

    use crate::model::EnsureGenericId;

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
        pub fn take_call(self) -> C {
            self.call
        }
        pub fn take_generic_id(self) -> GenericId {
            self.generic_id
        }
        pub fn match_adapter<T: EnsureGenericId>(&self) -> bool {
            T::match_adapter(&self.generic_id)
        }
    }
    impl<C: ProcedureCall + SithraCallRequest> Deref for SithraCall<C> {
        type Target = C;
        fn deref(&self) -> &Self::Target {
            &self.call
        }
    }
    impl<C: ProcedureCall + SithraCallRequest> DerefMut for SithraCall<C> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.call
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
    pub channel: Channel,
}
impl SendMessage {
    pub fn new<M: Message>(message: M, channel: Channel) -> Self {
        Self {
            message: message.into_raw(),
            channel,
        }
    }
    pub fn message<M: Message>(&self) -> M {
        M::from_raw(self.message.clone())
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
    pub message_id: MessageId,
}
impl DeleteMessage {
    pub fn new(message_id: MessageId) -> Self {
        Self { message_id }
    }
}
impl SithraCallRequest for DeleteMessage {
    type RESPONSE = ();
}