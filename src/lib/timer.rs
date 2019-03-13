use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::Duration;

pub struct Timer {
    pub receiver: Receiver<()>,
}

impl Timer {
    pub fn new() -> Timer {
        let (sender, receiver) = channel();

        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_millis(16));
                sender.send(()).unwrap();
            }
        });

        Timer { receiver: receiver }
    }
}
