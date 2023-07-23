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
    value: u32,
    icon: String,
}

impl BrightnessState {
    pub fn new(value: u32) -> Self {
        let icon = if value >= 89 {
            " "
        } else if value >= 78 {
            " "
        } else if value >= 67 {
            " "
        } else if value >= 56 {
            " "
        } else if value >= 45 {
            " "
        } else if value >= 34 {
            " "
        } else if value >= 23 {
            " "
        } else if value >= 12 {
            " "
        } else {
            " "
        }
        .to_string();
        Self { value, icon }
    }

    pub fn notify(&self) -> anyhow::Result<()> {
        Notifier::new("brightness").send(
            "Brightness",
            &format!("Set to {}%", self.value),
            Some(self.value),
        )
    }

    pub fn state(&self) -> anyhow::Result<()> {
        Stater::new("brightness").write(self)
    }
}

pub struct Brightness;

impl Brightness {
    pub fn get() -> anyhow::Result<u32> {
        let value = utf8_to_u32(Self::exec(&["get"])?.stdout)?;
        let percent = value as f32 * 100. / Self::max()? as f32;
        Ok(percent.round() as u32)
    }

    pub fn set(percent: u32) -> anyhow::Result<()> {
        Self::exec(&["set", &format!("{percent}%")])?;
        Self::update(0)
    }

    pub fn up() -> anyhow::Result<()> {
        Self::exec(&["set", "+5%"])?;
        Self::update(0)
    }

    pub fn down() -> anyhow::Result<()> {
        Self::exec(&["set", "5%-"])?;
        Self::update(0)
    }

    pub fn update(delay: u64) -> anyhow::Result<()> {
        if delay != 0 {
            std::thread::sleep(Duration::from_millis(delay));
        }
        let state = BrightnessState::new(Self::get()?);
        state.notify()?;
        state.state()
    }

    pub fn handle(operation: BrightnessOp) -> anyhow::Result<()> {
        match operation {
            BrightnessOp::Up => Self::up(),
            BrightnessOp::Down => Self::down(),
            BrightnessOp::Set { percent } => Self::set(percent),
            BrightnessOp::Update => Self::update(200),
        }
    }

    fn max() -> anyhow::Result<u32> {
        let value = utf8_to_u32(Self::exec(&["max"])?.stdout)?;
        Ok(value)
    }

    fn exec(args: &[impl AsRef<OsStr>]) -> anyhow::Result<Output> {
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
