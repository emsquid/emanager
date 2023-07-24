use crate::args::Command;
use crate::brightness::BrightnessOp;
use crate::manager::Manager;
use crate::system::SystemOp;
use crate::volume::VolumeOp;
use std::io::{BufRead, BufReader};
use std::os::unix::net::UnixStream;
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct Acpi;

impl Acpi {
    pub fn listen() -> anyhow::Result<()> {
        let stream = UnixStream::connect("/run/acpid.socket")?;
        let reader = BufReader::new(stream);

        let delay = Duration::from_micros(100);
        let mut last = Instant::now();
        for line in reader.lines().flatten() {
            if last.elapsed() >= delay {
                let event = line.split(' ').collect::<Vec<&str>>();
                Self::handle(&event)?;
                last = Instant::now();
            }
        }

        Ok(())
    }

    fn handle(event: &[&str]) -> anyhow::Result<()> {
        match event.get(0) {
            Some(&"button/lid") => match event.get(2) {
                Some(&"close") => Some(Command::System {
                    operation: SystemOp::Suspend,
                }),
                _ => None,
            },
            Some(&"button/sleep") => Some(Command::System {
                operation: SystemOp::Suspend,
            }),
            Some(&"video/brightnessup") => Some(Command::Brightness {
                operation: BrightnessOp::Up { percent: 5 },
            }),
            Some(&"video/brightnessdown") => Some(Command::Brightness {
                operation: BrightnessOp::Down { percent: 5 },
            }),
            Some(&"button/volumeup") => Some(Command::Volume {
                operation: VolumeOp::Up { percent: 5 },
            }),
            Some(&"button/volumedown") => Some(Command::Volume {
                operation: VolumeOp::Down { percent: 5 },
            }),
            Some(&"button/mute") => Some(Command::Volume {
                operation: VolumeOp::Mute,
            }),
            Some(&"jack/headphone") => Some(Command::Volume {
                operation: VolumeOp::Update,
            }),
            _ => None,
        }
        .map_or(Ok(()), Manager::handle)
    }
}
