use crate::prelude::*;
use crate::Message::LoadScenario;
use log::{Level as LogLevel, Level, Metadata, Record};
use std::sync::{
    mpsc::{channel, Receiver, Sender},
    Mutex,
};
fn print_debug() -> bool {
    const DEBUG: bool = true;
    rand_u32(0, 1_000) == 0 && DEBUG
}
#[derive(Debug)]
pub enum LogMessage {
    Trace(String),
    Debug(String),
    Info(String),
    Warn(String),
    Error(String),
}
pub struct Logger {
    sender: Mutex<Sender<LogMessage>>,
}
impl Logger {
    pub fn new() -> (Self, Receiver<LogMessage>) {
        let (sender, reciever) = channel();
        (
            Self {
                sender: Mutex::new(sender),
            },
            reciever,
        )
    }
}
impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
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

            self.sender
                .lock()
                .expect("failed to acquire lock")
                .send(message);
        }
    }

    fn flush(&self) {
        println!("flush??")
    }
}
