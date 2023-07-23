use anyhow::anyhow;
use clap::Subcommand;
use std::{fmt::Display, process::Command};

#[derive(Clone)]
pub struct System;

impl System {
    pub fn poweroff() -> anyhow::Result<()> {
        Self::exec("poweroff")
    }

    pub fn reboot() -> anyhow::Result<()> {
        Self::exec("reboot")
    }

    pub fn suspend() -> anyhow::Result<()> {
        Self::exec("suspend").and_then(|()| Self::lock())
    }

    pub fn lock() -> anyhow::Result<()> {
        Command::new("pkill").arg("swaylock").output()?;
        Command::new("swaylock").arg("-f").output()?;
        Ok(())
    }

    pub fn handle(operation: SystemOp) -> anyhow::Result<()> {
        match operation {
            SystemOp::Poweroff => Self::poweroff(),
            SystemOp::Reboot => Self::reboot(),
            SystemOp::Suspend => Self::suspend(),
            SystemOp::Lock => Self::lock(),
        }
    }

    fn exec(command: &str) -> anyhow::Result<()> {
        Command::new("systemctl").arg(command).output()?;
        Ok(())
    }
}

#[derive(Copy, Clone, Subcommand)]
pub enum SystemOp {
    /// Turn system off
    Poweroff,
    /// Reboot system
    Reboot,
    /// Suspend system
    Suspend,
    /// Lock system
    Lock,
}

impl Display for SystemOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SystemOp::Poweroff => write!(f, "poweroff"),
            SystemOp::Reboot => write!(f, "reboot"),
            SystemOp::Suspend => write!(f, "suspend"),
            SystemOp::Lock => write!(f, "lock"),
        }
    }
}

impl TryFrom<String> for SystemOp {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let operation = value.split(" ").collect::<Vec<&str>>();
        match operation.get(0) {
            Some(&"poweroff") => Ok(SystemOp::Poweroff),
            Some(&"reboot") => Ok(SystemOp::Reboot),
            Some(&"suspend") => Ok(SystemOp::Suspend),
            Some(&"lock") => Ok(SystemOp::Lock),
            _ => Err(anyhow!("Unknown operation for systemd: {value}")),
        }
    }
}
