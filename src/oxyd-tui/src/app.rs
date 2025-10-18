use oxyd_domain::{SystemMetrics, Process};
use std::sync::Arc;
use oxyd_domain::traits::ProcessManager;
use crate::tabs::Tab;
use crate::history::MetricsHistory;

#[derive(Debug, Clone)]
pub enum Action {
    Tick,
    Quit,
    SwitchTab(Tab),
    NextTab,
    PreviousTab,
    UpdateMetrics(SystemMetrics),
    ScrollUp,
    ScrollDown,
    PageUp,
    PageDown,
    SortByColumn(usize),
    
    // Process management - REAL ACTIONS
    LoadProcessList,
    ProcessListLoaded(Vec<Process>),
    SelectProcess(usize),
    KillSelectedProcess,
    SuspendSelectedProcess,
    ContinueSelectedProcess,
    TerminateSelectedProcess,
    ProcessActionComplete(String),  // Success message
    ProcessActionFailed(String),    // Error message
}

pub struct AppState {
    pub current_tab: Tab,
    pub should_quit: bool,
    pub metrics: Option<SystemMetrics>,
    pub metrics_history: Option<MetricsHistory>,
    pub scroll_offset: usize,
    pub selected_process: Option<usize>,
    pub sort_column: usize,
    pub sort_ascending: bool,
    pub update_count: u64,
    
    // Process management - NOVO
    pub process_list: Vec<Process>,
    pub process_filter: String,
    pub status_message: Option<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            current_tab: Tab::Overview,
            should_quit: false,
            metrics: None,
            metrics_history: Some(MetricsHistory::new()),
            scroll_offset: 0,
            selected_process: Some(0),  // Start with first process selected
            sort_column: 0,
            sort_ascending: false,
            update_count: 0,
            process_list: Vec::new(),
            process_filter: String::new(),
            status_message: None,
        }
    }
}

pub struct App {
    pub state: AppState,
    pub process_manager: Option<Arc<dyn ProcessManager>>,  // NOVO
}

impl App {
    pub fn new() -> Self {
        Self {
            state: AppState::default(),
            process_manager: None,
        }
    }

    pub fn with_process_manager(mut self, pm: Arc<dyn ProcessManager>) -> Self {
        self.process_manager = Some(pm);
        self
    }

    pub fn dispatch(&mut self, action: Action) {
        match action {
            Action::Tick => {
                // On tick, refresh process list if on Processes tab
                if self.state.current_tab == Tab::Processes {
                    // We'll trigger load in main loop
                }
            }
            Action::Quit => {
                self.state.should_quit = true;
            }
            Action::SwitchTab(tab) => {
                self.state.current_tab = tab;
                self.state.scroll_offset = 0;
                self.state.status_message = None;
            }
            Action::NextTab => {
                self.state.current_tab = self.state.current_tab.next();
                self.state.scroll_offset = 0;
                self.state.status_message = None;
            }
            Action::PreviousTab => {
                self.state.current_tab = self.state.current_tab.previous();
                self.state.scroll_offset = 0;
                self.state.status_message = None;
            }
            Action::UpdateMetrics(metrics) => {
                if let Some(ref mut history) = self.state.metrics_history {
                    history.push_cpu(metrics.cpu.overall_usage_percent);
                    history.push_memory(metrics.memory.usage_percent);
                    history.push_network(
                        metrics.network.total_bytes_sent,
                        metrics.network.total_bytes_received
                    );
                }
                
                self.state.metrics = Some(metrics);
                self.state.update_count += 1;
            }
            Action::ScrollUp => {
                if self.state.scroll_offset > 0 {
                    self.state.scroll_offset -= 1;
                }
                // Also update selected process
                if let Some(selected) = self.state.selected_process {
                    if selected > 0 {
                        self.state.selected_process = Some(selected - 1);
                    }
                }
            }
            Action::ScrollDown => {
                let max = self.state.process_list.len().saturating_sub(1);
                if let Some(selected) = self.state.selected_process {
                    if selected < max {
                        self.state.selected_process = Some(selected + 1);
                        self.state.scroll_offset += 1;
                    }
                }
            }
            Action::PageUp => {
                self.state.scroll_offset = self.state.scroll_offset.saturating_sub(10);
                if let Some(selected) = self.state.selected_process {
                    self.state.selected_process = Some(selected.saturating_sub(10));
                }
            }
            Action::PageDown => {
                let max = self.state.process_list.len().saturating_sub(1);
                if let Some(selected) = self.state.selected_process {
                    let new_selected = (selected + 10).min(max);
                    self.state.selected_process = Some(new_selected);
                    self.state.scroll_offset += 10;
                }
            }
            Action::SortByColumn(column) => {
                if self.state.sort_column == column {
                    self.state.sort_ascending = !self.state.sort_ascending;
                } else {
                    self.state.sort_column = column;
                    self.state.sort_ascending = false;
                }
                self.sort_processes();
            }
            Action::ProcessListLoaded(processes) => {
                self.state.process_list = processes;
                self.sort_processes();
                if self.state.selected_process.is_none() && !self.state.process_list.is_empty() {
                    self.state.selected_process = Some(0);
                }
            }
            Action::SelectProcess(index) => {
                if index < self.state.process_list.len() {
                    self.state.selected_process = Some(index);
                }
            }
            Action::ProcessActionComplete(msg) => {
                self.state.status_message = Some(msg);
            }
            Action::ProcessActionFailed(msg) => {
                self.state.status_message = Some(format!("ERROR: {}", msg));
            }
            // These actions are handled externally (in main loop)
            Action::LoadProcessList |
            Action::KillSelectedProcess |
            Action::SuspendSelectedProcess |
            Action::ContinueSelectedProcess |
            Action::TerminateSelectedProcess => {}
        }
    }

    fn sort_processes(&mut self) {
        let ascending = self.state.sort_ascending;
        match self.state.sort_column {
            0 => self.state.process_list.sort_by(|a, b| {
                if ascending { a.pid.cmp(&b.pid) } else { b.pid.cmp(&a.pid) }
            }),
            1 => self.state.process_list.sort_by(|a, b| {
                if ascending { a.name.cmp(&b.name) } else { b.name.cmp(&a.name) }
            }),
            2 => self.state.process_list.sort_by(|a, b| {
                if ascending { 
                    a.cpu_usage_percent.partial_cmp(&b.cpu_usage_percent).unwrap()
                } else { 
                    b.cpu_usage_percent.partial_cmp(&a.cpu_usage_percent).unwrap()
                }
            }),
            3 => self.state.process_list.sort_by(|a, b| {
                if ascending { 
                    a.memory_usage_bytes.cmp(&b.memory_usage_bytes)
                } else { 
                    b.memory_usage_bytes.cmp(&a.memory_usage_bytes)
                }
            }),
            _ => {}
        }
    }

    pub fn should_quit(&self) -> bool {
        self.state.should_quit
    }

    pub fn get_selected_process(&self) -> Option<&Process> {
        self.state.selected_process
            .and_then(|idx| self.state.process_list.get(idx))
    }
}
