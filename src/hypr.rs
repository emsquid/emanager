use crate::stater::Stater;
use hyprland::data::{Workspace, Workspaces};
use hyprland::event_listener::EventListener;
use hyprland::keyword::Keyword;
use hyprland::shared::{HyprData, HyprDataActive};
use rand::seq::IteratorRandom;
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::time::Duration;

const COLORS: &[&str] = &["7aa2f7", "9ece6a", "e0af68", "bb9af7", "7dcfff", "c0caf5"];

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

pub struct Hypr;

impl Hypr {
    pub fn listen() -> anyhow::Result<()> {
        while !Self::running() {
            std::thread::sleep(Duration::from_secs(1));
        }

        let mut listener = EventListener::new();
        listener.add_workspace_change_handler(|_| Self::change_workspace().unwrap());
        listener.add_active_window_change_handler(|_| Self::change_color().unwrap());

        Ok(listener.start_listener()?)
    }

    pub fn running() -> bool {
        Command::new("pgrep")
            .arg("Hyprland")
            .output()
            .is_ok_and(|output| !output.stdout.is_empty())
    }

    pub fn change_workspace() -> anyhow::Result<()> {
        let mut states = Workspaces::get()?
            .flat_map(WorkspaceState::try_from)
            .collect::<Vec<WorkspaceState>>();
        for id in 1..=5 {
            if states.iter().all(|state| state.id != id) {
                states.push(WorkspaceState::new(id, 0, false))
            }
        }
        states.sort_by_key(|workspace| workspace.id);
        Stater::new("workspaces").write(states)
    }

    pub fn change_color() -> anyhow::Result<()> {
        let color = Self::rand_color();
        Keyword::set("general:col.active_border", format!("rgba({color}ee)"))?;
        Command::new("eww")
            .args(["update", &format!("border-color=#{color}")])
            .output()?;
        Stater::new("color").write(color)
    }

    pub fn get_color() -> String {
        // temporary fix because hyprctl doesn't work for colors
        match Stater::new("color").read() {
            Ok(color) => color,
            Err(_) => "7aa2f7".to_string(),
        }
    }

    fn rand_color() -> String {
        COLORS
            .iter()
            .filter(|color| color.to_string() != Self::get_color())
            .choose(&mut rand::thread_rng())
            .unwrap()
            .to_string()
    }
}
