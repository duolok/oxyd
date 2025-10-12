use oxyd_domain::errors::ProcessError;
use tokio::fs;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use std::sync::Arc;
use std::collections::HashMap;

use super::parsers::{StatFields, parse_stat};

#[derive(Debug, Clone)]
pub struct CpuMeasurement {
    pub process_time: u64,
    pub system_time: u64,
    pub timestamp: std::time::Instant,
}

pub async fn calculate_cpu_usage_cached(
    pid: u32,
    stat_fields: &StatFields,
    cache: &Arc<Mutex<HashMap<u32, CpuMeasurement>>>,
) -> f64 {
    let process_time = stat_fields.utime + stat_fields.stime;
    let system_time = match get_system_cpu_time().await {
        Ok(time) => time,
        Err(_) => return 0.0,
    };

    let now = std::time::Instant::now();
    let mut cache_lock = cache.lock().await;

    if let Some(prev) = cache_lock.get(&pid) {
        let time_delta = now.duration_since(prev.timestamp).as_secs_f64();

        if time_delta < 0.1 {
            if prev.system_time > 0 {
                let num_cpus = num_cpus::get() as f64;
                let process_delta = process_time.saturating_sub(prev.process_time) as f64;
                let system_delta = system_time.saturating_sub(prev.system_time) as f64;

                if system_delta > 0.0 {
                    let cpu_percent = (process_delta / system_delta) * 100.0 * num_cpus;
                    return cpu_percent.min(100.0 * num_cpus);
                }
            }
            return 0.0;
        }

        let process_delta = process_time.saturating_sub(prev.process_time) as f64;
        let system_delta = system_time.saturating_sub(prev.system_time) as f64;

        if system_delta > 0.0 {
            let num_cpus = num_cpus::get() as f64;
            let cpu_percent = (process_delta / system_delta) * 100.0 * num_cpus;

            cache_lock.insert(pid, CpuMeasurement {
                process_time,
                system_time,
                timestamp: now,
            });

            return cpu_percent.min(100.0 * num_cpus);
        }
    }

    cache_lock.insert(pid, CpuMeasurement {
        process_time,
        system_time,
        timestamp: now,
    });

    0.0
}

pub async fn calculate_cpu_usage_instant(pid: u32) -> f64 {
    let measurement1 = match take_cpu_measurement(pid).await {
        Ok(m) => m,
        Err(_) => return 0.0,
    };

    sleep(Duration::from_millis(100)).await;

    let measurement2 = match take_cpu_measurement(pid).await {
        Ok(m) => m,
        Err(_) => return 0.0,
    };

    let process_delta = measurement2.process_time.saturating_sub(measurement1.process_time) as f64;
    let system_delta = measurement2.system_time.saturating_sub(measurement1.system_time) as f64;

    if system_delta > 0.0 {
        let num_cpus = num_cpus::get() as f64;
        let cpu_percent = (process_delta / system_delta) * 100.0 * num_cpus;
        return cpu_percent.min(100.0 * num_cpus);
    }

    0.0
}

async fn take_cpu_measurement(pid: u32) -> Result<CpuMeasurement, ProcessError> {
    let stat_path = format!("/proc/{}/stat", pid);
    let stat_content = fs::read_to_string(&stat_path).await
        .map_err(|e| ProcessError::ReadFailed(pid, format!("Failed to read stat: {}", e)))?;

    let stat_fields = parse_stat(&stat_content)?;
    let process_time = stat_fields.utime + stat_fields.stime;
    let system_time = get_system_cpu_time().await?;

    Ok(CpuMeasurement {
        process_time,
        system_time,
        timestamp: std::time::Instant::now(),
    })
}

async fn get_system_cpu_time() -> Result<u64, ProcessError> {
    let stat_content = fs::read_to_string("/proc/stat").await
        .map_err(|e| ProcessError::ReadFailed(0, format!("Failed to read /proc/stat: {}", e)))?;

    if let Some(first_line) = stat_content.lines().next() {
        if first_line.starts_with("cpu ") {
            let parts: Vec<&str> = first_line.split_whitespace().collect();
            let total: u64 = parts.iter()
                .skip(1)
                .take(10)
                .filter_map(|s| s.parse::<u64>().ok())
                .sum();

            return Ok(total);
        }
    }

    Err(ProcessError::ParseError("Could not parse /proc/stat".to_string()))
}
