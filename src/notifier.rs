use crate::hypr::Hypr;
use notify_rust::{Hint, Notification};

#[derive(Clone)]
pub struct Notifier {
    id: Option<u32>,
}

impl Notifier {
    pub fn new() -> Self {
        Self { id: None }
    }

    pub fn send(&mut self, summary: &str, body: &str, value: Option<u32>) -> anyhow::Result<()> {
        if Hypr::running() {
            let color = format!("#{}", Hypr::get_color());
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
            self.id = Some(if let Some(id) = self.id {
                notif.id(id).show()?.id()
            } else {
                notif.show()?.id()
            });
        }
        Ok(())
    }
}
