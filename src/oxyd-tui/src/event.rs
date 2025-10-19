use crate::app::{Action, InputMode};
use crate::tabs::Tab;
use crossterm::event::{self, Event as CrosstermEvent, KeyEvent};
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
        Self { tx, rx }
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

pub fn map_key_to_action(key: KeyEvent, input_mode: InputMode, current_tab: Tab) -> Option<Action> {
    use crossterm::event::KeyCode;

    if input_mode != InputMode::Normal {
        match key.code {
            KeyCode::Enter => return Some(Action::InputSubmit),
            KeyCode::Esc => return Some(Action::ExitInputMode),
            KeyCode::Backspace => return Some(Action::InputBackspace),
            KeyCode::Char(c) => return Some(Action::InputChar(c)),
            _ => return None,
        }
    }

    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => return Some(Action::Quit),
        KeyCode::Char('?') => return Some(Action::ToggleHelp),
        KeyCode::Tab => return Some(Action::NextTab),
        KeyCode::BackTab => return Some(Action::PreviousTab),
        KeyCode::Char('1') => return Some(Action::SwitchTab(Tab::Overview)),
        KeyCode::Char('2') => return Some(Action::SwitchTab(Tab::Cpu)),
        KeyCode::Char('3') => return Some(Action::SwitchTab(Tab::Memory)),
        KeyCode::Char('4') => return Some(Action::SwitchTab(Tab::Processes)),
        KeyCode::Char('5') => return Some(Action::SwitchTab(Tab::Network)),
        KeyCode::Char('6') => return Some(Action::SwitchTab(Tab::Disk)),
        KeyCode::Char('7') | KeyCode::Char('n') => {
            return Some(Action::SwitchTab(Tab::Notifications));
        }
        KeyCode::Char('8') | KeyCode::Char('z') => return Some(Action::SwitchTab(Tab::Settings)),
        KeyCode::Up | KeyCode::Char('k') => return Some(Action::ScrollUp),
        KeyCode::Down | KeyCode::Char('j') => return Some(Action::ScrollDown),
        KeyCode::PageUp => return Some(Action::PageUp),
        KeyCode::PageDown => return Some(Action::PageDown),
        KeyCode::Home => return Some(Action::Home),
        KeyCode::End => return Some(Action::End),
        _ => {}
    }

    match current_tab {
        Tab::Processes => match key.code {
            KeyCode::Char('K') => Some(Action::KillSelectedProcess),
            KeyCode::Char('s') => Some(Action::SuspendSelectedProcess),
            KeyCode::Char('c') => Some(Action::ContinueSelectedProcess),
            KeyCode::Char('t') => Some(Action::TerminateSelectedProcess),
            KeyCode::Char('r') => Some(Action::LoadProcessList),
            KeyCode::Char('p') => Some(Action::SortByColumn(0)),
            KeyCode::Char('n') => Some(Action::SortByColumn(1)),
            KeyCode::Char('C') => Some(Action::SortByColumn(2)),
            KeyCode::Char('M') => Some(Action::SortByColumn(3)),
            KeyCode::Char('/') => Some(Action::EnterInputMode(InputMode::SearchProcess)),
            KeyCode::Char('x') => Some(Action::ClearFilter),
            _ => None,
        },
        Tab::Notifications => match key.code {
            KeyCode::Char('m') => Some(Action::MarkAllNotificationsRead),
            KeyCode::Char('x') => Some(Action::ClearAllNotifications),
            _ => None,
        },
        Tab::Settings => match key.code {
            KeyCode::Char('c') => Some(Action::EnterInputMode(InputMode::EditCpuThreshold)),
            KeyCode::Char('m') => Some(Action::EnterInputMode(InputMode::EditMemoryThreshold)),
            KeyCode::Char('d') => Some(Action::EnterInputMode(InputMode::EditDiskThreshold)),
            _ => None,
        },
        _ => None,
    }
}
