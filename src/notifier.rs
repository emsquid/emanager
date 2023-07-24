use crate::{hypr::Hypr, logger::Logger};
use notify_rust::{Hint, Notification, Urgency};

pub struct Notifier {
    logger: Logger<u32>,
}

impl Notifier {
    pub fn new(name: &str) -> Self {
        Self {
            logger: Logger::new(&format!("{name}.id")),
        }
    }

    pub fn send(
        &self,
        summary: &str,
        body: &str,
        urgency: Option<Urgency>,
        value: Option<u32>,
    ) -> anyhow::Result<()> {
        if Hypr::running() {
            let color = format!("#{}ee", Hypr::get_color());
            let mut notif = Notification::new()
                .summary(summary)
                .body(body)
                .hint(Hint::Urgency(urgency.unwrap_or(Urgency::Normal)))
                .hint(Hint::Custom("frcolor".to_string(), color))
                .finalize();
            if let Some(value) = value {
                notif = notif
                    .hint(Hint::CustomInt("value".to_string(), value as i32))
                    .finalize();
            }
            let id = if let Ok(id) = self.logger.read() {
                notif.id(id).show()?.id()
            } else {
                notif.show()?.id()
            };
            self.logger.write(&id)?;
        }
        Ok(())
    }
}
