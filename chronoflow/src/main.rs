mod error;
mod types;
mod plugin;
mod scheduler;

use std::sync::Arc;
use plugin::PluginManager;
use scheduler::Scheduler;
use types::{Task, Schedule, PluginConfig};

#[tokio::main]
async fn main() {
    println!("🚀 ChronoFlow - Distributed Task Scheduler");
    println!("==========================================\n");
    
    let plugin_manager = Arc::new(PluginManager::new());
    let scheduler = Arc::new(Scheduler::new(Arc::clone(&plugin_manager)));
    
    // Add demo task
    let demo_task = Task::new(
        "Demo Logger Task".to_string(),
        Schedule::Interval { seconds: 10 },
        PluginConfig {
            name: "logger".to_string(),
            wasm_path: "".to_string(),
            config: serde_json::json!({
                "message": "Hello from ChronoFlow! Task is running..."
            }),
        },
    );
    
    let task_id = scheduler.add_task(demo_task);
    println!("✅ Added task with ID: {}", task_id);
    
    // Start scheduler
    scheduler.start().await;
    
    println!("\n⏰ Scheduler started! Tasks will run every 10 seconds.");
    println!("Press Ctrl+C to stop...\n");
    
    // Keep running
    tokio::signal::ctrl_c().await.unwrap();
    println!("\n👋 Shutting down ChronoFlow...");
}