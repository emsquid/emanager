use crate::acpi::Acpi;
use crate::args::Command;
use crate::brightness::Brightness;
use crate::hypr::Hypr;
use crate::systemd::System;
use crate::volume::Volume;
use crate::wifi::Wifi;
use anyhow::anyhow;

pub struct Manager;

impl Manager {
    pub fn daemon() -> anyhow::Result<()> {
        if Self::running() {
            return Err(anyhow!("Manager is already running"));
        }
        std::thread::scope(|scope| -> anyhow::Result<()> {
            let handle = scope.spawn(move || Acpi::listen());
            scope.spawn(move || Hypr::listen());
            scope.spawn(move || Wifi::listen());

            handle.join().unwrap()
        })
    }

    pub fn handle(command: Command) -> anyhow::Result<()> {
        match command {
            Command::System { operation } => System::handle(operation),
            Command::Brightness { operation } => Brightness::handle(operation),
            Command::Volume { operation } => Volume::handle(operation),
            _ => Ok(()),
        }
    }

    pub fn running() -> bool {
        let pgrep = std::process::Command::new("pgrep").arg("emanager").output();
        pgrep.is_ok_and(|output| {
            String::from_utf8(output.stdout).is_ok_and(|stdout| stdout.lines().count() > 1)
        })
    }
}
