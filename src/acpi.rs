use crate::args::Command;
use crate::brightness::BrightnessOp;
use crate::manager::Manager;
use crate::systemd::SystemOp;
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
                let event = line.split(" ").collect::<Vec<&str>>();
                Self::handle(event)?;
                last = Instant::now();
            }
        }

        Ok(())
    }

    fn handle(event: Vec<&str>) -> anyhow::Result<()> {
        match event[0] {
            "button/lid" => match event[2] {
                "close" => Some(Command::System {
                    operation: SystemOp::Suspend,
                }),
                _ => None,
            },
            "button/sleep" => Some(Command::System {
                operation: SystemOp::Suspend,
            }),
            "video/brightnessup" => Some(Command::Brightness {
                operation: BrightnessOp::Up,
            }),
            "video/brightnessdown" => Some(Command::Brightness {
                operation: BrightnessOp::Down,
            }),
            "button/volumeup" => Some(Command::Volume {
                operation: VolumeOp::Up,
            }),
            "button/volumedown" => Some(Command::Volume {
                operation: VolumeOp::Down,
            }),
            "button/mute" => Some(Command::Volume {
                operation: VolumeOp::Mute,
            }),
            "jack/headphone" => Some(Command::Volume {
                operation: VolumeOp::Update,
            }),
            _ => None,
        }
        .map_or(Ok(()), Manager::handle)
    }
}
