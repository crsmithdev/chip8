use log::{Level, Metadata, Record};
use std::collections::VecDeque;
use std::iter::FromIterator;
use std::sync::Mutex;

lazy_static! {
    static ref LOGGER: Logger = Logger::new();
    static ref BUFFER: Mutex<VecDeque<String>> = Mutex::new(VecDeque::new());
}

pub struct Logger {
    buffer: &'static Mutex<VecDeque<String>>,
}

impl Logger {
    pub fn new() -> Logger {
        Logger { buffer: &*BUFFER }
    }

    pub fn unread(&self) -> usize {
        self.buffer.lock().unwrap().len()
    }

    pub fn read(&self) -> Vec<String> {
        let mut buffer = self.buffer.lock().unwrap();
        let output = buffer.split_off(0);
        Vec::from_iter(output.into_iter())
    }
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Debug
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
            if record.level() >= Level::Info {
                let mut buffer = self.buffer.lock().unwrap();
                let message = format!("{}", record.args());
                buffer.push_back(message);
            }
        }
    }

    fn flush(&self) {}
}

pub fn init<'a>() -> &'a Logger {
    let logger = &*LOGGER;
    log::set_logger(logger).unwrap();
    log::set_max_level(log::LevelFilter::Debug);
    logger
}
