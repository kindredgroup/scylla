// $coverage:ignore-start
//! Ignored from coverage because of real database interactions. covered as part of component tests
//! Adapter to implement database operations.
use crate::adapter_utils::{
    handle_insert_return, handle_query_by_rn_return, handle_update_return, prepare_insert_task, prepare_query_task, prepare_update_task,
};
use crate::error::PgAdapterError;
use async_trait::async_trait;
use deadpool_postgres::{Client, Pool};
use scylla_models::{GetTaskModel, Task};
use scylla_operations::task::Persistence;
use serde_json::from_value;
use tokio_postgres::types::ToSql;

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
        order by data ->> 'priority' asc, data -> 'created' asc
        limit $4::Int
      ";
const GET_TASK_SQL: &str = "
        Select data::JSONB from task \
        where data ->> 'rn' = $1 \
      ";

pub struct PgAdapter {
    pub pool: Pool,
}

#[async_trait]
pub trait DbExecute {
    async fn execute(&self, sql: &str, params: &[&(dyn ToSql + Sync)]) -> Result<Vec<Task>, PgAdapterError>;
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
}
