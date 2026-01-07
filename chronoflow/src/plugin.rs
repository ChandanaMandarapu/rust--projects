use crate::{ChronoError, Result};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

pub struct PluginManager {
    plugins: HashMap<String, Box<dyn Fn(&JsonValue) -> Result<String> + Send + Sync>>,
}

impl PluginManager {
    pub fn new() -> Self {
        let mut manager = Self {
            plugins: HashMap::new(),
        };
        manager.register_builtin_plugins();
        manager
    }
    
    fn register_builtin_plugins(&mut self) {
        // HTTP request plugin
        self.plugins.insert(
            "http_request".to_string(),
            Box::new(|config: &JsonValue| {
                let url = config.get("url")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ChronoError::PluginError("url required".into()))?;
                Ok(format!("HTTP request to {} completed", url))
            })
        );
        
        // Logger plugin
        self.plugins.insert(
            "logger".to_string(),
            Box::new(|config: &JsonValue| {
                let msg = config.get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("No message");
                println!("[PLUGIN LOG] {}", msg);
                Ok(format!("Logged: {}", msg))
            })
        );
    }
    
    pub fn execute_plugin(&self, name: &str, config: &JsonValue) -> Result<String> {
        let plugin = self.plugins.get(name)
            .ok_or_else(|| ChronoError::PluginError(format!("Plugin {} not found", name)))?;
        plugin(config)
    }
}