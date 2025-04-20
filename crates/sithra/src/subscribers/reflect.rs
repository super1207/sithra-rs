use ioevent::{State, rpc::ProcedureCallData, subscriber};
use log::error;

use crate::client::ClientState;

#[subscriber]
pub async fn reflect_subscriber(state: State<ClientState>, pcd: ProcedureCallData) {
    let result = state.wright.emit(&pcd);
    if let Err(e) = result {
        error!("反射调用失败: {}", e);
    }
}
