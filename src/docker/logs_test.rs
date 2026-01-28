//! Test log fetching to diagnose freeze

use bollard::container::LogsOptions;
use futures::StreamExt;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    
    let docker = bollard::Docker::connect_with_defaults().unwrap();
    
    // List containers
    let containers = docker.list_containers::<String>(None).await.unwrap();
    
    if let Some(container) = containers.first() {
        let id = container.id.as_ref().unwrap();
        println!("Testing logs for container: {}", &id[..12]);
        
        let options = LogsOptions::<String> {
            stdout: true,
            stderr: true,
            timestamps: false,
            follow: false,
            tail: "10".to_string(),
            ..Default::default()
        };
        
        println!("Creating logs stream...");
        let start = std::time::Instant::now();
        
        let mut stream = docker.logs(id, Some(options));
        println!("Stream created in {:?}", start.elapsed());
        
        let mut count = 0;
        loop {
            match timeout(Duration::from_secs(1), stream.next()).await {
                Ok(Some(Ok(log))) => {
                    println!("Got log: {:?}", log);
                    count += 1;
                }
                Ok(Some(Err(e))) => {
                    println!("Error: {}", e);
                    break;
                }
                Ok(None) => {
                    println!("Stream ended");
                    break;
                }
                Err(_) => {
                    println!("Timeout after {:?}", start.elapsed());
                    break;
                }
            }
        }
        
        println!("Total logs: {}", count);
    } else {
        println!("No containers found");
    }
}
