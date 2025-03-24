use crate::model::task::{Task, TaskState};
use bson::{doc, Document};
use log::{error, info};
use mongodb::{
    error::Error as MongoDBError,
    options::{ClientOptions, FindOneOptions},
    Client, Collection,
};
use std::env;
use std::error::Error;
use std::fmt;
use std::str::FromStr;

// Improved error handling with enum
#[derive(Debug)]
pub enum MongoRepoError {
    ConnectionError(MongoDBError),
    QueryError(MongoDBError),
    InsertError(MongoDBError),
    UpdateError(MongoDBError),
    DeserializationError(String),
    InvalidTaskState(String),
    NotFound,
}

impl fmt::Display for MongoRepoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConnectionError(e) => write!(f, "MongoDB connection error: {}", e),
            Self::QueryError(e) => write!(f, "MongoDB query error: {}", e),
            Self::InsertError(e) => write!(f, "MongoDB insert error: {}", e),
            Self::UpdateError(e) => write!(f, "MongoDB update error: {}", e),
            Self::DeserializationError(msg) => write!(f, "Failed to deserialize document: {}", msg),
            Self::InvalidTaskState(msg) => write!(f, "Invalid task state: {}", msg),
            Self::NotFound => write!(f, "Document not found"),
        }
    }
}

impl Error for MongoRepoError {}

// Convert from MongoDBError to our custom error types
impl From<MongoDBError> for MongoRepoError {
    fn from(error: MongoDBError) -> Self {
        // This is a simplified conversion - in a real application,
        // you might want to inspect the error to determine the correct variant
        MongoRepoError::QueryError(error)
    }
}

#[derive(Clone)]
pub struct MongoRepository {
    collection: Collection<Document>,
}

impl MongoRepository {
    pub async fn init() -> Result<Self, MongoRepoError> {
        // Get MongoDB connection string from environment
        let mongo_uri = env::var("MONGO_URI")
            .unwrap_or_else(|_| "mongodb://admin:password@localhost:27017".to_string());
        let db_name = env::var("MONGO_DB").unwrap_or_else(|_| "task_service".to_string());
        let collection_name = env::var("MONGO_COLLECTION").unwrap_or_else(|_| "tasks".to_string());

        // Parse a connection string into options
        let client_options = ClientOptions::parse(&mongo_uri)
            .await
            .map_err(|e| MongoRepoError::ConnectionError(e))?;

        // Create a new client and connect to the server
        let client =
            Client::with_options(client_options).map_err(|e| MongoRepoError::ConnectionError(e))?;

        // Get a handle to the database and collection
        let database = client.database(&db_name);
        let collection = database.collection::<Document>(&collection_name);

        info!("Connected to MongoDB: {}", mongo_uri);

        Ok(Self { collection })
    }

    pub async fn put_task(&self, task: Task) -> Result<(), MongoRepoError> {
        let task_id = task.get_global_id();

        // Convert Task to Document
        let doc = doc! {
            "user_uuid": task.user_uuid,
            "task_uuid": task.task_uuid,
            "task_global_id": task_id.clone(),
            "task_type": task.task_type,
            "state": task.state.to_string(),
            "source_file": task.source_file,
            "result_file": task.result_file,
        };

        // Use upsert to update if exists or insert if not
        let filter = doc! { "task_global_id": &task_id };
        let options = mongodb::options::UpdateOptions::builder()
            .upsert(true)
            .build();

        match self
            .collection
            .update_one(filter, doc! { "$set": doc }, options)
            .await
        {
            Ok(result) => {
                info!(
                    "Task saved to MongoDB: {} (matched: {}, modified: {}, upserted: {})",
                    task_id,
                    result.matched_count,
                    result.modified_count,
                    result.upserted_id.is_some()
                );
                Ok(())
            }
            Err(e) => {
                error!("Failed to save task to MongoDB: {}", e);
                Err(MongoRepoError::UpdateError(e))
            }
        }
    }

    pub async fn get_task(&self, task_id: String) -> Option<Task> {
        let filter = doc! { "task_global_id": task_id.clone() };
        let options = FindOneOptions::builder().build();

        match self.collection.find_one(filter, options).await {
            Ok(Some(doc)) => match self.document_to_task(&doc) {
                Ok(task) => {
                    info!("Retrieved task from MongoDB: {}", task_id);
                    Some(task)
                }
                Err(e) => {
                    error!("Failed to convert document to task: {}", e);
                    None
                }
            },
            Ok(None) => {
                info!("Task not found: {}", task_id);
                None
            }
            Err(e) => {
                error!("Error finding task: {}", e);
                None
            }
        }
    }

    fn document_to_task(&self, doc: &Document) -> Result<Task, MongoRepoError> {
        // Extract fields from document with better error messages
        let user_uuid = doc
            .get_str("user_uuid")
            .map_err(|_| {
                MongoRepoError::DeserializationError("Missing or invalid user_uuid".into())
            })?
            .to_string();

        let task_uuid = doc
            .get_str("task_uuid")
            .map_err(|_| {
                MongoRepoError::DeserializationError("Missing or invalid task_uuid".into())
            })?
            .to_string();

        let task_type = doc
            .get_str("task_type")
            .map_err(|_| {
                MongoRepoError::DeserializationError("Missing or invalid task_type".into())
            })?
            .to_string();

        let state_str = doc
            .get_str("state")
            .map_err(|_| MongoRepoError::DeserializationError("Missing or invalid state".into()))?;

        let state = TaskState::from_str(state_str)
            .map_err(|_| MongoRepoError::InvalidTaskState(state_str.to_string()))?;

        let source_file = doc
            .get_str("source_file")
            .map_err(|_| {
                MongoRepoError::DeserializationError("Missing or invalid source_file".into())
            })?
            .to_string();

        // Optional field
        let result_file = match doc.get_str("result_file") {
            Ok(val) => Some(val.to_string()),
            Err(_) => None,
        };

        Ok(Task {
            user_uuid,
            task_uuid,
            task_type,
            state,
            source_file,
            result_file,
        })
    }
}
