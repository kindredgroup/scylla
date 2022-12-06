use chrono::Utc;
use scylla_models::Task;

pub fn filter_expired_tasks(tasks: Vec<Task>) -> Vec<Task> {
     tasks
          .into_iter()
          .filter(|t| match t.deadline {
            Some(dl) => dl < Utc::now(),
            None => false,
          })
          .collect()
}