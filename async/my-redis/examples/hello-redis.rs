use mini_redis::{client, Result};

// In tokio, a runtime is like a smart task manager that runs multiple tasks at the same time
// without blocking the main thread.

// The following attribute creates and starts a Tokio runtime. Async functions need to be run
// inside an async runtime. However, the main function is not async by default.
//
// Creates a runtime, runs async main function in the runtime, manages threads automatically
// based on the runtime configuration
#[tokio::main]
async fn main() -> Result<()> {
    // Open a connection to the mini-redis address
    let mut client = client::connect("127.0.0.1:6379").await?;

    // Set the key "hello" with value "world"
    client.set("hello", "world".into()).await?;

    // Get key "hello"
    let result = client.get("hello").await?;

    println!("Got value from the server; result={:?}", result);

    Ok(())
}
