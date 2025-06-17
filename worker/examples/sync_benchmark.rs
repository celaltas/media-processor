use std::{path::Path, thread, time::Instant};
use worker::{processor::ImageProcessor, redis_service::RedisService};

fn main() {
    let processor = ImageProcessor::new();
    let file_number = 156;
    println!("🚀 Image process worker started sync manner ...");
    let start = Instant::now();
    let mut handles = Vec::new();
    for _i in 0..file_number {
        let p = processor.clone();
        let handle = thread::spawn(move || {
            let mut redis = RedisService::new();
            match redis.dequeue_job() {
                Ok(job_id) => {
                    println!("📦 Processing job: {}", job_id);
                    if let Err(err) = handle_job(&p, &mut redis, &job_id) {
                        eprintln!("❌ Error processing job {}: {}", job_id, err);

                        if let Err(e) = redis.enqueue_job(&job_id) {
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

    for handle in handles {
        handle.join().unwrap();
    }

    let duration = start.elapsed();
    println!("All jobs completed in {:?}", duration);
}

fn handle_job(
    processor: &ImageProcessor,
    redis_service: &mut RedisService,
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
