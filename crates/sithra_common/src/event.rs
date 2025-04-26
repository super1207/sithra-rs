use std::ops::Deref;

use ioevent::Event;
use serde::{Deserialize, Deserializer, Serialize};

use crate::message::{Message, MessageRaw};
use crate::model::{Channel, UserId};

#[derive(Debug, Clone, Deserialize, Serialize, Event)]
pub struct MessageReceived<M: Message> {
    pub channel: Channel,
    pub user_id: UserId,
    #[serde(deserialize_with = "deserialize_message")]
    pub message: M,
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
