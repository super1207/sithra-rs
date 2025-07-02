use serde::{Deserialize, Serialize};

use crate::{api::response::ApiResponse, event::RawEvent};

pub mod api;
pub mod event;
pub mod message;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum OneBotMessage {
    Event(RawEvent),
    Api(ApiResponse),
}
