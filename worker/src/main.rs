use r2d2_redis::{RedisConnectionManager, r2d2};
use std::{
    path::Path,
    thread::{self, sleep},
    time::Duration,
};
use worker::{processor::ImageProcessor, redis_service_pool::RedisServicePooledCon};

fn main() {
    let image_processor = ImageProcessor::new();
    let num_of_threads = thread::available_parallelism()
        .map(|r| r.get())
        .unwrap_or(1);

    let manager =
        RedisConnectionManager::new("redis://default:secret_passwd@localhost:6379/0").unwrap();
    let connection_pool = r2d2::Pool::builder()
        .max_size(num_of_threads as u32)
        .build(manager)
        .unwrap();

    let mut handles = Vec::new();
    for _i in 0..num_of_threads {
        let connection_pool_clone = connection_pool.clone();
        let handle = thread::spawn(move || {
            let mut actual_conn = connection_pool_clone.get().unwrap();
            let mut redis_conn = RedisServicePooledCon::new(&mut actual_conn);
            loop {
                match redis_conn.dequeue_job() {
                    Ok(job_id) => {
                        println!("📦 Processing job: {}", job_id);
                        if let Err(err) = handle_job(&image_processor, &mut redis_conn, &job_id) {
                            eprintln!("❌ Error processing job {}: {}", job_id, err);

                            if let Err(e) = redis_conn.enqueue_job(&job_id) {
                                eprintln!("❌ Failed to re-enqueue job {}: {}", job_id, e);
                            } else {
                                println!("🔁 Job {} re-enqueued due to failure", job_id);
                            }
                        }
                    }
                    Err(err) => {
                        eprintln!("⚠️ Failed to fetch job: {}", err);
                        println!("⏳ Sleeping for 5 seconds...");
                        sleep(Duration::from_secs(5));
                    }
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

fn handle_job(
    processor: &ImageProcessor,
    redis_service: &mut RedisServicePooledCon,
    job_id: &str,
) -> Result<(), String> {
    let metadata = redis_service
        .get_job_metadata(job_id)
        .map_err(|e| format!("Failed to fetch metadata: {}", e))?;

    let metadata = metadata.ok_or_else(|| format!("Metadata not found for job: {}", job_id))?;
    let path = Path::new(&metadata.path);

    processor
        .process(path)
        .map_err(|e| format!("Image processing error: {}", e))?;

    redis_service
        .update_job_status(job_id, "processed")
        .map_err(|e| format!("Failed to update job status: {}", e))?;

    println!("✅ Successfully processed job {}", job_id);
    Ok(())
}
