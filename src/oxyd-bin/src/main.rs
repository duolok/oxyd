use oxyd_core::engine::{Engine};

#[tokio::main]
async fn main() {
    println!("Starting OXYD System Monitor");

    let engine = Engine::new_default();
    let process_manager = engine.process_manager();

      match process_manager.list_processes().await {
        Ok(pids) => {
            println!("Found {} processes:", pids.len());
            
            for pid in pids.iter() {
                match process_manager.get_process(*pid).await {
                    Ok(process) => {
                        println!("  PID {}: {} ({})", 
                            process.pid, 
                            process.name, 
                            match process.state {
                                oxyd_domain::models::ProcessState::Running => "Running",
                                oxyd_domain::models::ProcessState::Sleeping => "Sleeping",
                                oxyd_domain::models::ProcessState::Stopped => "Stopped",
                                oxyd_domain::models::ProcessState::Zombie => "Zombie",
                                _ => "Unknown",
                            }
                        );
                    }
                    Err(e) => eprintln!("    Error getting process {}: {}", pid, e),
                }
            }
        }
        Err(e) => eprintln!("Error listing processes: {}", e),
    }
}
