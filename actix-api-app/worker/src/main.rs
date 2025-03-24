use anyhow::{Context, Result};
use log::{error, info};
use redis::AsyncCommands;
use redis::Client as RedisClient;
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;
use tokio::time;

// Import the TaskMessage from the Redis queue module
#[derive(Serialize, Deserialize)]
struct TaskMessage {
    task_global_id: String,
}

#[derive(Serialize, Deserialize)]
struct TaskCompletionRequest {
    result_file: String,
}

#[derive(Serialize, Deserialize)]
struct Task {
    user_uuid: String,
    task_uuid: String,
    task_type: String,
    state: String,
    source_file: String,
    result_file: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct TaskIdentifier {
    task_global_id: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    // Get configuration from environment variables
    let redis_uri = env::var("REDIS_URI").unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let queue_name = env::var("REDIS_QUEUE").unwrap_or_else(|_| "task_queue".to_string());
    let api_base_url =
        env::var("API_BASE_URL").unwrap_or_else(|_| "http://localhost:80".to_string());

    // HTTP client for API calls
    let http_client = HttpClient::new();

    // Redis client for fetching tasks
    let redis_client =
        RedisClient::open(redis_uri.clone()).context("Failed to connect to Redis")?;

    info!("Worker service started");

    // Main processing loop
    loop {
        let process_result =
            process_next_task(&redis_client, &queue_name, &http_client, &api_base_url).await;

        if let Err(err) = process_result {
            error!("Error processing task: {:?}", err);
            // Wait before retrying after an error
            time::sleep(Duration::from_secs(5)).await;
        }
    }
}

// Function to process the next task from the queue
async fn process_next_task(
    redis_client: &RedisClient,
    queue_name: &str,
    http_client: &HttpClient,
    api_base_url: &str,
) -> Result<()> {
    // Get Redis connection
    let mut conn = redis_client
        .get_async_connection()
        .await
        .context("Failed to get Redis connection")?;

    // BLPOP blocks until a message is available or timeout is reached
    let result: Option<(String, String)> = conn
        .blpop(queue_name, 20)
        .await
        .context("Error executing BLPOP command")?;

    if let Some((_, message)) = result {
        // Deserialize the message
        let task_message: TaskMessage =
            serde_json::from_str(&message).context("Failed to deserialize task message")?;

        // Process the task
        process_task(http_client, api_base_url, &task_message.task_global_id).await?;

        Ok(())
    } else {
        // No message received within timeout, return without error
        Ok(())
    }
}

async fn process_task(http_client: &HttpClient, api_base_url: &str, task_id: &str) -> Result<()> {
    info!("Processing task: {}", task_id);

    // 1. Update task state to InProgress
    update_task_state(http_client, api_base_url, task_id, "start")
        .await
        .context("Failed to update task state to InProgress")?;

    // 2. Get task details
    let task = get_task(http_client, api_base_url, task_id)
        .await
        .context("Failed to get task details")?;

    // 3. Process the task
    info!("Processing source file: {}", task.source_file);

    // This is where the actual task processing/rendering would happen
    match execute_task_processing(&task).await {
        Ok(result_file) => {
            // 4. Complete the task
            complete_task(http_client, api_base_url, task_id, &result_file)
                .await
                .context("Failed to complete task")?;
            info!("Task completed: {}", task_id);
        }
        Err(err) => {
            error!("Task processing failed: {:?}", err);
            // 4. Mark task as failed
            update_task_state(http_client, api_base_url, task_id, "fail")
                .await
                .context("Failed to update task state to failed")?;
        }
    }

    Ok(())
}

async fn get_task(http_client: &HttpClient, api_base_url: &str, task_id: &str) -> Result<Task> {
    let url = format!("{}/task/{}", api_base_url, task_id);
    let response = http_client
        .get(&url)
        .send()
        .await
        .context("Failed to send GET request")?;

    let task = response
        .json::<Task>()
        .await
        .context("Failed to parse task JSON")?;

    Ok(task)
}

async fn update_task_state(
    http_client: &HttpClient,
    api_base_url: &str,
    task_id: &str,
    action: &str,
) -> Result<()> {
    let url = format!("{}/task/{}/{}", api_base_url, task_id, action);
    http_client
        .put(&url)
        .send()
        .await
        .context(format!("Failed to send PUT request to {}", action))?;

    Ok(())
}

async fn complete_task(
    http_client: &HttpClient,
    api_base_url: &str,
    task_id: &str,
    result_file: &str,
) -> Result<()> {
    let url = format!("{}/task/{}/complete", api_base_url, task_id);
    let request = TaskCompletionRequest {
        result_file: result_file.to_string(),
    };

    http_client
        .put(&url)
        .json(&request)
        .send()
        .await
        .context("Failed to send complete task request")?;

    Ok(())
}

// This function would contain your actual task processing logic
async fn execute_task_processing(task: &Task) -> Result<String> {
    // Simulate processing time
    time::sleep(Duration::from_secs(2)).await;

    // Here we would:
    // 1. Download the source file from S3 or other storage
    // 2. Process it (e.g., render 3D model)
    // 3. Upload the result to storage
    // 4. Return the path to the result file

    // For this example, we'll just return a dummy result file path
    let result_file = format!("processed_{}.result", task.task_uuid);

    Ok(result_file)
}
