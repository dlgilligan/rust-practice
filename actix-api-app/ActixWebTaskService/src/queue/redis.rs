// Fixed queue/redis.rs with corrected receive_task method
use log::{error, info};
use redis::{AsyncCommands, Client, RedisError};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Serialize, Deserialize)]
pub struct TaskMessage {
    pub task_global_id: String,
}

pub struct RedisQueue {
    client: Client,
    queue_name: String,
}

impl RedisQueue {
    pub fn init() -> Result<Self, RedisError> {
        // Get Redis connection string from environment
        let redis_uri =
            env::var("REDIS_URI").unwrap_or_else(|_| "redis://localhost:6379".to_string());
        let queue_name = env::var("REDIS_QUEUE").unwrap_or_else(|_| "task_queue".to_string());

        // Create Redis client
        let client = match Client::open(redis_uri.clone()) {
            Ok(client) => {
                info!("Connected to Redis: {}", redis_uri);
                client
            }
            Err(e) => {
                error!("Failed to connect to Redis: {}", e);
                return Err(e);
            }
        };

        Ok(Self { client, queue_name })
    }

    pub async fn send_task(&self, task_global_id: String) -> Result<(), RedisError> {
        // Serialize task message
        let task_message = TaskMessage { task_global_id };
        let message = match serde_json::to_string(&task_message) {
            Ok(msg) => msg,
            Err(e) => {
                error!("Failed to serialize task message: {}", e);
                return Err(RedisError::from(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Serialization error",
                )));
            }
        };

        // Get Redis connection from the client
        let mut conn = match self.client.get_async_connection().await {
            Ok(conn) => conn,
            Err(e) => {
                error!("Failed to get Redis connection: {}", e);
                return Err(e);
            }
        };

        // Push the task message to the Redis list
        match conn.rpush(&self.queue_name, message).await {
            Ok(_) => {
                info!("Task sent to Redis queue: {}", task_global_id);
                Ok(())
            }
            Err(e) => {
                error!("Failed to send task to Redis queue: {}", e);
                Err(e)
            }
        }
    }

    pub async fn receive_task(
        &self,
        timeout_seconds: u64,
    ) -> Result<Option<TaskMessage>, RedisError> {
        // Get Redis connection from the client
        let mut conn = match self.client.get_async_connection().await {
            Ok(conn) => conn,
            Err(e) => {
                error!("Failed to get Redis connection: {}", e);
                return Err(e);
            }
        };

        // BLPOP blocks until a message is available or timeout is reached
        let result: Option<(String, String)> = conn
            .blpop(&self.queue_name, timeout_seconds as usize)
            .await?;

        // Process the result
        match result {
            Some((_, message)) => {
                // Deserialize the message
                match serde_json::from_str::<TaskMessage>(&message) {
                    Ok(task_message) => {
                        info!(
                            "Received task from Redis queue: {}",
                            task_message.task_global_id
                        );
                        Ok(Some(task_message))
                    }
                    Err(e) => {
                        error!("Failed to deserialize task message: {}", e);
                        Err(RedisError::from(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("Deserialization error: {}", e),
                        )))
                    }
                }
            }
            None => {
                // Timeout reached, no message available
                Ok(None)
            }
        }
    }
}
