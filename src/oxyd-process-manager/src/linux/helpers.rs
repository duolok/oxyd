use tokio::fs;
use chrono::Utc;
use oxyd_domain::errors::ProcessError;

pub async fn count_connections(path: &str) -> u32 {
    match fs::read_to_string(path).await {
        Ok(content) => content.lines().count() as u32,
        Err(_) => 0,
    }
}

pub async fn get_boot_time() -> Result<chrono::DateTime<Utc>, ProcessError> {
    let stat_content = fs::read_to_string("/proc/stat").await
        .map_err(|e| ProcessError::ReadFailed(0, format!("Failed to read /proc/stat: {}", e)))?;

    for line in stat_content.lines() {
        if line.starts_with("btime") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let boot_timestamp = parts[1].parse::<i64>()
                    .map_err(|_| ProcessError::ParseError("Invalid btime".to_string()))?;
                return Ok(chrono::DateTime::from_timestamp(boot_timestamp, 0)
                    .unwrap_or_else(|| Utc::now()));
            }
        }
    }
    Err(ProcessError::ParseError("Could not find btime".to_string()))
}

pub async fn calculate_memory_percent(rss_bytes: u64) -> f64 {
    match fs::read_to_string("/proc/meminfo").await {
        Ok(content) => {
            for line in content.lines() {
                if line.starts_with("MemTotal:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let Ok(total_kb) = parts[1].parse::<u64>() {
                            let total_bytes = total_kb * 1024;
                            return (rss_bytes as f64 / total_bytes as f64) * 100.0;
                        }
                    }
                }
            }
            0.0
        }
        Err(_) => 0.0,
    }
}
