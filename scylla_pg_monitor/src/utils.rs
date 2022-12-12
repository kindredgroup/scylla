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

// $coverage:ignore-start
#[cfg(test)]
mod tests {
  use crate::utils::*;
  use chrono::Duration;
  #[test]
  fn filter_expired_tasks_cases() {
    let tasks = vec![Task {
      rn: "future_task".to_string(),
      deadline : Some(Utc::now() + Duration::seconds(5)),
      ..Task::default()
    },
    Task {
      rn: "past_task".to_string(),
      deadline : Some(Utc::now() - Duration::seconds(1)),
      ..Task::default()
    },
    Task {
      rn: "no_deadline".to_string(),
      deadline : None,
      ..Task::default()
    }
    ];
    let ret_tasks = filter_expired_tasks(tasks);
    assert_eq!(ret_tasks.len(), 1);
    assert_eq!(ret_tasks[0].rn, "past_task".to_string());
  }
}