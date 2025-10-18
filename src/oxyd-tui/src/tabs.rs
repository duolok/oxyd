#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Overview,
    Cpu,
    Memory,
    Processes,
    Network,
    Disk,
    Notifications,
}

impl Tab {
    pub fn next(&self) -> Self {
        match self {
            Tab::Overview => Tab::Cpu,
            Tab::Cpu => Tab::Memory,
            Tab::Memory => Tab::Processes,
            Tab::Processes => Tab::Network,
            Tab::Network => Tab::Disk,
            Tab::Disk => Tab::Notifications,
            Tab::Notifications => Tab::Overview,
        }
    }

    pub fn previous(&self) -> Self {
        match self {
            Tab::Overview => Tab::Notifications,
            Tab::Cpu => Tab::Overview,
            Tab::Memory => Tab::Cpu,
            Tab::Processes => Tab::Memory,
            Tab::Network => Tab::Processes,
            Tab::Disk => Tab::Network,
            Tab::Notifications => Tab::Disk,
        }
    }

    pub fn title(&self) -> &str {
        match self {
            Tab::Overview => "Overview",
            Tab::Cpu => "CPU",
            Tab::Memory => "Memory",
            Tab::Processes => "Processes",
            Tab::Network => "Network",
            Tab::Disk => "Disk",
            Tab::Notifications => "Notifications",
        }
    }

    pub fn all() -> Vec<Tab> {
        vec![
            Tab::Overview,
            Tab::Cpu,
            Tab::Memory,
            Tab::Processes,
            Tab::Network,
            Tab::Disk,
            Tab::Notifications,
        ]
    }
}
