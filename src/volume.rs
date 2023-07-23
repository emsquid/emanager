use crate::notifier::Notifier;
use crate::stater::Stater;
use anyhow::anyhow;
use clap::Subcommand;
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::fmt::Display;
use std::process::{Command, Output};
use std::time::Duration;

const PROGRAM: &str = "wpctl";
const ID: &str = "@DEFAULT_AUDIO_SINK@";

#[derive(Serialize, Deserialize)]
struct VolumeState {
    volume: u32,
    icon: String,
    mute: bool,
}

impl VolumeState {
    pub fn new(volume: u32, icon: &str, mute: bool) -> Self {
        Self {
            volume,
            icon: icon.to_string(),
            mute,
        }
    }
}

pub struct Volume {
    notifier: Notifier,
    stater: Stater<VolumeState>,
}

impl Volume {
    pub fn new() -> Self {
        Self {
            notifier: Notifier::new(),
            stater: Stater::new("volume"),
        }
    }

    pub fn working(&mut self) -> anyhow::Result<bool> {
        Ok(self.exec(&["get-volume", ID])?.stderr.is_empty())
    }

    pub fn muted(&mut self) -> anyhow::Result<bool> {
        let string = String::from_utf8(self.exec(&["get-volume", ID])?.stdout)?;
        Ok(string.contains("MUTED"))
    }

    pub fn get(&mut self) -> anyhow::Result<u32> {
        if self.working()? {
            let string = String::from_utf8(self.exec(&["get-volume", ID])?.stdout)?;
            let volume = string.split(" ").collect::<Vec<&str>>()[1]
                .trim()
                .parse::<f32>()?;
            Ok((volume * 100.).round() as u32)
        } else {
            Ok(0)
        }
    }

    pub fn set(&mut self, percent: u32) -> anyhow::Result<()> {
        if self.working()? && !self.muted()? {
            self.exec(&["set-volume", ID, &format!("{percent}%"), "-l", "1"])?;
        }
        self.update(0)
    }

    pub fn up(&mut self) -> anyhow::Result<()> {
        if self.working()? && !self.muted()? {
            self.exec(&["set-volume", ID, "5%+", "-l", "1"])?;
        }
        self.update(0)
    }

    pub fn down(&mut self) -> anyhow::Result<()> {
        if self.working()? && !self.muted()? {
            self.exec(&["set-volume", ID, "5%-"])?;
        }
        self.update(0)
    }

    pub fn mute(&mut self) -> anyhow::Result<()> {
        if self.working()? {
            self.exec(&["set-mute", ID, "toggle"])?;
        }
        self.update(0)
    }

    pub fn update(&mut self, delay: u64) -> anyhow::Result<()> {
        std::thread::sleep(Duration::from_millis(delay));
        self.notify()?;
        self.state()
    }

    pub fn handle(&mut self, operation: VolumeOp) -> anyhow::Result<()> {
        match operation {
            VolumeOp::Up => self.up(),
            VolumeOp::Down => self.down(),
            VolumeOp::Set { percent } => self.set(percent),
            VolumeOp::Mute => self.mute(),
            VolumeOp::Update => self.update(200),
        }
    }

    pub fn notify(&mut self) -> anyhow::Result<()> {
        if !self.working()? {
            self.notifier.send("Volume", "No output", None)
        } else if self.muted()? {
            self.notifier.send("Volume", "Muted", None)
        } else {
            let value = self.get()?;
            self.notifier
                .send("Volume", &format!("Set to {}%", value), Some(value))
        }
    }

    pub fn state(&mut self) -> anyhow::Result<()> {
        let volume = self.get()?;
        let icon = if !self.working()? || self.muted()? {
            "󰝟 "
        } else if volume >= 40 {
            "󰕾 "
        } else if volume >= 20 {
            "󰖀 "
        } else {
            "󰕿 "
        };
        let state = VolumeState::new(volume, icon, self.muted()?);
        self.stater.write(state)
    }

    fn exec(&mut self, args: &[impl AsRef<OsStr>]) -> anyhow::Result<Output> {
        let output = Command::new(PROGRAM).args(args).output()?;
        Ok(output)
    }
}

#[derive(Copy, Clone, Subcommand)]
pub enum VolumeOp {
    /// Increase by 5%
    Up,
    /// Decrease by 5%
    Down,
    /// Set to a percentage
    Set {
        #[arg(value_parser = clap::value_parser!(u32).range(0..=100))]
        percent: u32,
    },
    /// Toggle mute
    Mute,
    /// Update status and notify
    Update,
}

impl Display for VolumeOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VolumeOp::Up => write!(f, "up"),
            VolumeOp::Down => write!(f, "down"),
            VolumeOp::Set { percent: value } => write!(f, "set {value}"),
            VolumeOp::Mute => write!(f, "mute"),
            VolumeOp::Update => write!(f, "update"),
        }
    }
}
impl TryFrom<String> for VolumeOp {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let operation = value.split(" ").collect::<Vec<&str>>();
        match operation.get(0) {
            Some(&"up") => Ok(VolumeOp::Up),
            Some(&"down") => Ok(VolumeOp::Down),
            Some(&"set") => match operation.get(1) {
                Some(percent) => Ok(VolumeOp::Set {
                    percent: percent.parse()?,
                }),
                None => Err(anyhow!("Missing percent for operation set")),
            },
            Some(&"mute") => Ok(VolumeOp::Mute),
            Some(&"update") => Ok(VolumeOp::Update),
            _ => Err(anyhow!("Unknown operation for volume: {value}")),
        }
    }
}
