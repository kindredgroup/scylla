use scylla_models::{GetTaskModel, Task};
use serde_json::{to_value};
use crate::error::PgAdapterError;

pub fn prepare_insert_task(task: &Task) -> serde_json::Value {
    to_value(task).unwrap()
  }
pub fn handle_insert_return(tasks: Vec<Task>, original_task: &Task) -> Result<Task, PgAdapterError> {
    return match tasks.len() {
      0 => Err(PgAdapterError::DuplicateTask(original_task.rn.to_owned())),
      1 =>  Ok(tasks[0].clone()),
      _ => panic!("Unexpected number of rows returned from insert query")
    }
  }
#[derive(PartialEq, Debug)]  
pub struct UpdateParams {
    pub json_task: serde_json::Value,
    pub rn: String
  }
pub fn prepare_update_task(task: &Task) -> UpdateParams {
    UpdateParams {
      rn: task.rn.clone(),
      json_task: to_value(task).unwrap()
    }
  }
pub fn handle_update_return(tasks: Vec<Task>, original_task: &Task) -> Result<Task, PgAdapterError> {
  return match tasks.len() {
    0 => Err(PgAdapterError::NoTaskFound(original_task.rn.to_owned())),
    1 =>  Ok(tasks[0].clone()),
    _ => panic!("Unexpected number of rows returned from update query")
  }
}
#[derive(PartialEq, Debug)]  
pub struct QueryParams {
    pub status: String,
    pub queue: String,
    pub worker: String,
    pub limit: i32
}
pub fn prepare_query_task(get_task_model: &GetTaskModel) -> QueryParams {
    let status = get_task_model.status.clone().map_or_else(|| '%'.to_string(), |s| s.to_string().to_lowercase());
      let queue = get_task_model.queue.clone().map_or_else(|| '%'.to_string(), |q| q);
      let worker = get_task_model.worker.clone().map_or_else(|| '%'.to_string(), |w| w);
      let limit = get_task_model.limit.map_or_else(|| 100, |l| l);
    QueryParams {
      status,
      queue,
      worker,
      limit
    }
  }
pub fn handle_query_by_rn_return(tasks: Vec<Task>, rn: &String) -> Result<Task, PgAdapterError> {
  return match tasks.len() {
    0 => Err(PgAdapterError::NoTaskFound(rn.to_owned())),
    1 =>  Ok(tasks[0].clone()),
    _ => panic!("Unexpected number of rows returned from query by rn query")
  }
}

