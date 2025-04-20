use std::sync::{Mutex, OnceLock};

use ioevent::{EffectWright, Event};
use serde::{Deserialize, Serialize};

static LOGGER: OnceLock<EventifyLogger> = OnceLock::new();

pub struct EventifyLogger {
    effect: Mutex<Option<EffectWright>>,
}

impl EventifyLogger {
    pub fn new() -> Self {
        Self {
            effect: Mutex::new(None),
        }
    }

    pub fn write(&self, effect: EffectWright) {
        if let Ok(mut guard) = self.effect.lock() {
            *guard = Some(effect);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct LogEvent {
    pub level: LogLevel,
    pub message: String,
    pub target: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    #[serde(rename = "debug")]
    Debug,
    #[serde(rename = "info")]
    Info,
    #[serde(rename = "warn")]
    Warn,
    #[serde(rename = "error")]
    Error,
    #[serde(rename = "trace")]
    Trace,
}
impl From<log::Level> for LogLevel {
    fn from(value: log::Level) -> Self {
        match value {
            log::Level::Debug => LogLevel::Debug,
            log::Level::Info => LogLevel::Info,
            log::Level::Warn => LogLevel::Warn,
            log::Level::Error => LogLevel::Error,
            log::Level::Trace => LogLevel::Trace,
        }
    }
}
impl From<LogLevel> for log::Level {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Debug => log::Level::Debug,
            LogLevel::Info => log::Level::Info,
            LogLevel::Warn => log::Level::Warn,
            LogLevel::Error => log::Level::Error,
            LogLevel::Trace => log::Level::Trace,
        }
    }
}
impl From<LogLevel> for log::LevelFilter {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Debug => log::LevelFilter::Debug,
            LogLevel::Info => log::LevelFilter::Info,
            LogLevel::Warn => log::LevelFilter::Warn,
            LogLevel::Error => log::LevelFilter::Error,
            LogLevel::Trace => log::LevelFilter::Trace,
        }
    }
}

impl log::Log for EventifyLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Info
    }

    fn log(&self, record: &log::Record) {
        let log_event = LogEvent {
            level: LogLevel::from(record.level()),
            message: format!("{}", record.args()),
            target: record.target().to_string(),
        };
        if let Ok(guard) = self.effect.lock().as_ref() {
            if let Some(effect) = guard.as_ref() {
                let _ = effect.emit(&log_event);
            }
        }
    }

    fn flush(&self) {}
}

pub fn init_log(effect: EffectWright, level: log::LevelFilter) {
    let logger = LOGGER.get_or_init(|| EventifyLogger::new());
    logger.write(effect);
    log::set_logger(logger).unwrap();
    log::set_max_level(level);
}
