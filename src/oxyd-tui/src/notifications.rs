use chrono::{DateTime, Utc};
use std::collections::VecDeque;

const MAX_NOTIFICATIONS: usize = 50;

#[derive(Debug, Clone, PartialEq)]
pub enum NotificationLevel {
    Info,
    Warning,
    Critical,
    Success,
}

#[derive(Debug, Clone)]
pub struct Notification {
    pub id: usize,
    pub level: NotificationLevel,
    pub title: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub read: bool,
}

impl Notification {
    pub fn new(level: NotificationLevel, title: String, message: String) -> Self {
        static mut COUNTER: usize = 0;
        let id = unsafe {
            COUNTER += 1;
            COUNTER
        };

        Self {
            id,
            level,
            title,
            message,
            timestamp: Utc::now(),
            read: false,
        }
    }

    pub fn info(title: String, message: String) -> Self {
        Self::new(NotificationLevel::Info, title, message)
    }

    pub fn warning(title: String, message: String) -> Self {
        Self::new(NotificationLevel::Warning, title, message)
    }

    pub fn critical(title: String, message: String) -> Self {
        Self::new(NotificationLevel::Critical, title, message)
    }

    pub fn success(title: String, message: String) -> Self {
        Self::new(NotificationLevel::Success, title, message)
    }
}

#[derive(Debug, Clone)]
pub struct NotificationManager {
    notifications: VecDeque<Notification>,
    unread_count: usize,
}

impl NotificationManager {
    pub fn new() -> Self {
        Self {
            notifications: VecDeque::new(),
            unread_count: 0,
        }
    }

    pub fn add(&mut self, notification: Notification) {
        if self.notifications.len() >= MAX_NOTIFICATIONS {
            if let Some(removed) = self.notifications.pop_front() {
                if !removed.read {
                    self.unread_count = self.unread_count.saturating_sub(1);
                }
            }
        }
        
        self.unread_count += 1;
        self.notifications.push_back(notification);
    }

    pub fn add_info(&mut self, title: String, message: String) {
        self.add(Notification::info(title, message));
    }

    pub fn add_warning(&mut self, title: String, message: String) {
        self.add(Notification::warning(title, message));
    }

    pub fn add_critical(&mut self, title: String, message: String) {
        self.add(Notification::critical(title, message));
    }

    pub fn add_success(&mut self, title: String, message: String) {
        self.add(Notification::success(title, message));
    }

    pub fn mark_read(&mut self, id: usize) {
        if let Some(notif) = self.notifications.iter_mut().find(|n| n.id == id) {
            if !notif.read {
                notif.read = true;
                self.unread_count = self.unread_count.saturating_sub(1);
            }
        }
    }

    pub fn mark_all_read(&mut self) {
        for notif in &mut self.notifications {
            notif.read = true;
        }
        self.unread_count = 0;
    }

    pub fn clear(&mut self) {
        self.notifications.clear();
        self.unread_count = 0;
    }

    pub fn get_all(&self) -> Vec<&Notification> {
        self.notifications.iter().collect()
    }

    pub fn get_unread(&self) -> Vec<&Notification> {
        self.notifications.iter().filter(|n| !n.read).collect()
    }

    pub fn unread_count(&self) -> usize {
        self.unread_count
    }

    pub fn total_count(&self) -> usize {
        self.notifications.len()
    }
}

impl Default for NotificationManager {
    fn default() -> Self {
        Self::new()
    }
}
