use oxyd_domain::models::*;
use oxyd_core::engine::CoreEngine;

fn main() {
    println!("Starting OXYX System Monitor");

    let engine = CoreEngine::new_default();
    engine.run().await;
}