// $coverage:ignore-start
#[cfg(test)]
mod tests {
  use scylla_models::TaskStatus;

use crate::adapter_utils::*;

#[test]
fn prepare_insert_task_cases() {
    let t = Task::default();
    assert_eq!(prepare_insert_task(&t), serde_json::to_value(&t).unwrap());
}

#[test]
fn handle_insert_return_cases() {
    let original_task = Task {
        rn: "123".to_string(),
        ..Task::default()
    };

    assert_eq!(handle_insert_return(vec![], &original_task).is_err(), true);
    assert_eq!(
        handle_insert_return(vec![], &original_task).unwrap_err().to_string(),
        PgAdapterError::DuplicateTask(original_task.rn.to_owned()).to_string()
    );
    let ret_t = Task {
        rn: "123".to_string(),
        status: scylla_models::TaskStatus::Running,
        ..Task::default()
    };
    // In case single item is returned from db. That will be retruned back
    assert_eq!(handle_insert_return(vec![ret_t.clone()], &original_task).unwrap(), ret_t);

}

#[test]
#[should_panic(expected = "Unexpected number of rows returned from insert query")]
fn handle_insert_return_cases_panics() {
    let original_task = Task {
        rn: "123".to_string(),
        ..Task::default()
    };
    let ret_t = Task {
        rn: "123".to_string(),
        status: scylla_models::TaskStatus::Running,
        ..Task::default()
    };
    let ret_t1 = Task {
        rn: "123".to_string(),
        status: scylla_models::TaskStatus::Running,
        ..Task::default()
    };
    handle_insert_return(vec![ret_t.clone(),ret_t1.clone()], &original_task).unwrap();
}

#[test]
fn prepare_update_task_cases() {
  let t = Task {
    rn: "123".to_string(),
    ..Task::default()
  };
  assert_eq!(
    prepare_update_task(&t),
    UpdateParams { 
      rn: t.rn.clone(),
      json_task: to_value(t).unwrap()
    })
}

#[test]
fn handle_update_return_cases() {
  let original_t = Task {
    rn: "123".to_string(),
    ..Task::default()
  };
  let resp = handle_update_return(vec![], &original_t);
  assert_eq!(resp.is_err(), true);
  assert_eq!(
     resp.unwrap_err().to_string(),
     PgAdapterError::NoTaskFound(original_t.rn.to_owned()).to_string()
  );
  let ret_t = Task {
    rn: "123".to_string(),
    status: TaskStatus::Running,
    ..Task::default()
  };
  assert_eq!(handle_update_return(vec![ret_t.clone()], &original_t).unwrap(), ret_t)
}

#[test]
#[should_panic(expected = "Unexpected number of rows returned from update query")]
fn handle_update_return_cases_panics() {
    let original_task = Task {
        rn: "123".to_string(),
        ..Task::default()
    };
    let ret_t = Task {
        rn: "123".to_string(),
        status: scylla_models::TaskStatus::Running,
        ..Task::default()
    };
    let ret_t1 = Task {
        rn: "123".to_string(),
        status: scylla_models::TaskStatus::Running,
        ..Task::default()
    };
    handle_update_return(vec![ret_t.clone(),ret_t1.clone()], &original_task).unwrap();
}

#[test]
fn prepare_query_task_cases() {
  // default values
  let gtm = GetTaskModel {
    status: None,
    queue: None,
    worker: None,
    limit: None
  };
  assert_eq!(prepare_query_task(&gtm), QueryParams {
    status: "%".to_string(),
    queue: "%".to_string(),
    worker: "%".to_string(),
    limit: 100
  });
  // Everything passed
  let gtm = GetTaskModel {
    status: Some(TaskStatus::Cancelled),
    queue: Some("abc".to_string()),
    worker: Some("s".to_string()),
    limit: Some(20)
  };
  assert_eq!(prepare_query_task(&gtm), QueryParams {
    status: "cancelled".to_string(),
    queue: "abc".to_string(),
    worker: "s".to_string(),
    limit: 20
  });

}
#[test]
fn handle_query_by_rn_return_cases() {
  let original_t = Task {
    rn: "123".to_string(),
    ..Task::default()
  };
  let resp = handle_query_by_rn_return(vec![], &original_t.rn.clone());
  assert_eq!(resp.is_err(), true);
  assert_eq!(
     resp.unwrap_err().to_string(),
     PgAdapterError::NoTaskFound(original_t.rn.to_owned()).to_string()
  );
  let ret_t = Task {
    rn: "123".to_string(),
    status: TaskStatus::Running,
    ..Task::default()
  };
  assert_eq!(handle_query_by_rn_return(vec![ret_t.clone()], &original_t.rn.clone()).unwrap(), ret_t)
}

#[test]
#[should_panic(expected = "Unexpected number of rows returned from query by rn query")]
fn handle_query_by_rn_cases_panics() {
    let original_task = Task {
        rn: "123".to_string(),
        ..Task::default()
    };
    let ret_t = Task {
        rn: "123".to_string(),
        status: scylla_models::TaskStatus::Running,
        ..Task::default()
    };
    let ret_t1 = Task {
        rn: "123".to_string(),
        status: scylla_models::TaskStatus::Running,
        ..Task::default()
    };
    handle_query_by_rn_return(vec![ret_t.clone(),ret_t1.clone()], &original_task.rn.clone()).unwrap();
}
}