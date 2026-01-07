use crate::{Task, TaskExecution, ExecutionStatus, Schedule, PluginManager, Result, ChronoError};
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub struct Scheduler {
    tasks: Arc<Mutex<HashMap<Uuid, Task>>>,
    executions: Arc<Mutex<HashMap<Uuid, TaskExecution>>>,
    plugin_manager: Arc<PluginManager>,
}

impl Scheduler {
    pub fn new(plugin_manager: Arc<PluginManager>) -> Self {
        Self {
            tasks: Arc::new(Mutex::new(HashMap::new())),
            executions: Arc::new(Mutex::new(HashMap::new())),
            plugin_manager,
        }
    }
    
    pub fn add_task(&self, task: Task) -> Uuid {
        let id = task.id;
        self.tasks.lock().unwrap().insert(id, task);
        id
    }
    
    pub fn remove_task(&self, id: &Uuid) -> Result<()> {
        self.tasks.lock().unwrap().remove(id)
            .ok_or_else(|| ChronoError::TaskNotFound(id.to_string()))?;
        Ok(())
    }
    
    pub fn get_task(&self, id: &Uuid) -> Result<Task> {
        self.tasks.lock().unwrap().get(id)
            .cloned()
            .ok_or_else(|| ChronoError::TaskNotFound(id.to_string()))
    }
    
    pub fn list_tasks(&self) -> Vec<Task> {
        self.tasks.lock().unwrap().values().cloned().collect()
    }
    
    pub async fn start(&self) {
        let tasks = Arc::clone(&self.tasks);
        let executions = Arc::clone(&self.executions);
        let plugin_manager = Arc::clone(&self.plugin_manager);
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
            
            loop {
                interval.tick().await;
                let now = Utc::now();
                
                let task_list: Vec<Task> = tasks.lock().unwrap().values().cloned().collect();
                
                for mut task in task_list {
                    if !task.enabled {
                        continue;
                    }
                    
                    if should_run(&task, now) {
                        println!("Running task: {}", task.name);
                        start_execution(&task, &executions, &plugin_manager).await;
                        task.last_run = Some(now);
                        task.next_run = calculate_next_run(&task.schedule, now);
                        tasks.lock().unwrap().insert(task.id, task);
                    }
                }
            }
        });
    }
}

fn should_run(task: &Task, now: DateTime<Utc>) -> bool {
    match &task.schedule {
        Schedule::Once { at } => task.last_run.is_none() && now >= *at,
        Schedule::Interval { seconds } => {
            if let Some(last) = task.last_run {
                now >= last + Duration::seconds(*seconds as i64)
            } else {
                true
            }
        },
        Schedule::Cron(_) => {
            if let Some(next) = task.next_run {
                now >= next
            } else {
                true
            }
        }
    }
}

fn calculate_next_run(schedule: &Schedule, from: DateTime<Utc>) -> Option<DateTime<Utc>> {
    match schedule {
        Schedule::Once { .. } => None,
        Schedule::Interval { seconds } => Some(from + Duration::seconds(*seconds as i64)),
        Schedule::Cron(_) => Some(from + Duration::seconds(60)),
    }
}

async fn start_execution(
    task: &Task,
    executions: &Arc<Mutex<HashMap<Uuid, TaskExecution>>>,
    plugin_manager: &Arc<PluginManager>,
) -> Uuid {
    let exec_id = Uuid::new_v4();
    let execution = TaskExecution {
        id: exec_id,
        task_id: task.id,
        started_at: Utc::now(),
        finished_at: None,
        status: ExecutionStatus::Running,
        output: None,
        error: None,
    };
    
    executions.lock().unwrap().insert(exec_id, execution.clone());
    
    let task_clone = task.clone();
    let executions_clone = Arc::clone(executions);
    let plugin_manager_clone = Arc::clone(plugin_manager);
    
    tokio::spawn(async move {
        let result = plugin_manager_clone.execute_plugin(
            &task_clone.plugin.name,
            &task_clone.plugin.config,
        );
        
        let mut execs = executions_clone.lock().unwrap();
        if let Some(exec) = execs.get_mut(&exec_id) {
            exec.finished_at = Some(Utc::now());
            
            match result {
                Ok(output) => {
                    exec.status = ExecutionStatus::Success;
                    exec.output = Some(output);
                },
                Err(e) => {
                    exec.status = ExecutionStatus::Failed;
                    exec.error = Some(e.to_string());
                }
            }
        }
    });
    
    exec_id
}