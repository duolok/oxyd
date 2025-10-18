use std::collections::VecDeque;

const MAX_HISTORY: usize = 60;

#[derive(Debug, Clone)]
pub struct MetricsHistory {
    pub cpu_usage: VecDeque<f32>,
    pub memory_usage: VecDeque<f32>,
    pub network_tx: VecDeque<u64>,
    pub network_rx: VecDeque<u64>,
    pub disk_read: VecDeque<u64>,
    pub disk_write: VecDeque<u64>,
}

impl MetricsHistory {
    pub fn new() -> Self {
        Self {
            cpu_usage: VecDeque::with_capacity(MAX_HISTORY),
            memory_usage: VecDeque::with_capacity(MAX_HISTORY),
            network_tx: VecDeque::with_capacity(MAX_HISTORY),
            network_rx: VecDeque::with_capacity(MAX_HISTORY),
            disk_read: VecDeque::with_capacity(MAX_HISTORY),
            disk_write: VecDeque::with_capacity(MAX_HISTORY),
        }
    }

    pub fn push_cpu(&mut self, value: f32) {
        if self.cpu_usage.len() >= MAX_HISTORY {
            self.cpu_usage.pop_front();
        }
        self.cpu_usage.push_back(value);
    }

    pub fn push_memory(&mut self, value: f32) {
        if self.memory_usage.len() >= MAX_HISTORY {
            self.memory_usage.pop_front();
        }
        self.memory_usage.push_back(value);
    }

    pub fn push_network(&mut self, tx: u64, rx: u64) {
        if self.network_tx.len() >= MAX_HISTORY {
            self.network_tx.pop_front();
            self.network_rx.pop_front();
        }
        self.network_tx.push_back(tx);
        self.network_rx.push_back(rx);
    }

    pub fn push_disk(&mut self, read: u64, write: u64) {
        if self.disk_read.len() >= MAX_HISTORY {
            self.disk_read.pop_front();
            self.disk_write.pop_front();
        }
        self.disk_read.push_back(read);
        self.disk_write.push_back(write);
    }

    pub fn cpu_data(&self) -> Vec<u64> {
        self.cpu_usage.iter().map(|&x| x as u64).collect()
    }

    pub fn memory_data(&self) -> Vec<u64> {
        self.memory_usage.iter().map(|&x| x as u64).collect()
    }

    pub fn network_tx_data(&self) -> Vec<u64> {
        self.network_tx.iter().copied().collect()
    }

    pub fn network_rx_data(&self) -> Vec<u64> {
        self.network_rx.iter().copied().collect()
    }
}
