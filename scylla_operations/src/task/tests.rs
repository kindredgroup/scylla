// $coverage:ignore-start
use crate::task::{AddTaskModel, GetTaskModel, ScyllaOperationsError, ScyllaOperations};
use async_trait::async_trait;
use scylla_models::{Task, TaskStatus, UpdateTaskModel, UpdateOperation};
use thiserror::Error;

#[derive(PartialEq, Debug, Error)]
enum MyError{
  #[error("Scylla Op Error: {0}")]
  SyllaOpError(ScyllaOperationsError),
  #[error("DB Related Errors")]
  DBError
}

impl From<ScyllaOperationsError> for MyError {
  fn from(scylla_op_error: ScyllaOperationsError) -> Self {
    Self::SyllaOpError(scylla_op_error)
  }
}

struct ScyllaOperationsMock {

  insert: fn(Task) -> Result<Task, MyError>,
  update: fn(Task) -> Result<Task, MyError>,
  query: fn(&GetTaskModel) -> Result<Vec<Task>, MyError>,
  query_by_rn: fn(String) -> Result<Task, MyError>,
}
impl ScyllaOperationsMock {
  fn on_insert(mut self, f: fn(Task) -> Result<Task, MyError>) -> Self {
    self.insert = f;
    self
  }

  fn on_update(mut self, f: fn(Task) -> Result<Task, MyError>) -> Self {
    self.update = f;
    self
  }

  fn on_query(mut self, f: fn(&GetTaskModel) -> Result<Vec<Task>, MyError>) -> Self {
    self.query = f;
    self
  }

  fn on_query_by_rn(mut self, f: fn(String) -> Result<Task, MyError>) -> Self {
    self.query_by_rn = f;
    self
  }
}

impl Default for ScyllaOperationsMock {
  fn default() -> Self {
    Self {
      insert: |_| unimplemented!(),
      update: |_| unimplemented!(),
      query: |_| unimplemented!(),
      query_by_rn: |_| unimplemented!(),
    }
  }
}

#[async_trait]
impl ScyllaOperations for ScyllaOperationsMock {

  type PersistenceError = MyError;
  async fn insert(&self, task: Task) -> Result<Task, Self::PersistenceError> {
    (self.insert)(task)
  }

  async fn update(&self, task: Task) -> Result<Task,  Self::PersistenceError> {
    (self.update)(task)
  }

  async fn query(&self, get_task_model: &GetTaskModel) -> Result<Vec<Task>,  Self::PersistenceError> {
    (self.query)(get_task_model)
  }

  async fn query_by_rn(&self, rn: String) -> Result<Task,  Self::PersistenceError> {
    (self.query_by_rn)(rn)
  }
}

#[tokio::test]
async fn insert_returns_task() {
  let mock = ScyllaOperationsMock::default().on_insert(|t| {
    Ok(t)
  });

  let add_task_model = AddTaskModel {
    rn: "1234".to_string(),
    priority: 1,
    queue: "ss".to_string(),
    spec: serde_json::Value::default(),
  };
  let default_task:Task = Task::default();
  let returned_task = mock.add_task(&add_task_model).await.unwrap();
  assert_eq!(&returned_task.rn, &add_task_model.rn);
  assert_eq!(&returned_task.priority, &add_task_model.priority);
  assert_eq!(&returned_task.queue, &add_task_model.queue);
  assert_eq!(&returned_task.spec, &add_task_model.spec);
  // default values assigned
  assert_eq!(&returned_task.progress, &default_task.progress);
  assert_eq!(&returned_task.errors, &default_task.errors);
  assert_eq!(&returned_task.history, &default_task.history);
  assert_eq!(&returned_task.owner, &default_task.owner);
  assert_eq!(&returned_task.status, &default_task.status);

}


#[tokio::test]
async fn insert_returns_db_error() {
  let add_task_model = AddTaskModel {
    rn: "1234".to_string(),
    priority: 1,
    queue: "ss".to_string(),
    spec: serde_json::Value::default(),
  };
  let mock = ScyllaOperationsMock::default().on_insert(|_| Err(MyError::DBError));
  if let Err(e) = mock.add_task(&add_task_model).await {
    assert_eq!(e, MyError::DBError);
  }
}

#[tokio::test]
async fn insert_returns_duplicate_error() {
  let add_task_model = AddTaskModel {
    rn: "1234".to_string(),
    priority: 1,
    queue: "ss".to_string(),
    spec: serde_json::Value::default(),
  };
  let mock = ScyllaOperationsMock::default().on_insert(|_| Err(MyError::DBError));
  if let Err(e) = mock.add_task(&add_task_model).await {
    assert_eq!(e, MyError::DBError);
  }
}

#[tokio::test]
async fn get_task() {
  let mock = ScyllaOperationsMock::default().on_query_by_rn(|rn| Ok(Task { rn, ..Task::default() }));
   let task = mock.get_task("abc".to_string()).await.unwrap();
  assert_eq!(task.rn, "abc".to_string());
}

#[tokio::test]
async fn get_task_returns_no_record_found() {
  let mock = ScyllaOperationsMock::default().on_query_by_rn(|_rn| Err(MyError::DBError));
  assert_eq!(mock.get_task("abc".to_string()).await, Err(MyError::DBError));

}

