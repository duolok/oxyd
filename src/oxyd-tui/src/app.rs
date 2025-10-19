use crate::history::MetricsHistory;
use crate::notifications::NotificationManager;
use crate::tabs::Tab;
use oxyd_domain::traits::ProcessManager;
use oxyd_domain::{Process, SystemMetrics};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputMode {
    Normal,
    SearchProcess,
    EditCpuThreshold,
    EditMemoryThreshold,
    EditDiskThreshold,
}

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
    Home,
    End,
    SortByColumn(usize),

    LoadProcessList,
    ProcessListLoaded(Vec<Process>),
    SelectProcess(usize),
    KillSelectedProcess,
    SuspendSelectedProcess,
    ContinueSelectedProcess,
    TerminateSelectedProcess,
    ProcessActionComplete(String),
    ProcessActionFailed(String),

    ToggleHelp,

    MarkAllNotificationsRead,
    ClearAllNotifications,

    CheckAlerts(SystemMetrics),

    EnterInputMode(InputMode),
    ExitInputMode,
    InputChar(char),
    InputBackspace,
    InputSubmit,

    ClearFilter,
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

    pub process_list: Vec<Process>,
    pub filtered_process_list: Vec<Process>,
    pub process_filter: String,
    pub status_message: Option<String>,

    pub show_help: bool,

    pub notification_manager: NotificationManager,

    pub cpu_alert_threshold: f32,
    pub memory_alert_threshold: f32,
    pub disk_alert_threshold: f32,
    pub last_cpu_alert: Option<f32>,
    pub last_memory_alert: Option<f32>,
    pub last_disk_alert: Option<f32>,

    pub input_mode: InputMode,
    pub input_buffer: String,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            current_tab: Tab::Overview,
            should_quit: false,
            metrics: None,
            metrics_history: Some(MetricsHistory::new()),
            scroll_offset: 0,
            selected_process: Some(0),
            sort_column: 2,
            sort_ascending: false,
            update_count: 0,
            process_list: Vec::new(),
            filtered_process_list: Vec::new(),
            process_filter: String::new(),
            status_message: None,
            show_help: false,
            notification_manager: NotificationManager::new(),
            cpu_alert_threshold: 90.0,
            memory_alert_threshold: 90.0,
            disk_alert_threshold: 90.0,
            last_cpu_alert: None,
            last_memory_alert: None,
            last_disk_alert: None,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
        }
    }
}

