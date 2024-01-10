// $coverage:ignore-start
//! Ignored from coverage because of real database interactions. covered as part of component tests
//! Adapter to implement database operations.
use crate::adapter_utils::{
    handle_insert_return, handle_query_by_rn_return, handle_update_return, prepare_insert_task, prepare_query_task, prepare_update_task,
};
use crate::error::PgAdapterError;
use async_trait::async_trait;
use chrono::{Duration, Utc};
use deadpool_postgres::{Client, Pool};
use scylla_models::{GetTaskModel, Task, TaskHistory, TaskHistoryType};
use scylla_operations::task::Persistence;
use serde_json::{from_value, json};
use tokio_postgres::types::{Json, ToSql};

const INSERT_TASK_SQL: &str = "
    INSERT INTO task(data) VALUES ($1) \
    ON CONFLICT ((data->>'rn')) \
    DO NOTHING
    RETURNING data::JSONB
  ";
const UPDATE_TASK_SQL: &str = "
    UPDATE task SET data = data || $1 where data ->> 'rn' = $2 returning data
  ";
const GET_TASKS_SQL: &str = "
        Select data::JSONB from task \
        where data ->> 'status' like $1 \
        AND data ->> 'queue' like $2 \
        AND (('%' = $3 OR data ->> 'owner' like $3))
        order by data ->> 'priority' desc, data -> 'created' desc
        limit $4::Int
      ";
const GET_TASK_SQL: &str = "
        Select data::JSONB from task \
        where data ->> 'rn' = $1 \
      ";
const UPDATE_BATCH_TASK_SQL: &str = "
    WITH tasks AS ( Select data::JSONB from task \
        where data ->> 'status' like 'ready' \
        AND data ->> 'queue' like $1 \
        order by data ->> 'priority' desc, data -> 'created' desc
        limit $2::Int)
    UPDATE task t SET data = jsonb_set(jsonb_set(jsonb_set(jsonb_set( \
            jsonb_set(t.data, '{status}', '\"running\"'), \
         '{owner}', $3), '{deadline}', $4), '{updated}', $5), '{history, 100}', $6) from tasks where t.data ->> 'rn' = tasks.data ->> 'rn' returning t.data
  ";

const DELETE_BATCH_TASK_SQL: &str = "
    DELETE from task where data ->> 'status' in ('completed', 'cancelled', 'aborted') AND data ->> 'updated' < $1
";

pub struct PgAdapter {
    pub pool: Pool,
}

#[async_trait]
pub trait DbExecute {
    async fn execute(&self, sql: &str, params: &[&(dyn ToSql + Sync)]) -> Result<Vec<Task>, PgAdapterError>;
    async fn execute_count(&self, sql: &str, params: &[&(dyn ToSql + Sync)]) -> Result<u64, PgAdapterError>;
}

#[async_trait]
impl DbExecute for PgAdapter {
    async fn execute(&self, sql: &str, params: &[&(dyn ToSql + Sync)]) -> Result<Vec<Task>, PgAdapterError> {
        let client: Client = self.pool.get().await?;
        let stmt = client.prepare_cached(sql).await?;
        let rows = client.query(&stmt, params).await?;
        Ok(rows
            .into_iter()
            .map(|row| {
                let task_value: Task = from_value(row.get(0)).unwrap();
                task_value
            })
            .collect())
    }
    async fn execute_count(&self, sql: &str, params: &[&(dyn ToSql + Sync)]) -> Result<u64, PgAdapterError> {
        let client: Client = self.pool.get().await?;
        let stmt = client.prepare_cached(sql).await?;
        let rows = client.execute(&stmt, params).await?;
        Ok(rows)
    }
}

#[async_trait]
impl Persistence for PgAdapter {
    type PersistenceError = PgAdapterError;

    async fn insert(&self, task: Task) -> Result<Task, PgAdapterError> {
        let execute_resp = &self.execute(INSERT_TASK_SQL, &[&prepare_insert_task(&task)]).await?;
        let t = handle_insert_return(execute_resp, &task)?;
        Ok(t.clone())
    }

    async fn update(&self, task: Task) -> Result<Task, PgAdapterError> {
        let up = prepare_update_task(&task);
        let execute_resp = &self.execute(UPDATE_TASK_SQL, &[&up.json_task, &up.rn]).await?;
        let t = handle_update_return(execute_resp, &task)?;
        Ok(t.clone())
    }

    async fn query(&self, get_task_model: &GetTaskModel) -> Result<Vec<Task>, PgAdapterError> {
        let qp = prepare_query_task(get_task_model);
        self.execute(GET_TASKS_SQL, &[&qp.status, &qp.queue, &qp.worker, &qp.limit]).await
    }

    async fn query_by_rn(&self, rn: String) -> Result<Task, PgAdapterError> {
        let execute_resp = &self.execute(GET_TASK_SQL, &[&rn]).await?;
        let t = handle_query_by_rn_return(execute_resp, &rn)?;
        Ok(t.clone())
    }

    async fn update_batch(&self, queue: String, limit: i32, worker: String, task_timeout_in_secs: i64) -> Result<Vec<Task>, Self::PersistenceError> {
        let deadline = Json(json!(Utc::now() + Duration::seconds(task_timeout_in_secs)));
        let updated = Json(json!(Utc::now()));
        let worker_json = Json(json!(worker));
        let task_history = Json(json!(TaskHistory {
            typ: TaskHistoryType::Assignment,
            time: Utc::now(),
            worker: worker.clone(),
            progress: Some(0.0),
        }));
        self.execute(UPDATE_BATCH_TASK_SQL, &[&queue, &limit, &worker_json, &deadline, &updated, &task_history])
            .await
    }

    async fn delete_batch(&self, retention_time_in_secs: i64) -> Result<u64, Self::PersistenceError> {
        let deletion_time = format!("{:?}", Utc::now() - Duration::seconds(retention_time_in_secs));
        self.execute_count(DELETE_BATCH_TASK_SQL, &[&deletion_time]).await
    }
}
