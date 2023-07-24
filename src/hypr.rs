use crate::logger::Logger;
use clap::ValueEnum;
use hyprland::data::{Client, Workspace, Workspaces};
use hyprland::keyword::Keyword;
use hyprland::shared::{HyprData, HyprDataActive, HyprDataActiveOptional};
use rand::seq::IteratorRandom;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::io::{BufRead, BufReader};
use std::os::unix::net::UnixStream;
use std::process::Command;
use std::time::Duration;

const COLORS: &[&str] = &["7aa2f7", "9ece6a", "e0af68", "bb9af7", "7dcfff", "c0caf5"];

pub struct Hypr;

impl Hypr {
    pub fn listen() -> anyhow::Result<()> {
        while !Self::running() {
            std::thread::sleep(Duration::from_secs(1));
        }

        Self::change_workspace()?;

        let stream = Self::stream()?;
        let reader = BufReader::new(stream);
        let mut current = Self::get_active_address()?;
        for line in reader.lines().flatten() {
            let address = Self::get_active_address()?;
            if line.starts_with("workspace") {
                Self::change_workspace()?;
            } else if line.starts_with("activewindowv2") && address != current {
                Self::change_color()?;
                current = address;
            }
        }

        Ok(())
    }

    pub fn running() -> bool {
        let pgrep = Command::new("pgrep").arg("Hyprland").output();
        pgrep.is_ok_and(|output| !output.stdout.is_empty())
    }

    pub fn change_layout(layout: Layout) -> anyhow::Result<()> {
        Keyword::set("input:kb_layout", layout.to_string())?;
        Logger::new("layout").write(&layout)
    }

    pub fn change_workspace() -> anyhow::Result<()> {
        let mut states = Workspaces::get()?
            .flat_map(WorkspaceState::try_from)
            .collect::<Vec<WorkspaceState>>();
        for id in 1..=5 {
            if states.iter().all(|state| state.id != id) {
                states.push(WorkspaceState::new(id, 0, false));
            }
        }
        states.sort_by_key(|workspace| workspace.id);
        Logger::new("workspaces").write(&states)
    }

    pub fn change_color() -> anyhow::Result<()> {
        let color = Self::rand_color();
        Keyword::set("general:col.active_border", format!("rgba({color}ee)"))?;
        Logger::new("color").write(&color)
    }

    pub fn get_color() -> String {
        // temporary fix because hyprctl doesn't work for colors
        match Logger::new("color").read() {
            Ok(color) => color,
            Err(_) => "7aa2f7".to_string(),
        }
    }

    fn get_active_address() -> anyhow::Result<Option<Vec<u8>>> {
        Ok(Client::get_active()?.map(|client| client.address.as_vec()))
    }

    fn stream() -> anyhow::Result<UnixStream> {
        let signature = std::env::var("HYPRLAND_INSTANCE_SIGNATURE")?;
        Ok(UnixStream::connect(format!(
            "/tmp/hypr/{signature}/.socket2.sock"
        ))?)
    }

    fn rand_color() -> String {
        (*COLORS
            .iter()
            .filter(|color| (**color).to_string() != Self::get_color())
            .choose(&mut rand::thread_rng())
            .unwrap())
        .to_string()
    }
}

#[derive(Serialize, Deserialize)]
pub struct WorkspaceState {
    id: i32,
    windows: u16,
    active: bool,
}

impl TryFrom<Workspace> for WorkspaceState {
    type Error = anyhow::Error;
    fn try_from(value: Workspace) -> Result<Self, Self::Error> {
        Ok(Self::new(
            value.id,
            value.windows,
            value.id == Workspace::get_active()?.id,
        ))
    }
}

impl WorkspaceState {
    pub fn new(id: i32, windows: u16, active: bool) -> Self {
        Self {
            id,
            windows,
            active,
        }
    }
}

#[derive(Serialize, Deserialize, Copy, Clone, ValueEnum)]
pub enum Layout {
    Fr,
    Us,
}

impl Display for Layout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Layout::Fr => write!(f, "fr"),
            Layout::Us => write!(f, "us"),
        }
    }
}
