// $coverage:ignore-start
use chrono::{Duration, Utc};
use scylla_operations::task::Persistence;

use crate::error::PgAdapterError;

use super::PgManager;
use async_trait::async_trait;
use scylla_models::*;

struct MockPgAdapter {
    insert: fn(Task) -> Result<Task, PgAdapterError>,
    insert_many: fn(Vec<Task>) -> Result<TaskBatch, PgAdapterError>,
    update: fn(Task) -> Result<Task, PgAdapterError>,
    query: fn(&GetTaskModel) -> Result<Vec<Task>, PgAdapterError>,
    query_by_rn: fn(String) -> Result<Task, PgAdapterError>,
    reset_batch: fn() -> Result<Vec<Task>, PgAdapterError>,
    lease_batch: fn(queue: String, limit: i32, worker: String, task_timeout_in_secs: i64) -> Result<Vec<Task>, PgAdapterError>,
    delete_batch: fn(retention_time_in_secs: i64) -> Result<u64, PgAdapterError>,
}

impl MockPgAdapter {
    fn on_insert(mut self, f: fn(Task) -> Result<Task, PgAdapterError>) -> Self {
        self.insert = f;
        self
    }

    fn on_insert_many(mut self, f: fn(Vec<Task>) -> Result<TaskBatch, PgAdapterError>) -> Self {
        self.insert_many = f;
        self
    }

    fn on_update(mut self, f: fn(Task) -> Result<Task, PgAdapterError>) -> Self {
        self.update = f;
        self
    }

    fn on_query(mut self, f: fn(&GetTaskModel) -> Result<Vec<Task>, PgAdapterError>) -> Self {
        self.query = f;
        self
    }

    fn on_query_by_rn(mut self, f: fn(String) -> Result<Task, PgAdapterError>) -> Self {
        self.query_by_rn = f;
        self
    }

    fn on_reset_batch(mut self, f: fn() -> Result<Vec<Task>, PgAdapterError>) -> Self {
        self.reset_batch = f;
        self
    }
}

impl Default for MockPgAdapter {
    fn default() -> Self {
        Self {
            insert: |_| unimplemented!(),
            insert_many: |_| unimplemented!(),
            update: |_| unimplemented!(),
            query: |_| unimplemented!(),
            query_by_rn: |_| unimplemented!(),
            lease_batch: |_, _, _, _| unimplemented!(),
            delete_batch: |_| unimplemented!(),
            reset_batch: || unimplemented!(),
        }
    }
}

#[async_trait]
impl Persistence for MockPgAdapter {
    type PersistenceError = PgAdapterError;
    async fn insert(&self, task: Task) -> Result<Task, Self::PersistenceError> {
        (self.insert)(task)
    }

    async fn insert_many(&self, tasks: Vec<Task>) -> Result<TaskBatch, Self::PersistenceError> {
        (self.insert_many)(tasks)
    }

    async fn update(&self, task: Task) -> Result<Task, Self::PersistenceError> {
        (self.update)(task)
    }

    async fn query(&self, get_task_model: &GetTaskModel) -> Result<Vec<Task>, Self::PersistenceError> {
        (self.query)(get_task_model)
    }

    async fn query_by_rn(&self, rn: String) -> Result<Task, Self::PersistenceError> {
        (self.query_by_rn)(rn)
    }

    async fn lease_batch(&self, queue: String, limit: i32, worker: String, task_timeout_in_secs: i64) -> Result<Vec<Task>, Self::PersistenceError> {
        (self.lease_batch)(queue, limit, worker, task_timeout_in_secs)
    }
    async fn delete_batch(&self, retention_time_in_secs: i64) -> Result<u64, Self::PersistenceError> {
        (self.delete_batch)(retention_time_in_secs)
    }
    async fn reset_batch(&self) -> Result<Vec<Task>, PgAdapterError> {
        (self.reset_batch)()
    }
}

