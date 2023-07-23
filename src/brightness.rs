use crate::notifier::Notifier;
use crate::stater::Stater;
use crate::utils::utf8_to_u32;
use anyhow::anyhow;
use clap::Subcommand;
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::fmt::Display;
use std::process::{Command, Output};
use std::time::Duration;

const PROGRAM: &str = "brightnessctl";

#[derive(Serialize, Deserialize)]
struct BrightnessState {
    brightness: u32,
    icon: String,
}

impl BrightnessState {
    pub fn new(brightness: u32, icon: &str) -> Self {
        Self {
            brightness,
            icon: icon.to_string(),
        }
    }
}

pub struct Brightness {
    notifier: Notifier,
    stater: Stater<BrightnessState>,
}

impl Brightness {
    pub fn new() -> Self {
        Self {
            notifier: Notifier::new(),
            stater: Stater::new("brightness"),
        }
    }

    pub fn get(&mut self) -> anyhow::Result<u32> {
        let value = utf8_to_u32(self.exec(&["get"])?.stdout)?;
        let percent = value as f32 * 100. / self.max()? as f32;
        Ok(percent.round() as u32)
    }

    pub fn set(&mut self, percent: u32) -> anyhow::Result<()> {
        self.exec(&["set", &format!("{percent}%")])?;
        self.update(0)
    }

    pub fn up(&mut self) -> anyhow::Result<()> {
        let percent = self.get()?;
        self.set(percent + 5)
    }

    pub fn down(&mut self) -> anyhow::Result<()> {
        let percent = self.get()?;
        self.set(percent - 5)
    }

    pub fn update(&mut self, delay: u64) -> anyhow::Result<()> {
        std::thread::sleep(Duration::from_millis(delay));
        self.notify()?;
        self.state()
    }

    pub fn handle(&mut self, operation: BrightnessOp) -> anyhow::Result<()> {
        match operation {
            BrightnessOp::Up => self.up(),
            BrightnessOp::Down => self.down(),
            BrightnessOp::Set { percent } => self.set(percent),
            BrightnessOp::Update => self.update(200),
        }
    }

    pub fn notify(&mut self) -> anyhow::Result<()> {
        let value = self.get()?;
        self.notifier
            .send("Brightness", &format!("Set to {}%", value), Some(value))
    }

    pub fn state(&mut self) -> anyhow::Result<()> {
        let brightness = self.get()?;
        let icon = if brightness >= 89 {
            " "
        } else if brightness >= 78 {
            " "
        } else if brightness >= 67 {
            " "
        } else if brightness >= 56 {
            " "
        } else if brightness >= 45 {
            " "
        } else if brightness >= 34 {
            " "
        } else if brightness >= 23 {
            " "
        } else if brightness >= 12 {
            " "
        } else {
            " "
        };
        let state = BrightnessState::new(brightness, icon);
        self.stater.write(state)
    }

    fn max(&mut self) -> anyhow::Result<u32> {
        let value = utf8_to_u32(self.exec(&["max"])?.stdout)?;
        Ok(value)
    }

    fn exec(&mut self, args: &[impl AsRef<OsStr>]) -> anyhow::Result<Output> {
        let output = Command::new(PROGRAM).args(args).output()?;
        Ok(output)
    }
}

#[derive(Copy, Clone, Subcommand)]
pub enum BrightnessOp {
    /// Increase by 5%
    Up,
    /// Decrease by 5%
    Down,
    /// Set to a percentage
    Set {
        #[arg(value_parser = clap::value_parser!(u32).range(0..=100))]
        percent: u32,
    },
    /// Update status and notify
    Update,
}

impl Display for BrightnessOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BrightnessOp::Up => write!(f, "up"),
            BrightnessOp::Down => write!(f, "down"),
            BrightnessOp::Set { percent: value } => write!(f, "set {value}"),
            BrightnessOp::Update => write!(f, "update"),
        }
    }
}
impl TryFrom<String> for BrightnessOp {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let operation = value.split(" ").collect::<Vec<&str>>();
        match operation.get(0) {
            Some(&"up") => Ok(BrightnessOp::Up),
            Some(&"down") => Ok(BrightnessOp::Down),
            Some(&"set") => match operation.get(1) {
                Some(percent) => Ok(BrightnessOp::Set {
                    percent: percent.parse()?,
                }),
                None => Err(anyhow!("Missing percent for operation set")),
            },
            Some(&"update") => Ok(BrightnessOp::Update),
            _ => Err(anyhow!("Unknown operation for brightness: {value}")),
        }
    }
}
