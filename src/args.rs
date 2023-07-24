use crate::brightness::BrightnessOp;
use crate::hypr::Layout;
use crate::system::SystemOp;
use crate::volume::VolumeOp;
use clap::{command, Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Launch manager daemon
    Daemon,
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
    /// Change layout
    Layout {
        #[arg(value_enum)]
        layout: Layout,
    },
}
