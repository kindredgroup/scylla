use scylla_operations::task::ScyllaOperations;

use crate::{adapter::PgAdapter, error::PgAdapterError};

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
  impl ScyllaOperations for MockPgAdapter {
  
    type PersistenceError = PgAdapterError;
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
async fn test_pg() {
    let pgm = PgManager {
        pg_adapter : Box::new(MockPgAdapter::default())
    };
   // pgm.fetch_task("123".to_string()).await;
    //assert_eq!(pgm.fetch_task("123".to_string()).await, );
    
}