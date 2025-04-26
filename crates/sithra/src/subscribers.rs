pub mod api;
pub mod logger;
pub mod reflect;
use ioevent::{EventData, Subscriber, create_subscriber};
use logger::*;
use reflect::reflect_event;

use crate::client::ClientState;

pub const SUBSCRIBERS: &[Subscriber<ClientState>] = &[
    /* old version
    create_subscriber!(api_send_private_msg),
    create_subscriber!(api_send_group_msg),
    create_subscriber!(api_delete_msg),
    create_subscriber!(api_set_group_kick),
    create_subscriber!(api_set_group_ban),
    create_subscriber!(api_set_group_admin),
    create_subscriber!(api_set_group_card),
    create_subscriber!(api_set_group_leave),
    create_subscriber!(api_set_friend_add_request),
    create_subscriber!(api_set_group_add_request),
    create_subscriber!(api_get_stranger_info),
    create_subscriber!(api_get_group_info),
    create_subscriber!(api_get_group_member_info),
    create_subscriber!(api_get_group_member_list),
    create_subscriber!(api_get_msg),
    create_subscriber!(api_create_forward_msg), */
    create_subscriber!(log_subscriber),
    create_subscriber!(tracing_subscriber),
    create_subscriber!(reflect_event),
];

#[ioevent::subscriber]
pub async fn tracing_subscriber(event: EventData) {
    log::trace!("tracing_subscriber: {:?}", event);
}
