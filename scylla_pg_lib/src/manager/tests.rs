// $coverage:ignore-start
use chrono::{Duration, Utc};
use scylla_operations::task::Persistence;

use crate::error::PgAdapterError;

use super::PgManager;
use async_trait::async_trait;
use scylla_models::*;

struct MockPgAdapter {
    insert: fn(Task) -> Result<Task, PgAdapterError>,
    update: fn(Task) -> Result<Task, PgAdapterError>,
    query: fn(&GetTaskModel) -> Result<Vec<Task>, PgAdapterError>,
    query_by_rn: fn(String) -> Result<Task, PgAdapterError>,
}
impl MockPgAdapter {
    fn on_insert(mut self, f: fn(Task) -> Result<Task, PgAdapterError>) -> Self {
        self.insert = f;
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
}

impl Default for MockPgAdapter {
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
impl Persistence for MockPgAdapter {
    type PersistenceError = PgAdapterError;
    async fn insert(&self, task: Task) -> Result<Task, Self::PersistenceError> {
        (self.insert)(task)
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
}

#[tokio::test]
async fn pg_manager_mock_adapter() {
    let mock = MockPgAdapter::default()
        .on_insert(Ok)
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
        });
    let pgm = PgManager { pg_adapter: Box::new(mock) };
    assert_eq!(pgm.fetch_task("rn".to_string()).await.unwrap().rn, "query_by_rn".to_string());
    assert_eq!(
        pgm.fetch_tasks(GetTaskModel {
            limit: None,
            queue: None,
            worker: None,
            status: None
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
            queue: "s".to_string()
        })
        .await
        .unwrap()
        .rn,
        "add".to_string()
    );
    // update cases
    assert_eq!(pgm.lease_task("2".to_string(), "w".to_string(), None).await.unwrap().rn, "update".to_string());
    assert_eq!(pgm.cancel_task("2".to_string()).await.unwrap().rn, "update".to_string());

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
    assert_eq!(pgm.heartbeat_task("2".to_string(), None, Some(5)).await.unwrap().rn, "update".to_string());
    assert_eq!(pgm.complete_task("2".to_string()).await.unwrap().rn, "update".to_string());
    assert_eq!(
        pgm.abort_task(
            "2".to_string(),
            TaskError {
                code: "123".to_string(),
                args: serde_json::Value::default(),
                description: "sd".to_string()
            }
        )
        .await
        .unwrap()
        .rn,
        "update".to_string()
    );
    assert_eq!(pgm.yield_task("2".to_string()).await.unwrap().rn, "update".to_string());
    assert_eq!(pgm.reset_task("2".to_string()).await.unwrap().rn, "update".to_string());
}
