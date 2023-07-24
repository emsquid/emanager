use crate::logger::Logger;
use crate::notifier::Notifier;
use anyhow::anyhow;
use battery::{units::ratio::percent, Battery as Batt, Manager, State};
use notify_rust::Urgency;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub struct Battery;

impl Battery {
    pub fn listen() -> anyhow::Result<()> {
        let (manager, mut battery) = Self::get_battery()?;
        let mut current = None;
        loop {
            let state = Self::get_state(&manager, &mut battery)?;
            if Some(&state) != current.as_ref() {
                state.notify()?;
                state.log()?;
                current = Some(state)
            }
            std::thread::sleep(Duration::from_secs(2));
        }
    }

    fn get_state(manager: &Manager, battery: &mut Batt) -> anyhow::Result<BatteryState> {
        manager.refresh(battery)?;
        let value = battery.state_of_charge().get::<percent>();
        let state = battery.state();
        Ok(BatteryState::new(value.round() as u32, state))
    }

    fn get_battery() -> anyhow::Result<(Manager, Batt)> {
        let manager = Manager::new()?;
        let battery = manager
            .batteries()?
            .flatten()
            .next()
            .ok_or(anyhow!("No battery found"))?;
        Ok((manager, battery))
    }
}

#[derive(Serialize, Deserialize, PartialEq)]
struct BatteryState {
    value: u32,
    status: String,
    icon: String,
}

impl BatteryState {
    pub fn new(value: u32, state: State) -> Self {
        let icon = if state == State::Charging {
            " "
        } else if state == State::Full || state == State::Unknown {
            " "
        } else if value >= 85 {
            " "
        } else if value >= 60 {
            " "
        } else if value >= 40 {
            " "
        } else if value >= 15 {
            " "
        } else {
            " "
        }
        .to_string();
        let status = match state {
            State::Unknown => "Not charging",
            State::Charging => "Charging",
            State::Discharging => "Discharging",
            State::Empty => "Empty",
            State::Full => "Full",
            _ => "Unknown",
        }
        .to_string();
        Self {
            value,
            status,
            icon,
        }
    }

    pub fn notify(&self) -> anyhow::Result<()> {
        if self.status != "Charging" && self.value <= 10 {
            Notifier::new("battery").send(
                "Battery very low",
                "Connect charger",
                Some(Urgency::Critical),
                None,
            )?;
        }
        Ok(())
    }

    pub fn log(&self) -> anyhow::Result<()> {
        Logger::new("battery").write(self)
    }
}
