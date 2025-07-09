pub mod response {
    use crate::util::de_str_from_num;
    use serde::{Deserialize, Serialize};
    use sithra_kit::{
        transport::datapack::DataPack,
        types::{message::Message, smallvec::SmallVec},
    };
    use ulid::Ulid;

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct ApiResponse {
        retcode: i32,
        echo:    Ulid,
        data:    Option<ApiResponseKind>,
    }

    impl ApiResponse {
        #[must_use]
        pub fn into_rep(self, bot_id: &str) -> DataPack {
            let Self {
                retcode,
                echo,
                data,
            } = self;
            let Some(data) = data else {
                return DataPack::builder()
                    .correlate(echo)
                    .bot_id(bot_id)
                    .build_with_error(&format!("Call OneBot API Error, RETCODE: {retcode}"));
            };
            match data {
                ApiResponseKind::SendMessage(send_msg) => {
                    let payload: Message = Message {
                        id:      send_msg.message_id,
                        content: SmallVec::new(),
                    };
                    DataPack::builder().correlate(echo).build_with_payload(payload)
                }
            }
        }
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[serde(untagged)]
    pub enum ApiResponseKind {
        SendMessage(SendMessageResponse),
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct SendMessageResponse {
        #[serde(deserialize_with = "de_str_from_num")]
        message_id: String,
    }
}

pub mod request {
    use serde::{Deserialize, Serialize};
    use sithra_kit::types::smallvec::SmallVec;
    use ulid::Ulid;

    use crate::message::internal::InternalOneBotSegment;

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct ApiCall<T> {
        pub action: String,
        pub params: T,
        pub echo:   Ulid,
    }

    impl<T> ApiCall<T> {
        pub fn new(action: &impl ToString, params: T, echo: Ulid) -> Self {
            Self {
                action: action.to_string(),
                params,
                echo,
            }
        }
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct SendMessage {
        #[serde(flatten)]
        pub message_type: SendMessageKind,
        pub message:      SmallVec<[InternalOneBotSegment; 1]>,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[serde(rename_all = "snake_case", tag = "message_type")]
    pub enum SendMessageKind {
        Private { user_id: String },
        Group { group_id: String },
    }
}