#[tokio::test]
async fn pg_manager_mock_adapter() {
    let t_now = Utc::now();
    let task1 = Task {
        rn: "1".to_string(),
        priority: 1,
        queue: "a".to_string(),
        created: t_now,
        updated: t_now,
        ..Task::default()
    };

    let task2 = Task {
        rn: "2".to_string(),
        priority: 2,
        queue: "b".to_string(),
        created: t_now,
        updated: t_now,
        ..Task::default()
    };

    let task3 = Task {
        rn: "3".to_string(),
        priority: 3,
        queue: "c".to_string(),
        created: t_now,
        updated: t_now,
        ..Task::default()
    };

    let mock = MockPgAdapter::default()
        .on_insert(Ok)
        .on_insert_many(|tasks| {
            Ok(TaskBatch {
                inserted_tasks: tasks[1..].to_vec(),
                conflicting_tasks: vec![tasks[0].clone()],
            })
        })
        .on_query_by_rn(|_rn| {
            Ok(Task {
                rn: "query_by_rn".to_string(),
                ..Task::default()
            })
        })
        .on_query(|_gtm| {
            Ok(vec![Task {
                rn: "query".to_string(),
                ..Task::default()
            }])
        })
        .on_update(|_t| {
            Ok(Task {
                rn: "update".to_string(),
                ..Task::default()
            })
        })
        .on_reset_batch(|| {
            Ok(vec![Task {
                rn: "reset".to_string(),
                ..Task::default()
            }])
        });
    let pgm = PgManager { pg_adapter: Box::new(mock) };
    assert_eq!(pgm.fetch_task("rn".to_string()).await.unwrap().rn, "query_by_rn".to_string());
    assert_eq!(
        pgm.fetch_tasks(GetTaskModel {
            limit: None,
            queue: None,
            worker: None,
            status: None,
        })
        .await
        .unwrap()[0]
            .rn,
        "query".to_string()
    );
    assert_eq!(
        pgm.insert_task(AddTaskModel {
            rn: "add".to_string(),
            spec: serde_json::Value::default(),
            priority: 1,
            queue: "s".to_string(),
        })
        .await
        .unwrap()
        .rn,
        "add".to_string()
    );

    let insert_many_tasks_result = pgm
        .insert_tasks(vec![
            AddTaskModel {
                rn: task1.rn.clone(),
                spec: task1.spec.clone(),
                priority: task1.priority,
                queue: task1.queue.clone(),
            },
            AddTaskModel {
                rn: task2.rn.clone(),
                spec: task2.spec.clone(),
                priority: task2.priority,
                queue: task2.queue.clone(),
            },
            AddTaskModel {
                rn: task3.rn.clone(),
                spec: task3.spec.clone(),
                priority: task3.priority,
                queue: task3.queue.clone(),
            },
        ])
        .await
        .unwrap();

    assert_eq!(
        insert_many_tasks_result
            .inserted_tasks
            .iter()
            .map(|t| Task {
                created: t_now,
                updated: t_now,
                ..t.clone()
            })
            .collect::<Vec<Task>>(),
        vec![task2.clone(), task3.clone(),]
    );

    assert_eq!(
        insert_many_tasks_result
            .conflicting_tasks
            .iter()
            .map(|t| Task {
                created: t_now,
                updated: t_now,
                ..t.clone()
            })
            .collect::<Vec<Task>>(),
        vec![task1.clone()]
    );
    // update cases
    assert_eq!(pgm.lease_task("2".to_string(), "w".to_string(), None).await.unwrap().rn, "update".to_string());
    assert_eq!(pgm.cancel_task("2".to_string()).await.unwrap().rn, "update".to_string());
    // reset
    assert_eq!(pgm.reset_batch().await.unwrap().first().unwrap().rn, "reset".to_string());

    //heartbeat
    let mock = MockPgAdapter::default()
        .on_query_by_rn(|_rn| {
            Ok(Task {
                rn: "query_by_rn".to_string(),
                owner: Some("worker".to_string()),
                deadline: Some(Utc::now() - Duration::seconds(10)),
                status: TaskStatus::Running,
                ..Task::default()
            })
        })
        .on_update(|_t| {
            Ok(Task {
                rn: "update".to_string(),
                ..Task::default()
            })
        });
    let pgm = PgManager { pg_adapter: Box::new(mock) };
    assert_eq!(
        pgm.heartbeat_task("2".to_string(), "worker".to_string(), None, Some(5)).await.unwrap().rn,
        "update".to_string()
    );
    assert_eq!(pgm.complete_task("2".to_string(), None).await.unwrap().rn, "update".to_string());
    assert_eq!(
        pgm.abort_task(
            "2".to_string(),
            TaskError {
                code: "123".to_string(),
                args: serde_json::Value::default(),
                description: "sd".to_string(),
            },
        )
        .await
        .unwrap()
        .rn,
        "update".to_string()
    );
    assert_eq!(pgm.yield_task("2".to_string()).await.unwrap().rn, "update".to_string());
    assert_eq!(pgm.reset_task("2".to_string()).await.unwrap().rn, "update".to_string());
}
