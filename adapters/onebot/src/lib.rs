use serde::{Deserialize, Serialize};

use crate::{api::response::ApiResponse, event::RawEvent};

pub mod api;
pub mod event;
pub mod message;
pub mod util;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum OneBotMessage {
    Event(RawEvent),
    Api(ApiResponse),
}

#[cfg(test)]
mod tests {
    use crate::OneBotMessage;

    #[test]
    fn de() {
        let r = r#"{"message_type":"private","sub_type":"friend","message_id":62224923,"user_id":3605331714,"message":[{"type":"text","data":{"text":"aaaa"}}],"raw_message":"aaaa","font":0,"sender":{"user_id":3605331714,"nickname":"\u4FD7\u624B","sex":"unknown"},"target_id":1921576220,"message_style":{"bubble_id":0,"pendant_id":0,"font_id":0,"font_effect_id":0,"is_cs_font_effect_enabled":false,"bubble_diy_text_id":0},"time":1752007344,"self_id":1921576220,"post_type":"message"}"#;
        let _e: OneBotMessage = serde_json::from_str(r).unwrap();
    }
}