pub struct App {
    pub state: AppState,
    pub process_manager: Option<Arc<dyn ProcessManager>>,
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            state: AppState::default(),
            process_manager: None,
        };

        app.state.notification_manager.add_info(
            "Welcome to OXYD".to_string(),
            "Linux System Monitor started successfully".to_string(),
        );

        app
    }

    pub fn with_process_manager(mut self, pm: Arc<dyn ProcessManager>) -> Self {
        self.process_manager = Some(pm);
        self
    }

    pub fn dispatch(&mut self, action: Action) {
        match action {
            Action::Tick => {}
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
                        metrics.network.total_bytes_received,
                    );
                }

                self.dispatch(Action::CheckAlerts(metrics.clone()));

                self.state.metrics = Some(metrics);
                self.state.update_count += 1;
            }
            Action::ScrollUp => {
                let list_len = self.state.filtered_process_list.len();
                if let Some(selected) = self.state.selected_process {
                    if selected > 0 {
                        self.state.selected_process = Some(selected - 1);
                        if selected - 1 < self.state.scroll_offset {
                            self.state.scroll_offset = selected - 1;
                        }
                    }
                } else if list_len > 0 {
                    self.state.selected_process = Some(0);
                }
            }
            Action::ScrollDown => {
                let max = self.state.filtered_process_list.len().saturating_sub(1);
                if let Some(selected) = self.state.selected_process {
                    if selected < max {
                        let new_selected = selected + 1;
                        self.state.selected_process = Some(new_selected);
                        let visible_rows = 20;
                        if new_selected >= self.state.scroll_offset + visible_rows {
                            self.state.scroll_offset = new_selected - visible_rows + 1;
                        }
                    }
                } else if !self.state.filtered_process_list.is_empty() {
                    self.state.selected_process = Some(0);
                }
            }
            Action::PageUp => {
                let page_size = 20;
                if let Some(selected) = self.state.selected_process {
                    let new_selected = selected.saturating_sub(page_size);
                    self.state.selected_process = Some(new_selected);
                    self.state.scroll_offset = new_selected.saturating_sub(5);
                } else if !self.state.filtered_process_list.is_empty() {
                    self.state.selected_process = Some(0);
                    self.state.scroll_offset = 0;
                }
            }
            Action::PageDown => {
                let page_size = 20;
                let max = self.state.filtered_process_list.len().saturating_sub(1);
                if let Some(selected) = self.state.selected_process {
                    let new_selected = (selected + page_size).min(max);
                    self.state.selected_process = Some(new_selected);
                    let visible_rows = 20;
                    if new_selected >= self.state.scroll_offset + visible_rows {
                        self.state.scroll_offset = new_selected.saturating_sub(visible_rows / 2);
                    }
                } else if !self.state.filtered_process_list.is_empty() {
                    self.state.selected_process = Some(0);
                    self.state.scroll_offset = 0;
                }
            }
            Action::Home => {
                if !self.state.filtered_process_list.is_empty() {
                    self.state.selected_process = Some(0);
                    self.state.scroll_offset = 0;
                }
            }
            Action::End => {
                let max = self.state.filtered_process_list.len().saturating_sub(1);
                if !self.state.filtered_process_list.is_empty() {
                    self.state.selected_process = Some(max);
                    let visible_rows = 20;
                    self.state.scroll_offset = max.saturating_sub(visible_rows - 1);
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
                self.apply_filter();
            }
            Action::ProcessListLoaded(processes) => {
                self.state.process_list = processes;
                self.sort_processes();
                self.apply_filter();
                if self.state.selected_process.is_none()
                    && !self.state.filtered_process_list.is_empty()
                {
                    self.state.selected_process = Some(0);
                }
            }
            Action::SelectProcess(index) => {
                if index < self.state.filtered_process_list.len() {
                    self.state.selected_process = Some(index);
                }
            }
            Action::ProcessActionComplete(msg) => {
                self.state.status_message = Some(msg.clone());
                self.state
                    .notification_manager
                    .add_success("Process Action".to_string(), msg);
            }
            Action::ProcessActionFailed(msg) => {
                self.state.status_message = Some(format!("ERROR: {}", msg));
                self.state
                    .notification_manager
                    .add_critical("Process Action Failed".to_string(), msg);
            }
            Action::ToggleHelp => {
                self.state.show_help = !self.state.show_help;
            }
            Action::MarkAllNotificationsRead => {
                self.state.notification_manager.mark_all_read();
            }
            Action::ClearAllNotifications => {
                self.state.notification_manager.clear();
            }
            Action::CheckAlerts(metrics) => {
                // Check CPU alert
                let cpu_usage = metrics.cpu.overall_usage_percent;
                if cpu_usage > self.state.cpu_alert_threshold {
                    if self.state.last_cpu_alert.is_none()
                        || self.state.last_cpu_alert.unwrap() < self.state.cpu_alert_threshold
                    {
                        self.state.notification_manager.add_warning(
                            "High CPU Usage".to_string(),
                            format!(
                                "CPU usage is at {:.1}% (threshold: {:.1}%)",
                                cpu_usage, self.state.cpu_alert_threshold
                            ),
                        );
                    }
                    self.state.last_cpu_alert = Some(cpu_usage);
                } else {
                    self.state.last_cpu_alert = None;
                }

                let mem_usage = metrics.memory.usage_percent;
                if mem_usage > self.state.memory_alert_threshold {
                    if self.state.last_memory_alert.is_none()
                        || self.state.last_memory_alert.unwrap() < self.state.memory_alert_threshold
                    {
                        self.state.notification_manager.add_warning(
                            "High Memory Usage".to_string(),
                            format!(
                                "Memory usage is at {:.1}% (threshold: {:.1}%)",
                                mem_usage, self.state.memory_alert_threshold
                            ),
                        );
                    }
                    self.state.last_memory_alert = Some(mem_usage);
                } else {
                    self.state.last_memory_alert = None;
                }

                for disk in &metrics.disks {
                    let disk_usage = disk.info.usage_percent;
                    if disk_usage > self.state.disk_alert_threshold {
                        if self.state.last_disk_alert.is_none()
                            || self.state.last_disk_alert.unwrap() < self.state.disk_alert_threshold
                        {
                            self.state.notification_manager.add_warning(
                                "High Disk Usage".to_string(),
                                format!(
                                    "Disk {} usage is at {:.1}% (threshold: {:.1}%)",
                                    disk.info.mount_point,
                                    disk_usage,
                                    self.state.disk_alert_threshold
                                ),
                            );
                        }
                        self.state.last_disk_alert = Some(disk_usage);
                        break;
                    } else {
                        self.state.last_disk_alert = None;
                    }
                }
            }
            Action::EnterInputMode(mode) => {
                self.state.input_mode = mode;
                self.state.input_buffer.clear();

                match mode {
                    InputMode::EditCpuThreshold => {
                        self.state.input_buffer = format!("{:.0}", self.state.cpu_alert_threshold);
                    }
                    InputMode::EditMemoryThreshold => {
                        self.state.input_buffer =
                            format!("{:.0}", self.state.memory_alert_threshold);
                    }
                    InputMode::EditDiskThreshold => {
                        self.state.input_buffer = format!("{:.0}", self.state.disk_alert_threshold);
                    }
                    InputMode::SearchProcess => {
                        self.state.input_buffer = self.state.process_filter.clone();
                    }
                    _ => {}
                }
            }
            Action::ExitInputMode => {
                self.state.input_mode = InputMode::Normal;
                self.state.input_buffer.clear();
            }
            Action::InputChar(c) => {
                if self.state.input_mode != InputMode::Normal {
                    self.state.input_buffer.push(c);

                    // Live filtering for search
                    if self.state.input_mode == InputMode::SearchProcess {
                        self.state.process_filter = self.state.input_buffer.clone();
                        self.apply_filter();
                        self.state.selected_process = Some(0);
                        self.state.scroll_offset = 0;
                    }
                }
            }
            Action::InputBackspace => {
                if self.state.input_mode != InputMode::Normal {
                    self.state.input_buffer.pop();

                    // Live filtering for search
                    if self.state.input_mode == InputMode::SearchProcess {
                        self.state.process_filter = self.state.input_buffer.clone();
                        self.apply_filter();
                        self.state.selected_process = Some(0);
                        self.state.scroll_offset = 0;
                    }
                }
            }
            Action::InputSubmit => {
                match self.state.input_mode {
                    InputMode::EditCpuThreshold => {
                        if let Ok(value) = self.state.input_buffer.parse::<f32>() {
                            if value >= 0.0 && value <= 100.0 {
                                self.state.cpu_alert_threshold = value;
                                self.state.notification_manager.add_success(
                                    "Settings Updated".to_string(),
                                    format!("CPU threshold set to {:.0}%", value),
                                );
                            } else {
                                self.state.notification_manager.add_critical(
                                    "Invalid Input".to_string(),
                                    "Value must be between 0 and 100".to_string(),
                                );
                            }
                        } else {
                            self.state.notification_manager.add_critical(
                                "Invalid Input".to_string(),
                                "Please enter a valid number".to_string(),
                            );
                        }
                    }
                    InputMode::EditMemoryThreshold => {
                        if let Ok(value) = self.state.input_buffer.parse::<f32>() {
                            if value >= 0.0 && value <= 100.0 {
                                self.state.memory_alert_threshold = value;
                                self.state.notification_manager.add_success(
                                    "Settings Updated".to_string(),
                                    format!("Memory threshold set to {:.0}%", value),
                                );
                            } else {
                                self.state.notification_manager.add_critical(
                                    "Invalid Input".to_string(),
                                    "Value must be between 0 and 100".to_string(),
                                );
                            }
                        } else {
                            self.state.notification_manager.add_critical(
                                "Invalid Input".to_string(),
                                "Please enter a valid number".to_string(),
                            );
                        }
                    }
                    InputMode::EditDiskThreshold => {
                        if let Ok(value) = self.state.input_buffer.parse::<f32>() {
                            if value >= 0.0 && value <= 100.0 {
                                self.state.disk_alert_threshold = value;
                                self.state.notification_manager.add_success(
                                    "Settings Updated".to_string(),
                                    format!("Disk threshold set to {:.0}%", value),
                                );
                            } else {
                                self.state.notification_manager.add_critical(
                                    "Invalid Input".to_string(),
                                    "Value must be between 0 and 100".to_string(),
                                );
                            }
                        } else {
                            self.state.notification_manager.add_critical(
                                "Invalid Input".to_string(),
                                "Please enter a valid number".to_string(),
                            );
                        }
                    }
                    InputMode::SearchProcess => {
                        // Already applied live
                    }
                    _ => {}
                }
                self.state.input_mode = InputMode::Normal;
                self.state.input_buffer.clear();
            }
            Action::ClearFilter => {
                self.state.process_filter.clear();
                self.state.input_buffer.clear();
                self.apply_filter();
                self.state.selected_process = Some(0);
                self.state.scroll_offset = 0;
            }
            Action::LoadProcessList
            | Action::KillSelectedProcess
            | Action::SuspendSelectedProcess
            | Action::ContinueSelectedProcess
            | Action::TerminateSelectedProcess => {}
        }
    }

    fn sort_processes(&mut self) {
        let ascending = self.state.sort_ascending;
        match self.state.sort_column {
            0 => self.state.process_list.sort_by(|a, b| {
                if ascending {
                    a.pid.cmp(&b.pid)
                } else {
                    b.pid.cmp(&a.pid)
                }
            }),
            1 => self.state.process_list.sort_by(|a, b| {
                if ascending {
                    a.name.cmp(&b.name)
                } else {
                    b.name.cmp(&a.name)
                }
            }),
            2 => self.state.process_list.sort_by(|a, b| {
                if ascending {
                    a.cpu_usage_percent
                        .partial_cmp(&b.cpu_usage_percent)
                        .unwrap()
                } else {
                    b.cpu_usage_percent
                        .partial_cmp(&a.cpu_usage_percent)
                        .unwrap()
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

    fn apply_filter(&mut self) {
        if self.state.process_filter.is_empty() {
            self.state.filtered_process_list = self.state.process_list.clone();
        } else {
            let filter_lower = self.state.process_filter.to_lowercase();
            self.state.filtered_process_list = self
                .state
                .process_list
                .iter()
                .filter(|p| {
                    p.name.to_lowercase().contains(&filter_lower)
                        || p.command.to_lowercase().contains(&filter_lower)
                        || p.pid.to_string().contains(&filter_lower)
                        || p.user.to_lowercase().contains(&filter_lower)
                })
                .cloned()
                .collect();
        }
    }

    pub fn should_quit(&self) -> bool {
        self.state.should_quit
    }

    pub fn get_selected_process(&self) -> Option<&Process> {
        self.state
            .selected_process
            .and_then(|idx| self.state.filtered_process_list.get(idx))
    }
}
