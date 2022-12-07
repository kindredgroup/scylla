use async_trait::async_trait;
use deadpool_postgres::{Client, Pool};
use scylla_models::{GetTaskModel, Task};
use scylla_operations::task::{ScyllaOperations};
// use postgres::Row;
use serde_json::{from_value, to_value};
use tokio_postgres::types::ToSql;
use crate::error::PgAdapterError;
#[cfg(test)]
use mockall::{automock, mock, predicate::*};

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
#[cfg_attr(test, automock)]
pub trait DbExecute {
  async fn execute(&self, sql: &str, params: &[&(dyn ToSql + Sync)]) -> Result<Vec<Task>, PgAdapterError>;
}

#[async_trait]
impl DbExecute for PgAdapter {
  async fn execute(&self, sql: &str, params: &[&(dyn ToSql + Sync)]) -> Result<Vec<Task>, PgAdapterError> {
    let client: Client = self.pool.get().await?;
    let stmt = client.prepare_cached(sql).await?;
    let rows = client.query(&stmt, params).await?;
    Ok(
      rows
        .into_iter()
        .map(|row| {
          let task_value: Task = from_value(row.get(0)).unwrap();
          task_value
        })
        .collect(),
    )
  }
}


fn prepare_insert_task(task: &Task) -> serde_json::Value {
  to_value(task).unwrap()
}
fn handle_insert_return(tasks: Vec<Task>, original_task: &Task) -> Result<Task, PgAdapterError> {
  if tasks.is_empty() {
    return Err(PgAdapterError::DuplicateTask(original_task.rn.to_owned()));
  } else {
    return Ok(tasks[0].clone())
  }
// Returning 1st item in case more than 1 is returned from insert query. 
}

struct UpdateParams {
  json_task: serde_json::Value,
  rn: String
}
fn prepare_update_task(task: &Task) -> UpdateParams {
  UpdateParams {
    rn: task.rn.clone(),
    json_task: to_value(task).unwrap()
  }
}
fn handle_update_return(tasks: Vec<Task>, original_task: &Task) -> Result<Task, PgAdapterError> {
  if tasks.is_empty() {
    return Err(PgAdapterError::NoTaskFound(original_task.rn.to_owned()));
  }
  Ok(tasks[0].clone())
}
struct QueryParams {
  status: String,
  queue: String,
  worker: String,
  limit: i32
}
fn prepare_query_task(get_task_model: &GetTaskModel) -> QueryParams {
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
fn handle_query_by_rn_return(tasks: Vec<Task>, rn: &String) -> Result<Task, PgAdapterError> {
  if tasks.is_empty() {
    return Err(PgAdapterError::NoTaskFound(rn.to_owned()));
  }
  Ok(tasks[0].clone())
}

#[async_trait]
impl ScyllaOperations for PgAdapter {

  type PersistenceError = PgAdapterError;

  async fn insert(&self, task: Task) -> Result<Task, PgAdapterError> {
    handle_insert_return(
      self.execute(INSERT_TASK_SQL, &[&prepare_insert_task(&task)]).await?,
      &task
    )   
  }

  async fn update(&self, task: Task) -> Result<Task, PgAdapterError> {
    let up = prepare_update_task(&task);
    handle_update_return(self.execute(UPDATE_TASK_SQL, &[&up.json_task, &up.rn]).await?, &task)
  }

  async fn query(&self, get_task_model: &GetTaskModel) -> Result<Vec<Task>, PgAdapterError> {
    let qp = prepare_query_task(get_task_model);
    self.execute(GET_TASKS_SQL, &[&qp.status, &qp.queue, &qp.worker, &qp.limit]).await   
  }

  async fn query_by_rn(&self, rn: String) -> Result<Task, PgAdapterError> {
    handle_query_by_rn_return(self.execute(GET_TASK_SQL, &[&rn]).await?, &rn)
  }
}
#[cfg(test)]
mod tests;
