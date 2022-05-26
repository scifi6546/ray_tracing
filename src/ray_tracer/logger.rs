use log::{Level as LogLevel, Level, Metadata, Record};
use std::sync::mpsc::{channel, Receiver, Sender};
pub enum LogMessage {Debug(String)}
pub struct Logger {
    sender: Sender<LogMessage>,
}
impl Logger {
    pub fn new() -> (Self, Receiver<LogMessage>) {
        let (sender, reciever) = channel();
        (Self { sender }, reciever)
    }
}
impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()){
            println!("todo configure other end!");
            let message = match record.level(){LogLevel::Debug=>LogMessage::Debug(record.args())}
        }
        todo!()
    }

    fn flush(&self) {
        todo!()
    }
}
