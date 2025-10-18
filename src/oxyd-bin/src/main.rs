use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use oxyd_collectors::UnifiedCollector;
use oxyd_core::engine::Engine;
use oxyd_tui::{App, Event, EventHandler, event::map_key_to_action, app::Action};
use oxyd_domain::{ProcessSignal, traits::ProcessManager};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let engine = Engine::new_default();
    let process_manager = engine.process_manager().clone();
    
    let collector = UnifiedCollector::new(process_manager.clone(), true);
    engine.add_collector(Box::new(collector)).await;

    let mut app = App::new().with_process_manager(process_manager.clone());

    let mut event_handler = EventHandler::new();
    event_handler.start_polling().await;

    let mut metrics_rx = engine.subscribe_metrics();

    let (action_tx, mut action_rx) = mpsc::unbounded_channel::<Action>();

    let engine_handle = tokio::spawn(async move {
        if let Err(e) = engine.run().await {
            eprintln!("Engine error: {}", e);
        }
    });

    let action_tx_clone = action_tx.clone();
    tokio::spawn(async move {
        while let Ok(metrics) = metrics_rx.recv().await {
            let _ = action_tx_clone.send(Action::UpdateMetrics(metrics));
        }
    });

    let action_tx_clone = action_tx.clone();
    let pm_clone = process_manager.clone();
    tokio::spawn(async move {
        let _ = action_tx_clone.send(Action::LoadProcessList);
        
        let mut interval = interval(Duration::from_secs(3));
        loop {
            interval.tick().await;
            let _ = action_tx_clone.send(Action::LoadProcessList);
        }
    });

    // Main loop
    loop {
        // Render
        terminal.draw(|f| {
            oxyd_tui::ui::render(f, &app.state);
        })?;

        tokio::select! {
            Some(event) = event_handler.next() => {
                match event {
                    Event::Key(key) => {
                        if let Some(action) = map_key_to_action(key) {
                            let _ = action_tx.send(action);
                        }
                    }
                    Event::Tick => {
                        let _ = action_tx.send(Action::Tick);
                    }
                    Event::Resize => {
                    }
                }
            }
            Some(action) = action_rx.recv() => {
                match action.clone() {
                    Action::LoadProcessList => {
                        let pm = process_manager.clone();
                        let tx = action_tx.clone();
                        tokio::spawn(async move {
                            match load_process_list(pm).await {
                                Ok(processes) => {
                                    let _ = tx.send(Action::ProcessListLoaded(processes));
                                }
                                Err(e) => {
                                    let _ = tx.send(Action::ProcessActionFailed(
                                        format!("Failed to load processes: {}", e)
                                    ));
                                }
                            }
                        });
                    }
                    Action::KillSelectedProcess => {
                        if let Some(process) = app.get_selected_process() {
                            let pid = process.pid;
                            let name = process.name.clone();
                            let pm = process_manager.clone();
                            let tx = action_tx.clone();
                            
                            tokio::spawn(async move {
                                match pm.kill_process(pid).await {
                                    Ok(_) => {
                                        let _ = tx.send(Action::ProcessActionComplete(
                                            format!("Killed process {} (PID: {})", name, pid)
                                        ));
                                        let _ = tx.send(Action::LoadProcessList);
                                    }
                                    Err(e) => {
                                        let _ = tx.send(Action::ProcessActionFailed(
                                            format!("Failed to kill {}: {}", name, e)
                                        ));
                                    }
                                }
                            });
                        }
                    }
                    Action::SuspendSelectedProcess => {
                        if let Some(process) = app.get_selected_process() {
                            let pid = process.pid;
                            let name = process.name.clone();
                            let pm = process_manager.clone();
                            let tx = action_tx.clone();
                            
                            tokio::spawn(async move {
                                match pm.suspend_process(pid).await {
                                    Ok(_) => {
                                        let _ = tx.send(Action::ProcessActionComplete(
                                            format!("Suspended process {} (PID: {})", name, pid)
                                        ));
                                        let _ = tx.send(Action::LoadProcessList);
                                    }
                                    Err(e) => {
                                        let _ = tx.send(Action::ProcessActionFailed(
                                            format!("Failed to suspend {}: {}", name, e)
                                        ));
                                    }
                                }
                            });
                        }
                    }
                    Action::ContinueSelectedProcess => {
                        if let Some(process) = app.get_selected_process() {
                            let pid = process.pid;
                            let name = process.name.clone();
                            let pm = process_manager.clone();
                            let tx = action_tx.clone();
                            
                            tokio::spawn(async move {
                                match pm.continue_process(pid).await {
                                    Ok(_) => {
                                        let _ = tx.send(Action::ProcessActionComplete(
                                            format!("Continued process {} (PID: {})", name, pid)
                                        ));
                                        let _ = tx.send(Action::LoadProcessList);
                                    }
                                    Err(e) => {
                                        let _ = tx.send(Action::ProcessActionFailed(
                                            format!("Failed to continue {}: {}", name, e)
                                        ));
                                    }
                                }
                            });
                        }
                    }
                    Action::TerminateSelectedProcess => {
                        if let Some(process) = app.get_selected_process() {
                            let pid = process.pid;
                            let name = process.name.clone();
                            let pm = process_manager.clone();
                            let tx = action_tx.clone();
                            
                            tokio::spawn(async move {
                                match pm.send_signal(pid, ProcessSignal::Terminate).await {
                                    Ok(_) => {
                                        let _ = tx.send(Action::ProcessActionComplete(
                                            format!("Terminated process {} (PID: {})", name, pid)
                                        ));
                                        let _ = tx.send(Action::LoadProcessList);
                                    }
                                    Err(e) => {
                                        let _ = tx.send(Action::ProcessActionFailed(
                                            format!("Failed to terminate {}: {}", name, e)
                                        ));
                                    }
                                }
                            });
                        }
                    }
                    _ => {}
                }

                app.dispatch(action);
                
                if app.should_quit() {
                    break;
                }
            }
        }
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    drop(engine_handle);

    Ok(())
}

async fn load_process_list(
    process_manager: Arc<dyn ProcessManager>
) -> Result<Vec<oxyd_domain::models::Process>, Box<dyn std::error::Error>> {
    let pids = process_manager.list_processes().await?;
    
    let mut processes = Vec::new();
    
    for pid in pids.iter().take(500) {
        if let Ok(process) = process_manager.get_process(*pid).await {
            processes.push(process);
        }
    }
    
    processes.sort_by(|a, b| {
        b.cpu_usage_percent.partial_cmp(&a.cpu_usage_percent).unwrap()
    });
    
    Ok(processes)
}
