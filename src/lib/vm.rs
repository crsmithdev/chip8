extern crate sdl2;
extern crate sdl2_sys;

use cpu::{Chip8, Chip8Error, Chip8State, MAX_PROGRAM_SIZE};
use rom;

use std::io::Read;

use std::cmp;
use std::path;
use std::fs;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::thread;
use std::sync::RwLock;
use std::time::Duration;
use std::sync::{Arc, Mutex};

const DEFAULT_SPEED: f32 = 100f32;

#[derive(Debug)]
enum VmMessage {
    Speed(f32),
    Load(String),
    Pause,
    Step,
}

pub struct Vm {
    vm: Arc<Mutex<Chip8>>,
    sender: Option<Sender<VmMessage>>,
    error: Arc<RwLock<Option<Chip8Error>>>,
}

impl Vm {
    pub fn new() -> Vm {
        let mut vm = Chip8::new();
        vm.load_rom(&rom::BOOT).unwrap();

        Vm {
            vm: Arc::new(Mutex::new(vm)),
            error: Arc::new(RwLock::new(None)),
            sender: None,
        }
    }

    pub fn run(&mut self) {
        let vm = self.vm.clone();
        let error = self.error.clone();
        let (sender, receiver) = channel();
        let mut speed = DEFAULT_SPEED;
        let mut paused = false;

        self.sender = Some(sender);

        thread::spawn(move || {
            loop {
                let mut vm = vm.lock().unwrap();
                let mut step = !paused;

                for message in receiver.try_iter() {
                    match message {
                        VmMessage::Speed(f) => speed *= f,
                        VmMessage::Pause => {
                            paused = !paused;
                            step = !paused;
                        },
                        VmMessage::Step => {
                            step = true;
                        }
                        VmMessage::Load(s) => {
                            let bytes = Vm::load_threaded(&s).unwrap();
                            vm.load_rom(&bytes).unwrap();
                        }
                    };
                }

                if step {
                    if let Err(err) = vm.step() {
                        *(error.write().unwrap()) = Some(err);
                        println!("error: {:?}", err);
                        paused = true;
                    }
                }

                let ms = cmp::max(1, (1.0f32 / speed * 1000f32) as u64); // TODO improve
                thread::sleep(Duration::from_millis(ms));
            }
        });
    }

    pub fn state(&self) -> Chip8State {
        let vm = self.vm.lock().unwrap();
        vm.state()
    }

    pub fn speed(&mut self, factor: f32) {
        self.send(VmMessage::Speed(factor));
    }

    pub fn pause(&mut self) {
        self.send(VmMessage::Pause);
    }

    pub fn step(&mut self) {
        self.send(VmMessage::Step);
    }

    pub fn load(&mut self, path: &str) {
        self.send(VmMessage::Load(path.to_owned()));    // TODO surface errors
    }

    fn send(&mut self, message: VmMessage) {
        if let Some(ref mut sender) = self.sender {
            sender.send(message).unwrap();
        }
    }

    fn load_threaded(path: &str) -> Result<Box<[u8]>, Chip8Error> {
        let path = path::Path::new(path);
        let mut buffer = [0; MAX_PROGRAM_SIZE];

        match fs::File::open(&path) {
            Err(_) => return Err(Chip8Error::ProgramLoadError), // TODO error handling
            Ok(mut file) => {
                return match file.read(&mut buffer) {
                    Err(_) => Err(Chip8Error::ProgramLoadError), // TODO error handling
                    Ok(_) => Ok(Box::new(buffer)),
                };
            }
        }
    }
}
