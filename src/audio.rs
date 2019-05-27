use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired, AudioStatus};
use sdl2::AudioSubsystem;

struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        for x in out.iter_mut() {
            *x = if self.phase <= 0.5 {
                self.volume
            } else {
                -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}

pub struct Audio {
    device: AudioDevice<SquareWave>,
}

impl Audio {
    const FREQUENCY: i32 = 44_100;
    const PHASE_INC: f32 = 220.0;
    const PHASE: f32 = 0.0;
    const VOLUME: f32 = 0.25;
    const CHANNELS: u8 = 1;

    pub fn new(audio: &AudioSubsystem) -> Audio {
        let desired_spec = AudioSpecDesired {
            freq: Some(Self::FREQUENCY),
            channels: Some(Self::CHANNELS),
            samples: None,
        };

        let device = audio
            .open_playback(None, &desired_spec, |spec| SquareWave {
                phase_inc: Self::PHASE_INC / spec.freq as f32,
                phase: Self::PHASE,
                volume: Self::VOLUME,
            })
            .unwrap();

        Audio { device }
    }

    pub fn on(&self) {
        if self.device.status() != AudioStatus::Playing {
            self.device.resume();
        }
    }

    pub fn off(&self) {
        if self.device.status() == AudioStatus::Playing {
            self.device.pause();
        };
    }
}
