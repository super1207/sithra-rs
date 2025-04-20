use ioevent::subscriber;
use sithra_common::log::LogEvent;

#[subscriber]
pub async fn log_subscriber(event: LogEvent) {
    let level = event.level.into();
    log::log!(target: event.target.as_str(), level, "{}", event.message);
}