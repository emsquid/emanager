use crate::brightness::BrightnessOp;
use crate::systemd::SystemOp;
use crate::volume::VolumeOp;
use anyhow::anyhow;
use clap::{command, Parser, Subcommand};
use std::fmt::Display;

#[derive(Parser)]
#[command(author, version, about)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Launch manager
    Start,
    /// Commands to manage systemd
    System {
        #[command(subcommand)]
        operation: SystemOp,
    },
    /// Commands to manage backlight
    Brightness {
        #[command(subcommand)]
        operation: BrightnessOp,
    },
    /// Commands to manage volume
    Volume {
        #[command(subcommand)]
        operation: VolumeOp,
    },
}

impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::Start => write!(f, "start"),
            Command::System { operation: op } => write!(f, "system {op}"),
            Command::Brightness { operation: op } => write!(f, "brightness {op}"),
            Command::Volume { operation: op } => write!(f, "volume {op}"),
        }
    }
}

impl TryFrom<String> for Command {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let command = value.split(" ").collect::<Vec<&str>>();
        match command.get(0) {
            Some(&"start") => Ok(Command::Start),
            Some(&"system") => match command.get(1..) {
                Some(subcommand) => match SystemOp::try_from(subcommand.join(" ")) {
                    Ok(operation) => Ok(Command::System { operation }),
                    Err(e) => Err(anyhow!(e)),
                },
                None => Err(anyhow!("Missing operation for systemd")),
            },
            Some(&"brightness") => match command.get(1..) {
                Some(subcommand) => match BrightnessOp::try_from(subcommand.join(" ")) {
                    Ok(operation) => Ok(Command::Brightness { operation }),
                    Err(e) => Err(anyhow!(e)),
                },
                None => Err(anyhow!("Missing operation for brightness")),
            },
            Some(&"volume") => match command.get(1..) {
                Some(subcommand) => match VolumeOp::try_from(subcommand.join(" ")) {
                    Ok(operation) => Ok(Command::Volume { operation }),
                    Err(e) => Err(anyhow!(e)),
                },
                None => Err(anyhow!("Missing operation for volume")),
            },
            _ => Err(anyhow!("Unknown command: {value}")),
        }
    }
}
