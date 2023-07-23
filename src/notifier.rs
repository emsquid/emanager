use crate::{hypr::Hypr, stater::Stater};
use notify_rust::{Hint, Notification};

pub struct Notifier {
    stater: Stater<u32>,
}

impl Notifier {
    pub fn new(name: &str) -> Self {
        Self {
            stater: Stater::new(&format!("{name}.id")),
        }
    }

    pub fn send(&self, summary: &str, body: &str, value: Option<u32>) -> anyhow::Result<()> {
        if Hypr::running() {
            let color = format!("#{}ee", Hypr::get_color());
            let mut notif = Notification::new()
                .summary(summary)
                .body(body)
                .hint(Hint::Custom("frcolor".to_string(), color))
                .finalize();
            if let Some(value) = value {
                notif = notif
                    .hint(Hint::CustomInt("value".to_string(), value as i32))
                    .finalize();
            }
            let id = if let Ok(id) = self.stater.read() {
                notif.id(id).show()?.id()
            } else {
                notif.show()?.id()
            };
            self.stater.write(&id)?;
        }
        Ok(())
    }
}
