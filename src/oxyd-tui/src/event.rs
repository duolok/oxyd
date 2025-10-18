use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyEvent};
use std::time::Duration;
use tokio::sync::mpsc;

pub enum Event {
    Key(KeyEvent),
    Tick,
    Resize,
}

pub struct EventHandler {
    tx: mpsc::UnboundedSender<Event>,
    rx: mpsc::UnboundedReceiver<Event>,
}

impl EventHandler {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self { tx , rx }
    }

    pub async fn next(&mut self) -> Option<Event> {
        self.rx.recv().await
    }

    pub fn sender(&self) -> mpsc::UnboundedSender<Event> {
        self.tx.clone()
    }

    pub async fn start_polling(&self) {
        let tx = self.tx.clone();
        tokio::spawn(async move {
            loop {
                if event::poll(Duration::from_millis(100)).unwrap_or(false) {
                    match event::read() {
                        Ok(CrosstermEvent::Key(key)) => {
                            let _ = tx.send(Event::Key(key));
                        }
                        Ok(CrosstermEvent::Resize(_, _)) => {
                            let _ = tx.send(Event::Resize);
                        }
                        _ => {}
                    }
                }
            }
        });

        let tx = self.tx.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(250));
            loop {
                interval.tick().await;
                let _ = tx.send(Event::Tick);
            }
        });
    }
}

pub fn map_key_to_action(key: KeyEvent) -> Option<crate::app::Action> {
    use crate::app::Action;
    use crossterm::event::KeyCode;

    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => Some(Action::Quit),
        KeyCode::Char('?') => Some(Action::ToggleHelp),
        KeyCode::Tab => Some(Action::NextTab),
        KeyCode::BackTab => Some(Action::PreviousTab),
        KeyCode::Char('1') => Some(Action::SwitchTab(crate::tabs::Tab::Overview)),
        KeyCode::Char('2') => Some(Action::SwitchTab(crate::tabs::Tab::Cpu)),
        KeyCode::Char('3') => Some(Action::SwitchTab(crate::tabs::Tab::Memory)),
        KeyCode::Char('4') => Some(Action::SwitchTab(crate::tabs::Tab::Processes)),
        KeyCode::Char('5') => Some(Action::SwitchTab(crate::tabs::Tab::Network)),
        KeyCode::Char('6') => Some(Action::SwitchTab(crate::tabs::Tab::Disk)),
        KeyCode::Char('7') => Some(Action::SwitchTab(crate::tabs::Tab::Notifications)),
        KeyCode::Up | KeyCode::Char('k') => Some(Action::ScrollUp),
        KeyCode::Down | KeyCode::Char('j') => Some(Action::ScrollDown),
        KeyCode::PageUp => Some(Action::PageUp),
        KeyCode::PageDown => Some(Action::PageDown),
        KeyCode::Home => Some(Action::Home),
        KeyCode::End => Some(Action::End),
        
        KeyCode::Char('K') => Some(Action::KillSelectedProcess),
        KeyCode::Char('s') => Some(Action::SuspendSelectedProcess),
        KeyCode::Char('c') => Some(Action::ContinueSelectedProcess),
        KeyCode::Char('t') => Some(Action::TerminateSelectedProcess),
        KeyCode::Char('r') => Some(Action::LoadProcessList),
        
        KeyCode::Char('p') => Some(Action::SortByColumn(0)), 
        KeyCode::Char('n') => Some(Action::SortByColumn(1)), 
        KeyCode::Char('C') => Some(Action::SortByColumn(2)), 
        KeyCode::Char('M') => Some(Action::SortByColumn(3)), 
        
        KeyCode::Char('m') => Some(Action::MarkAllNotificationsRead),
        KeyCode::Char('x') => Some(Action::ClearAllNotifications),
        
        _ => None,
    }
}
