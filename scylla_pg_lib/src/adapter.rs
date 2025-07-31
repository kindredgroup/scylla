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
use log::debug;
use scylla_models::{GetTaskModel, Task, TaskHistory, TaskHistoryType};
use scylla_operations::task::Persistence;
use serde_json::{from_value, json};
use tokio_postgres::error::SqlState;
use tokio_postgres::types::{Json, ToSql};
use tokio_postgres::IsolationLevel;

const CONST_DELAY: u64 = 10;
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
const LEASE_N_TASK_SQL: &str = "
    UPDATE task t SET data = jsonb_set(jsonb_set(jsonb_set(jsonb_set( \
            jsonb_set(t.data, '{status}', '\"running\"'), \
         '{owner}', $3), '{deadline}', $4), '{updated}', $5), '{history, 100}', $6) where t.data ->> 'rn' IN (Select data::JSONB ->> 'rn' from task \
        where data ->> 'status' = 'ready' \
        AND data ->> 'queue' like $1 \
        order by data ->> 'priority' desc, data -> 'created' asc
        limit $2::Int FOR UPDATE SKIP LOCKED) returning t.data";

const RESET_BATCH_TASK_SQL: &str = "
        UPDATE task t SET data = jsonb_set(t.data, '{history, 100}', jsonb_build_object(
	'typ', 'TaskTimeout',
	'time', to_char(timezone('UTC'::text, now()), 'YYYY-MM-DD HH:MI:SS.MSZ'),
	'worker', t.data->>'owner',
	'progress', (t.data->>'progress')::float
      )) || jsonb_build_object('progress', 0, 'status', 'ready', 'owner', null, 'deadline', null, 'updated', to_char(timezone('UTC'::text, now()), 'YYYY-MM-DD HH:MI:SS.MSZ'))  \
             where t.data ->> 'deadline' < $1 AND t.data ->> 'status' = 'running' \
             returning t.data";

const DELETE_BATCH_TASK_SQL: &str = "
    DELETE from task where data ->> 'status' in ('completed', 'cancelled', 'aborted') AND data ->> 'updated' < $1
";

pub struct PgAdapter {
    pub pool: Pool,
}

#[async_trait]
pub trait DbExecute {
    async fn execute(&self, sql: &str, params: &[&(dyn ToSql + Sync)], isolation_level: IsolationLevel) -> Result<Vec<Task>, PgAdapterError>;
    async fn execute_count(&self, sql: &str, params: &[&(dyn ToSql + Sync)], isolation_level: IsolationLevel) -> Result<u64, PgAdapterError>;
}

#[async_trait]
impl DbExecute for PgAdapter {
    async fn execute(&self, sql: &str, params: &[&(dyn ToSql + Sync)], isolation_level: IsolationLevel) -> Result<Vec<Task>, PgAdapterError> {
        let max_tries = 10;
        let mut try_count = 1;
        let mut tasks: Option<Vec<Task>> = None;
        let error: Option<PgAdapterError>;
        loop {
            let mut client: Client = self.pool.get().await?;
            let stmt = client.prepare_cached(sql).await?;
            let tx = client.build_transaction().isolation_level(isolation_level).start().await?;
            match tx.query(&stmt, params).await {
                Ok(rows) => {
                    debug!("row count : {} returned from query : {} for params: {:?}", rows.len(), sql, params);
                    tasks = Some(
                        rows.into_iter()
                            .map(|row| {
                                let task_value: Task = from_value(row.get(0)).unwrap();
                                task_value
                            })
                            .collect(),
                    );
                    if let Err(e) = tx.commit().await {
                        try_count += 1;
                        log::error!("commit for tx failed : {}", e.to_string());
                    } else {
                        error = None;
                        break;
                    }
                }
                Err(e) => {
                    if let Err(err) = tx.rollback().await {
                        log::error!("rollback for tx failed : {}", err.to_string());
                    }
                    if try_count == max_tries {
                        error = Some(PgAdapterError::DbError(e));
                        break;
                    } else {
                        match e.code() {
                            Some(&SqlState::T_R_SERIALIZATION_FAILURE) => {
                                let random_delay: u64 = rand::random_range(((try_count - 1) * 10 * (try_count - 1))..(try_count * 10 * try_count));
                                // log::error!("delay : {}, try_count: {try_count}", CONST_DELAY + random_delay);
                                tokio::time::sleep(std::time::Duration::from_millis(CONST_DELAY + random_delay)).await;
                                try_count += 1;
                            }
                            _ => {
                                log::error!("Error processing transaction {:?}", e);
                                error = Some(PgAdapterError::DbError(e));
                                break;
                            }
                        }
                    }
                }
            }
        }
        if let Some(e) = error {
            return Err(e);
        }
        return Ok(tasks.unwrap());
    }
    async fn execute_count(&self, sql: &str, params: &[&(dyn ToSql + Sync)], isolation_level: IsolationLevel) -> Result<u64, PgAdapterError> {
        let mut client: Client = self.pool.get().await?;
        let stmt = client.prepare_cached(sql).await?;
        let tx = client.build_transaction().isolation_level(isolation_level).start().await?;
        
        match tx.execute(&stmt, params).await {
            Ok(rows) => {
                match tx.commit().await {
                    Ok(_) => Ok(rows),
                    Err(commit_err) => {
                        log::error!("commit for tx failed: {}", commit_err);
                        Err(PgAdapterError::DbError(commit_err))
                    }
                }
            }
            Err(e) => {
                if let Err(rollback_err) = tx.rollback().await {
                    log::error!("rollback for tx failed: {}", rollback_err);
                }
                Err(PgAdapterError::DbError(e))
            }
        }
    }
}

