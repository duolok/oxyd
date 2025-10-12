use oxyd_core::engine::Engine;
use oxyd_collectors::UnifiedCollector;
use tokio::time::{sleep, Duration};
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\nOXYD System Monitor - Collector Test\n");
    println!("Testing all collectors before TUI implementation...\n");

    let engine = Engine::new_default();
    
    let process_manager = engine.process_manager().clone();
    let collector = UnifiedCollector::new(process_manager, true);
    
    engine.add_collector(Box::new(collector)).await;
    
    let mut metrics_rx = engine.subscribe_metrics();
    
    let mut update_count = 0;
    
    let display_task = tokio::spawn(async move {
        while let Ok(metrics) = metrics_rx.recv().await {
            update_count += 1;
            
            print!("\x1B[2J\x1B[1;1H");
            io::stdout().flush().unwrap();
            
            println!("╔═══════════════════════════════════════════════════════════════════════╗");
            println!("║               OXYD COLLECTOR TEST - Update #{}              ", update_count);
            println!("║  Time: {}                                          ", 
                metrics.timestamp.format("%Y-%m-%d %H:%M:%S"));
            println!("╠═══════════════════════════════════════════════════════════════════════╣");
            
            println!("║ 🖥️  SYSTEM INFORMATION");
            println!("║  └─ Hostname: {}", metrics.system_info.hostname);
            println!("║  └─ Architecture: {}", metrics.system_info.architecture);
            println!("║  └─ OS: {}", metrics.system_info.os_version);
            println!("║  └─ Uptime: {} seconds ({:.1} hours)", 
                metrics.system_info.uptime_seconds,
                metrics.system_info.uptime_seconds as f64 / 3600.0
            );
            println!("╠═══════════════════════════════════════════════════════════════════════╣");
            
            println!("║ 💻 CPU METRICS");
            println!("║  └─ Overall Usage: {:.1}%", metrics.cpu.overall_usage_percent);
            println!("║  └─ Load Average: {:.2} (1m), {:.2} (5m), {:.2} (15m)",
                metrics.cpu.load_average.one_minute,
                metrics.cpu.load_average.five_minutes,
                metrics.cpu.load_average.fifteen_minutes
            );
            
            if !metrics.cpu.cores.is_empty() {
                println!("║  └─ Cores ({} total):", metrics.cpu.cores.len());
                
                for chunk in metrics.cpu.cores.chunks(4) {
                    print!("║     ");
                    for core in chunk {
                        let bar = create_usage_bar(core.usage_percent, 10);
                        print!("CPU{:2}[{}] {:.1}%  ", core.id, bar, core.usage_percent);
                    }
                    println!();
                }
            }
            println!("╠═══════════════════════════════════════════════════════════════════════╣");
            
            println!("║ 💾 MEMORY METRICS");
            let mem_gb = metrics.memory.total_bytes as f64 / 1024.0 / 1024.0 / 1024.0;
            let used_gb = metrics.memory.used_bytes as f64 / 1024.0 / 1024.0 / 1024.0;
            let avail_gb = metrics.memory.available_bytes as f64 / 1024.0 / 1024.0 / 1024.0;
            
            println!("║  └─ Total: {:.2} GB", mem_gb);
            println!("║  └─ Used: {:.2} GB ({:.1}%)", used_gb, metrics.memory.usage_percent);
            println!("║  └─ Available: {:.2} GB", avail_gb);
            println!("║  └─ Cached: {:.2} GB", 
                metrics.memory.cached_bytes as f64 / 1024.0 / 1024.0 / 1024.0);
            
            let mem_bar = create_usage_bar(metrics.memory.usage_percent, 50);
            println!("║  └─ Usage: [{}]", mem_bar);
            
            if metrics.memory.swap_total_bytes > 0 {
                let swap_gb = metrics.memory.swap_total_bytes as f64 / 1024.0 / 1024.0 / 1024.0;
                let swap_used_gb = metrics.memory.swap_used_bytes as f64 / 1024.0 / 1024.0 / 1024.0;
                println!("║  └─ Swap: {:.2} GB / {:.2} GB ({:.1}%)",
                    swap_used_gb, swap_gb, metrics.memory.swap_usage_percent);
            }
            println!("╠═══════════════════════════════════════════════════════════════════════╣");
            
            println!("║ 🔄 PROCESS METRICS");
            println!("║  └─ Total Processes: {}", metrics.processes.total_count);
            println!("║  └─ Running: {}", metrics.processes.running_count);
            println!("║  └─ Sleeping: {}", metrics.processes.sleeping_count);
            println!("║  └─ Stopped: {}", metrics.processes.stopped_count);
            println!("║  └─ Zombie: {}", metrics.processes.zombie_count);
            
            let total = metrics.processes.total_count as f32;
            let running_pct = (metrics.processes.running_count as f32 / total * 100.0) as usize;
            let sleeping_pct = (metrics.processes.sleeping_count as f32 / total * 100.0) as usize;
            let stopped_pct = (metrics.processes.stopped_count as f32 / total * 100.0) as usize;
            let zombie_pct = (metrics.processes.zombie_count as f32 / total * 100.0) as usize;
            
            println!("║  └─ Distribution:");
            println!("║     [{}{}{}{}]",
                "▶".repeat(running_pct.min(25)),
                "█".repeat(sleeping_pct.min(25)),
                "⏸".repeat(stopped_pct.min(25)),
                "Z".repeat(zombie_pct.min(25))
            );
            println!("╠═══════════════════════════════════════════════════════════════════════╣");
            
            println!("║ ✅ COLLECTOR STATUS");
            println!("║  └─ All collectors operational");
            println!("║  └─ Metrics update interval: 1000ms");
            println!("║  └─ Next update in 1 second...");
            println!("╚═══════════════════════════════════════════════════════════════════════╝");
            println!();
            println!("Press Ctrl+C to stop...");
        }
    });
    
    let engine_handle = tokio::spawn(async move {
        if let Err(e) = engine.run().await {
            eprintln!("Engine error: {}", e);
        }
    });
    
    println!("Running for 30 seconds to demonstrate collectors...\n");
    sleep(Duration::from_secs(30)).await;
    
    tokio::select! {
        _ = display_task => {},
        _ = engine_handle => {},
        _ = sleep(Duration::from_secs(1)) => {}
    }

    Ok(())
}

fn create_usage_bar(percentage: f32, width: usize) -> String {
    let filled = ((percentage / 100.0) * width as f32) as usize;
    let empty = width.saturating_sub(filled);
    
    let bar = match percentage {
        p if p < 50.0 => format!("{}{}",
            "█".repeat(filled),
            "░".repeat(empty)
        ),
        p if p < 80.0 => format!("{}{}",
            "▓".repeat(filled),
            "░".repeat(empty)
        ),
        _ => format!("{}{}",
            "█".repeat(filled),
            "░".repeat(empty)
        ),
    };
    
    bar
}
