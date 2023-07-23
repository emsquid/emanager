use crate::acpi::Acpi;
use crate::args::Command;
use crate::brightness::Brightness;
use crate::hypr::Hypr;
use crate::systemd::System;
use crate::volume::Volume;
use anyhow::anyhow;
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};

const SOCKET: &str = "/tmp/emanager.socket";

pub struct Manager {
    systemd: System,
    brightness: Brightness,
    volume: Volume,
}

impl Manager {
    pub fn new() -> Self {
        Self {
            systemd: System::new(),
            brightness: Brightness::new(),
            volume: Volume::new(),
        }
    }

    pub fn start(&mut self) -> anyhow::Result<()> {
        if Self::running() {
            return Err(anyhow!("Manager is already running"));
        }
        if std::fs::metadata(SOCKET).is_ok() {
            std::fs::remove_file(SOCKET)?;
        }
        let listener = UnixListener::bind(SOCKET)?;

        std::thread::scope(|scope| -> anyhow::Result<()> {
            let handle = scope.spawn(move || self.listen(listener));
            scope.spawn(move || Acpi::listen());
            scope.spawn(move || Hypr::listen());

            handle.join().unwrap()
        })
    }

    fn listen(&mut self, listener: UnixListener) -> anyhow::Result<()> {
        loop {
            let (mut stream, _) = listener.accept()?;
            let mut message = String::new();
            stream.read_to_string(&mut message)?;

            if !message.is_empty() {
                self.handle(message.try_into()?)?;
            }
        }
    }

    fn handle(&mut self, command: Command) -> anyhow::Result<()> {
        match command {
            Command::System { operation } => self.systemd.handle(operation),
            Command::Brightness { operation } => self.brightness.handle(operation),
            Command::Volume { operation } => self.volume.handle(operation),
            _ => Ok(()),
        }
    }

    pub fn send(command: Command) -> anyhow::Result<()> {
        if !Self::running() {
            return Err(anyhow!("Manager is not running"));
        }
        let mut stream = UnixStream::connect(SOCKET)?;
        stream.write_all(command.to_string().as_bytes())?;
        Ok(())
    }

    pub fn running() -> bool {
        UnixStream::connect(SOCKET).is_ok()
    }
}
