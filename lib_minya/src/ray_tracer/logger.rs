use crate::prelude::*;
use log::{Level as LogLevel, Level, Metadata, Record};
use std::sync::{
    mpsc::{channel, Receiver, Sender},
    Mutex,
};
fn print_debug() -> bool {
    const DEBUG: bool = true;
    rand_u32(0, 1_000) == 0 && DEBUG
}
#[derive(Debug, Clone)]
pub enum LogMessage {
    Trace(String),
    Debug(String),
    Info(String),
    Warn(String),
    Error(String),
}
static mut LOG_MESSAGES: Mutex<Option<Vec<LogMessage>>> = Mutex::new(None);
static mut LOG_SETUP: Mutex<bool> = Mutex::new(false);
static mut LOGGER: Logger = Logger {};
pub struct Logger {}

impl Logger {
    pub fn init() {
        unsafe {
            let mut log_setup = LOG_SETUP.lock().expect("failed to get lock");
            if *log_setup == false {
                log::set_logger(&LOGGER)
                    .map(|()| log::set_max_level(log::LevelFilter::Debug))
                    .expect("failed to set logger");
                *log_setup = true;
            }
        }
    }
    pub fn new() -> Self {
        Self {}
    }
    pub fn get_log_messages() -> Vec<LogMessage> {
        let log_opt = unsafe { LOG_MESSAGES.lock().expect("failed to get lock") };
        if log_opt.is_some() {
            log_opt.as_ref().unwrap().clone()
        } else {
            Vec::new()
        }
    }
}
impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let mut log_result = unsafe { LOG_MESSAGES.lock().expect("failed to get lock") };
            if log_result.is_none() {
                log::set_max_level(log::LevelFilter::Debug);
                *log_result = Some(vec![]);
            }
            let mut log_messages = log_result.as_mut().unwrap();
            let message = match record.level() {
                LogLevel::Trace => LogMessage::Trace(record.args().to_string()),
                LogLevel::Debug => {
                    if print_debug() {
                        LogMessage::Debug(record.args().to_string())
                    } else {
                        return;
                    }
                }
                LogLevel::Info => LogMessage::Info(record.args().to_string()),
                LogLevel::Warn => LogMessage::Warn(record.args().to_string()),
                LogLevel::Error => LogMessage::Error(record.args().to_string()),
            };

            log_messages.push(message);
        }
    }

    fn flush(&self) {}
}
