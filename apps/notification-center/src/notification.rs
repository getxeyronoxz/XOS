use std::collections::HashMap;
use chrono::{DateTime, Local, Duration};

#[derive(Clone)]
pub struct Notification {
    pub id: u64,
    pub app_name: String,
    pub summary: String,
    pub body: String,
    pub timestamp: DateTime<Local>,
}

pub struct NotificationStore {
    notifications: Vec<Notification>,
    next_id: u64,
}

impl NotificationStore {
    pub fn new() -> Self {
        let mut store = Self {
            notifications: Vec::new(),
            next_id: 1,
        };
        store.prepopulate();
        store
    }

    fn prepopulate(&mut self) {
        let now = Local::now();
        self.add(
            "System".to_string(),
            "System Update Available".to_string(),
            "XOS update 2026.06.2 is ready to install.".to_string(),
            now - Duration::minutes(15),
        );
        self.add(
            "Firefox".to_string(),
            "Download completed".to_string(),
            "xos-spec-v4.pdf finished downloading.".to_string(),
            now - Duration::minutes(45),
        );
        self.add(
            "File Manager".to_string(),
            "Trash Emptied".to_string(),
            "Successfully cleared 2.4 GB of temporary storage.".to_string(),
            now - Duration::hours(2),
        );
        self.add(
            "System".to_string(),
            "Battery Low".to_string(),
            "Battery is at 15%. Performance mode switched to Battery Focus.".to_string(),
            now - Duration::hours(5),
        );
    }

    pub fn add(&mut self, app_name: String, summary: String, body: String, timestamp: DateTime<Local>) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.notifications.push(Notification {
            id,
            app_name,
            summary,
            body,
            timestamp,
        });
        id
    }

    pub fn remove(&mut self, id: u64) {
        self.notifications.retain(|n| n.id != id);
    }

    pub fn clear_all(&mut self) {
        self.notifications.clear();
    }

    pub fn get_all(&self) -> &[Notification] {
        &self.notifications
    }

    pub fn get_grouped(&self) -> HashMap<String, Vec<&Notification>> {
        let mut grouped = HashMap::new();
        for n in &self.notifications {
            grouped.entry(n.app_name.clone())
                .or_insert_with(Vec::new)
                .push(n);
        }
        grouped
    }
}