#[tokio::test]
async fn get_tasks_default_params() {
  let mock = ScyllaOperationsMock::default().on_query(|get_task_model| {
    // default parameters set
    assert_eq!(get_task_model.limit, GetTaskModel::default().limit);
    assert_eq!(get_task_model.status, GetTaskModel::default().status);
    assert_eq!(get_task_model.worker, None);
    assert_eq!(get_task_model.queue, None);
    Ok(vec![Task::default()])
  });
  let task = mock.get_tasks(&GetTaskModel::default()).await.unwrap();
  assert_eq!(task[0].rn, "".to_string());
}

#[tokio::test]
async fn get_tasks_by_status() {
  let mock = ScyllaOperationsMock::default().on_query(|get_task_model| {
    // default parameters set
    assert_eq!(get_task_model.limit, GetTaskModel::default().limit);
    assert_eq!(get_task_model.status, Some(TaskStatus::Ready));
    assert_eq!(get_task_model.worker, None);
    assert_eq!(get_task_model.queue, None);
    Ok(vec![Task::default()])
  });
  let gtm = GetTaskModel {
    status: Some(TaskStatus::Ready),
    ..GetTaskModel::default()
  };
  let task = mock.get_tasks(&gtm).await.unwrap();
  assert_eq!(task[0].rn, "".to_string());
}

#[tokio::test]
async fn get_tasks_calls_query() {

  let mock: ScyllaOperationsMock = ScyllaOperationsMock::default().on_query(|get_task_model| {
    assert_eq!(get_task_model.limit, Some(10));
    assert_eq!(get_task_model.status, Some(TaskStatus::Ready));
    assert_eq!(get_task_model.worker, Some("test worker".to_string()));
    assert_eq!(get_task_model.queue, Some("queue1".to_string()));
    
    Ok(vec![Task {
      rn: "unique_id".to_string(),
      ..Task::default()
    }])
   
  });
  let gtm = GetTaskModel {
    status: Some(TaskStatus::Ready),
    limit: Some(10),
    queue: Some("queue1".to_string()),
    worker: Some("test worker".to_string())
  };
  let tasks = mock.get_tasks(&gtm).await.unwrap();
  assert_eq!(tasks[0].rn, "unique_id".to_string());
}

#[tokio::test]
async fn update_task_calls_get_and_update() {

  let mock: ScyllaOperationsMock = ScyllaOperationsMock::default().on_update(|t: Task| {
   // task is updated based on update operation before passing down to update method
    assert_eq!(t.rn, "unique_id".to_string());
    assert_eq!(t.status, TaskStatus::Running);
    assert_eq!(t.owner, Some("worker1".to_string()));
    Ok(Task {
      rn: "unique_id".to_string(),
      status: TaskStatus::Running,
      owner: Some("worker1".to_string()),
      ..Task::default()
    })  
  }).on_query_by_rn(|rn| {
    assert_eq!(rn, "unique_id".to_string());
    // get task returns task in ready state
    Ok(Task {
      rn: "unique_id".to_string(),
      status: TaskStatus::Ready,
      ..Task::default()
    })  
  });
  let utm = UpdateTaskModel {
    rn: "unique_id".to_string(),
    operation: UpdateOperation::Lease,
    status: Some(TaskStatus::Running),
    worker: Some("worker1".to_string()),
    error: None,
    progress: None
  };
  let task = mock.update_task(&utm).await.unwrap();
  assert_eq!(task.rn, "unique_id".to_string());
  assert_eq!(task.status, TaskStatus::Running);
  assert_eq!(task.owner, Some("worker1".to_string()));
} 

#[tokio::test]
async fn update_task_returns_scylla_op_error() {

  let mock: ScyllaOperationsMock = ScyllaOperationsMock::default().on_update(|t: Task| {
    // This will never be called in this case
     assert_eq!(t.rn, "unique_id".to_string());
     assert_eq!(t.status, TaskStatus::Running);
     assert_eq!(t.owner, Some("worker1".to_string()));
     Ok(Task {
       rn: "unique_id".to_string(),
       status: TaskStatus::Running,
       owner: Some("worker1".to_string()),
       ..Task::default()
     })  
   }).on_query_by_rn(|rn| {
     assert_eq!(rn, "unique_id".to_string());
     // get task returns task in ready state
     Ok(Task {
       rn: "unique_id".to_string(),
       status: TaskStatus::Ready,
       ..Task::default()
     })  
   });
   let utm = UpdateTaskModel {
     rn: "unique_id".to_string(),
     operation: UpdateOperation::Status,
     status: Some(TaskStatus::Completed),
     worker: None,
     error: None,
     progress: None
   };

   assert_eq!(mock.update_task(&utm).await, Err(MyError::SyllaOpError(ScyllaOperationsError::InvalidStatusTransition(TaskStatus::Ready, vec![TaskStatus::Cancelled]))));
}
#[tokio::test]
async fn get_task_calls_query_by_rn() {

  let mock: ScyllaOperationsMock = ScyllaOperationsMock::default().on_query_by_rn(|rn| {
     assert_eq!(rn, "unique_id_input".to_string());
     Ok(Task {
       rn: "unique_id".to_string(),
       status: TaskStatus::Ready,
       ..Task::default()
     })  
   });
   let task = mock.get_task("unique_id_input".to_string()).await.unwrap();
   assert_eq!(task.rn, "unique_id".to_string());
   assert_eq!(task.status, TaskStatus::Ready);
   assert_eq!(task.owner, None);
}

