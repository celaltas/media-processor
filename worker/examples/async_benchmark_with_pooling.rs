use deadpool::managed::{Pool, PoolConfig};
use futures::future::join_all;
use std::{path::Path, sync::Arc, time::Instant};
use tokio::{sync::Semaphore, task};
use worker::{connection_manager::RedisConnectionManager, processor::ImageProcessor};

#[tokio::main]
async fn main() {
    let processor = ImageProcessor::new();
    let client = redis::Client::open("redis://default:secret_passwd@localhost:6379/0").unwrap();
    let connection_pool = Arc::new(
        Pool::<RedisConnectionManager>::builder(RedisConnectionManager::new(client))
            .config(PoolConfig::default())
            .max_size(156)
            .build()
            .expect("Failed to create connection pool"),
    );
    let file_number = 156;
    // let semaphore = Arc::new(Semaphore::new(12));
    println!("🚀 Image process worker started async manner ...");
    let start = Instant::now();
    let mut handles = Vec::new();
    for _i in 0..file_number {
        let p = processor.clone();
        let pool = Arc::clone(&connection_pool);
        // let semaphore = Arc::clone(&semaphore);
        let handle = task::spawn(async move {
            let mut redis = pool.get().await.unwrap();
            // let _permit = semaphore.acquire().await.unwrap();
            match redis.dequeue_job().await {
                Ok(job_id) => {
                    println!("📦 Processing job: {}", job_id);
                    if let Err(err) = handle_job(&p, &mut redis, &job_id).await {
                        eprintln!("❌ Error processing job {}: {}", job_id, err);

                        if let Err(e) = redis.enqueue_job(&job_id).await {
                            eprintln!("❌ Failed to re-enqueue job {}: {}", job_id, e);
                        } else {
                            println!("🔁 Job {} re-enqueued due to failure", job_id);
                        }
                    }
                }
                Err(err) => {
                    eprintln!("⚠️ Failed to fetch job: {}", err);
                }
            };
        });

        handles.push(handle);
    }
    join_all(handles).await;
    let duration = start.elapsed();
    println!("All jobs completed in {:?}", duration);
}

async fn handle_job(
    processor: &ImageProcessor,
    redis_service: &mut deadpool::managed::Object<RedisConnectionManager>,
    job_id: &str,
) -> Result<(), String> {
    let metadata = redis_service
        .get_job_metadata(job_id)
        .await
        .map_err(|e| format!("Failed to fetch metadata: {}", e))?;

    let metadata = metadata.ok_or_else(|| format!("Metadata not found for job: {}", job_id))?;
    let path = Path::new(&metadata.path);

    let processor = processor.clone();
    let path = path.to_owned();

    task::spawn_blocking(move || processor.process(&path))
        .await
        .map_err(|e| format!("Task join error: {}", e))?
        .map_err(|e| format!("Image processing error: {}", e))?;

    redis_service
        .update_job_status(job_id, "processed")
        .await
        .map_err(|e| format!("Failed to update job status: {}", e))?;

    println!("✅ Successfully processed job {}", job_id);
    Ok(())
}
