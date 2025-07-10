use serde::{Deserialize, Deserializer, Serialize};
use sithra_kit::server::response::Response;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use ulid::Ulid;

use crate::{AdapterState, api::request::ApiCall};

pub(crate) fn de_str_from_num<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let num: i64 = Deserialize::deserialize(deserializer)?;
    Ok(num.to_string())
}

pub fn send_req<T: Serialize>(
    state: &AdapterState,
    id: Ulid,
    api_call: &ApiCall<T>,
    err: &str,
) -> Option<Response> {
    let req = serde_json::to_string(api_call);
    let req = match req {
        Err(se_err) => {
            log::error!("Failed to serialize {err} request: {se_err}");
            let mut response =
                Response::error(format!("Failed to serialize {err} request: {se_err}"));
            response.correlate(id);
            return Some(response);
        }
        Ok(req) => req,
    };
    let result = state.ws_tx.send(WsMessage::Text(req.into()));
    if let Err(ws_err) = result {
        log::error!("Failed to send {err} request: {ws_err}");
        let mut response = Response::error(format!("Failed to send {err} request: {ws_err}"));
        response.correlate(id);
        return Some(response);
    }
    None
}