#[async_trait]
impl Persistence for PgAdapter {
    type PersistenceError = PgAdapterError;

    async fn insert(&self, task: Task) -> Result<Task, PgAdapterError> {
        let execute_resp = &self
            .execute(INSERT_TASK_SQL, &[&prepare_insert_task(&task)], IsolationLevel::RepeatableRead)
            .await?;
        let t = handle_insert_return(execute_resp, &task)?;
        Ok(t.clone())
    }

    async fn update(&self, task: Task) -> Result<Task, PgAdapterError> {
        let up = prepare_update_task(&task);
        let execute_resp = &self.execute(UPDATE_TASK_SQL, &[&up.json_task, &up.rn], IsolationLevel::RepeatableRead).await?;
        let t = handle_update_return(execute_resp, &task)?;
        Ok(t.clone())
    }

    async fn query(&self, get_task_model: &GetTaskModel) -> Result<Vec<Task>, PgAdapterError> {
        let qp = prepare_query_task(get_task_model);
        self.execute(GET_TASKS_SQL, &[&qp.status, &qp.queue, &qp.worker, &qp.limit], IsolationLevel::RepeatableRead)
            .await
    }

    async fn query_by_rn(&self, rn: String) -> Result<Task, PgAdapterError> {
        let execute_resp = &self.execute(GET_TASK_SQL, &[&rn], IsolationLevel::RepeatableRead).await?;
        let t = handle_query_by_rn_return(execute_resp, &rn)?;
        Ok(t.clone())
    }

    async fn lease_batch(&self, queue: String, limit: i32, worker: String, task_timeout_in_secs: i64) -> Result<Vec<Task>, Self::PersistenceError> {
        let deadline = Json(json!(Utc::now() + Duration::seconds(task_timeout_in_secs)));
        let updated = Json(json!(Utc::now()));
        let worker_json = Json(json!(worker));
        let task_history = Json(json!(TaskHistory {
            typ: TaskHistoryType::Assignment,
            time: Utc::now(),
            worker: worker.clone(),
            progress: Some(0.0),
        }));

        self.execute(
            LEASE_N_TASK_SQL,
            &[&queue, &limit, &worker_json, &deadline, &updated, &task_history],
            IsolationLevel::ReadCommitted,
        )
        .await
    }

    async fn delete_batch(&self, retention_time_in_secs: i64) -> Result<u64, Self::PersistenceError> {
        let deletion_time = format!("{:?}", Utc::now() - Duration::seconds(retention_time_in_secs));
        self.execute_count(DELETE_BATCH_TASK_SQL, &[&deletion_time], IsolationLevel::RepeatableRead)
            .await
    }

    async fn reset_batch(&self) -> Result<Vec<Task>, Self::PersistenceError> {
        let deadline = format!("{:?}", Utc::now());

        self.execute(RESET_BATCH_TASK_SQL, &[&deadline], IsolationLevel::RepeatableRead).await
    }
}

// impl PgAdapter {
//     async fn retry_operation(&self, count: i32, operation: Box<dyn Fn() -> Result<Vec<Task>, Self::PersistenceError>>) -> Result<Vec<Task>, Self::PersistenceError> {}
// }
