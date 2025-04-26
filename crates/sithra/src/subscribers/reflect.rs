use ioevent::{State, event::AnyEvent, rpc::ProcedureCallData, subscriber};
use log::error;

use crate::client::ClientState;

#[subscriber]
pub async fn reflect_procedure(state: State<ClientState>, pcd: ProcedureCallData) {
    let result = state.wright.emit(&pcd);
    if let Err(e) = result {
        error!("反射调用失败: {}", e);
    }
}

#[subscriber]
pub async fn reflect_event(state: State<ClientState>, event: AnyEvent) {
    let result = state.wright.emit(&event);
    if let Err(e) = result {
        error!("反射事件失败: {}", e);
    }
}
