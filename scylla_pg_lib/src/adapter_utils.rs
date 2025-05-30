use crate::error::PgAdapterError;
use scylla_models::{GetTaskModel, Task};
use serde_json::to_value;

/// # Panics
/// In case task cannot be converted to `serde_json::Value`
pub fn prepare_insert_task(task: &Task) -> serde_json::Value {
    to_value(task).unwrap()
}
/// # Errors
/// Returns `PgAdapterError::DuplicateTask` Error
/// # Panics
/// In case return count is more than 1
pub fn handle_insert_return<'a>(tasks: &'a [Task], original_task: &Task) -> Result<&'a Task, PgAdapterError> {
    match tasks.len() {
        0 => Err(PgAdapterError::DuplicateTask(original_task.rn.clone())),
        1 => Ok(&tasks[0]),
        _ => panic!("Unexpected number of rows returned from insert query"),
    }
}
#[derive(PartialEq, Eq, Debug)]
pub struct UpdateParams {
    pub json_task: serde_json::Value,
    pub rn: String,
}
/// # Panics
/// In case task cannot be converted to `serde_json::Value`
pub fn prepare_update_task(task: &Task) -> UpdateParams {
    UpdateParams {
        rn: task.rn.clone(),
        json_task: to_value(task).unwrap(),
    }
}
/// # Errors
/// Returns `PgAdapterError::NoTaskFound` Error
/// # Panics
/// In case return count is more than 1
pub fn handle_update_return<'a>(tasks: &'a [Task], original_task: &'a Task) -> Result<&'a Task, PgAdapterError> {
    match tasks.len() {
        0 => Err(PgAdapterError::NoTaskFound(original_task.rn.clone())),
        1 => Ok(&tasks[0]),
        _ => panic!("Unexpected number of rows returned from update query"),
    }
}
#[derive(PartialEq, Eq, Debug)]
pub struct QueryParams {
    pub status: String,
    pub queue: String,
    pub worker: String,
    pub limit: i32,
}
pub fn prepare_query_task(get_task_model: &GetTaskModel) -> QueryParams {
    let status = get_task_model.status.clone().map_or_else(|| '%'.to_string(), |s| s.to_string().to_lowercase());
    let queue = get_task_model.queue.clone().map_or_else(|| '%'.to_string(), |q| q);
    let worker = get_task_model.worker.clone().map_or_else(|| '%'.to_string(), |w| w);
    let limit = get_task_model.limit.map_or_else(|| 100, |l| l);
    QueryParams { status, queue, worker, limit }
}
/// # Errors
/// Returns `PgAdapterError::NoTaskFound` Error
/// # Panics
/// In case return count is more than 1
pub fn handle_query_by_rn_return<'a>(tasks: &'a [Task], rn: &str) -> Result<&'a Task, PgAdapterError> {
    match tasks.len() {
        0 => Err(PgAdapterError::NoTaskFound(rn.to_owned())),
        1 => Ok(&tasks[0]),
        _ => panic!("Unexpected number of rows returned from query by rn query"),
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

        assert!(handle_insert_return(&[], &original_task).is_err());
        assert_eq!(
            handle_insert_return(&[], &original_task).unwrap_err().to_string(),
            PgAdapterError::DuplicateTask(original_task.rn.to_owned()).to_string()
        );
        let ret_t = Task {
            rn: "123".to_string(),
            status: scylla_models::TaskStatus::Running,
            ..Task::default()
        };
        // In case single item is returned from db. That will be retruned back
        assert_eq!(*handle_insert_return(&[ret_t.clone()], &original_task).unwrap(), ret_t);
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
        handle_insert_return(&vec![ret_t, ret_t1], &original_task).unwrap();
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
            }
        )
    }

    #[test]
    fn handle_update_return_cases() {
        let original_t = Task {
            rn: "123".to_string(),
            ..Task::default()
        };
        let empty_vec = Vec::new();
        let resp = handle_update_return(&empty_vec, &original_t);
        assert!(resp.is_err());
        assert_eq!(resp.unwrap_err().to_string(), PgAdapterError::NoTaskFound(original_t.rn.to_owned()).to_string());
        let ret_t = Task {
            rn: "123".to_string(),
            status: TaskStatus::Running,
            ..Task::default()
        };
        assert_eq!(*handle_update_return(&[ret_t.clone()], &original_t).unwrap(), ret_t)
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
        handle_update_return(&vec![ret_t, ret_t1], &original_task).unwrap();
    }

    #[test]
    fn prepare_query_task_cases() {
        // default values
        let gtm = GetTaskModel {
            status: None,
            queue: None,
            worker: None,
            limit: None,
        };
        assert_eq!(
            prepare_query_task(&gtm),
            QueryParams {
                status: "%".to_string(),
                queue: "%".to_string(),
                worker: "%".to_string(),
                limit: 100
            }
        );
        // Everything passed
        let gtm = GetTaskModel {
            status: Some(TaskStatus::Cancelled),
            queue: Some("abc".to_string()),
            worker: Some("s".to_string()),
            limit: Some(20),
        };
        assert_eq!(
            prepare_query_task(&gtm),
            QueryParams {
                status: "cancelled".to_string(),
                queue: "abc".to_string(),
                worker: "s".to_string(),
                limit: 20
            }
        );
    }
    #[test]
    fn handle_query_by_rn_return_cases() {
        let original_t = Task {
            rn: "123".to_string(),
            ..Task::default()
        };
        let empty_vec = Vec::new();
        let resp = handle_query_by_rn_return(&empty_vec, &original_t.rn);
        assert!(resp.is_err());
        assert_eq!(resp.unwrap_err().to_string(), PgAdapterError::NoTaskFound(original_t.rn.to_owned()).to_string());
        let ret_t = Task {
            rn: "123".to_string(),
            status: TaskStatus::Running,
            ..Task::default()
        };
        assert_eq!(*handle_query_by_rn_return(&[ret_t.clone()], &original_t.rn).unwrap(), ret_t)
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
        handle_query_by_rn_return(&vec![ret_t, ret_t1], &original_task.rn).unwrap();
    }
}
