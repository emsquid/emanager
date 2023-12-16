use crate::logger::Logger;
use crate::notifier::Notifier;
use crate::utils::utf8_to_u32;
use clap::Subcommand;
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::process::{Command, Output};
use std::time::Duration;

const PROGRAM: &str = "brightnessctl";

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

    pub fn up(percent: u32) -> anyhow::Result<()> {
        Self::exec(&["set", &format!("+{percent}%")])?;
        Self::update(0)
    }

    pub fn down(percent: u32) -> anyhow::Result<()> {
        Self::exec(&["set", &format!("{percent}%-")])?;
        Self::update(0)
    }

    pub fn update(delay: u64) -> anyhow::Result<()> {
        if delay != 0 {
            std::thread::sleep(Duration::from_millis(delay));
        }
        let state = BrightnessState::new(Self::get()?);
        state.notify()?;
        state.log()
    }

    pub fn handle(operation: BrightnessOp) -> anyhow::Result<()> {
        match operation {
            BrightnessOp::Up { percent } => Self::up(percent),
            BrightnessOp::Down { percent } => Self::down(percent),
            BrightnessOp::Set { percent } => Self::set(percent),
            BrightnessOp::Update => Self::update(500),
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
    /// Increase by percentage
    Up {
        #[arg(default_value_t = 5, value_parser = clap::value_parser!(u32).range(0..=100))]
        percent: u32,
    },
    /// Decrease by percentage
    Down {
        #[arg(default_value_t = 5, value_parser = clap::value_parser!(u32).range(0..=100))]
        percent: u32,
    },
    /// Set to a percentage
    Set {
        #[arg(value_parser = clap::value_parser!(u32).range(0..=100))]
        percent: u32,
    },
    /// Update status and notify
    Update,
}

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
            None,
            Some(self.value),
        )
    }

    pub fn log(&self) -> anyhow::Result<()> {
        Logger::new("brightness").write(self)
    }
}
