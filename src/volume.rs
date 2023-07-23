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
    value: u32,
    muted: bool,
    working: bool,
    icon: String,
}

impl VolumeState {
    pub fn new(value: u32, muted: bool, working: bool) -> Self {
        let icon = if !working || muted {
            "󰝟 "
        } else if value >= 40 {
            "󰕾 "
        } else if value >= 20 {
            "󰖀 "
        } else {
            "󰕿 "
        }
        .to_string();
        Self {
            value,
            muted,
            working,
            icon,
        }
    }

    pub fn notify(&self) -> anyhow::Result<()> {
        let notifier = Notifier::new("volume");
        if !self.working {
            notifier.send("Volume", "No output", None)
        } else if self.muted {
            notifier.send("Volume", "Muted", None)
        } else {
            notifier.send(
                "Volume",
                &format!("Set to {}%", self.value),
                Some(self.value),
            )
        }
    }

    pub fn state(&self) -> anyhow::Result<()> {
        Stater::new("volume").write(self)
    }
}

pub struct Volume;

impl Volume {
    pub fn working() -> anyhow::Result<bool> {
        Ok(Self::exec(&["get-volume", ID])?.stderr.is_empty())
    }

    pub fn muted() -> anyhow::Result<bool> {
        let string = String::from_utf8(Self::exec(&["get-volume", ID])?.stdout)?;
        Ok(string.contains("MUTED"))
    }

    pub fn get() -> anyhow::Result<u32> {
        if Self::working()? {
            let string = String::from_utf8(Self::exec(&["get-volume", ID])?.stdout)?;
            let volume = string.split(" ").collect::<Vec<&str>>()[1]
                .trim()
                .parse::<f32>()?;
            Ok((volume * 100.).round() as u32)
        } else {
            Ok(0)
        }
    }

    pub fn set(percent: u32) -> anyhow::Result<()> {
        if !Self::muted()? {
            Self::exec(&["set-volume", ID, &format!("{percent}%"), "-l", "1"])?;
        }
        Self::update(0)
    }

    pub fn up() -> anyhow::Result<()> {
        if !Self::muted()? {
            Self::exec(&["set-volume", ID, "5%+", "-l", "1"])?;
        }
        Self::update(0)
    }

    pub fn down() -> anyhow::Result<()> {
        if !Self::muted()? {
            Self::exec(&["set-volume", ID, "5%-"])?;
        }
        Self::update(0)
    }

    pub fn mute() -> anyhow::Result<()> {
        Self::exec(&["set-mute", ID, "toggle"])?;
        Self::update(0)
    }

    pub fn update(delay: u64) -> anyhow::Result<()> {
        if delay != 0 {
            std::thread::sleep(Duration::from_millis(delay))
        };
        let (working, muted, value) = (Self::working()?, Self::muted()?, Self::get()?);
        let state = VolumeState::new(value, muted, working);
        state.notify()?;
        state.state()
    }

    pub fn handle(operation: VolumeOp) -> anyhow::Result<()> {
        match operation {
            VolumeOp::Up => Self::up(),
            VolumeOp::Down => Self::down(),
            VolumeOp::Set { percent } => Self::set(percent),
            VolumeOp::Mute => Self::mute(),
            VolumeOp::Update => Self::update(200),
        }
    }

    fn exec(args: &[impl AsRef<OsStr>]) -> anyhow::Result<Output> {
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
