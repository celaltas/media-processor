use crate::{processor::ImageProcessor, redis_service::RedisService};
use std::{path::Path, time::Duration};
use tokio::time::sleep;

mod job;
mod processor;
mod redis_service;

#[tokio::main]
async fn main() {
    let image_processor = ImageProcessor::new();
    let mut redis_service = RedisService::new();

    println!("🚀 Image process worker started...");

    loop {
        match redis_service.dequeue_job() {
            Ok(job_id) => {
                println!("📦 Processing job: {}", job_id);
                if let Err(err) = handle_job(&image_processor, &mut redis_service, &job_id) {
                    eprintln!("❌ Error processing job {}: {}", job_id, err);

                    if let Err(e) = redis_service.enqueue_job(&job_id) {
                        eprintln!("❌ Failed to re-enqueue job {}: {}", job_id, e);
                    } else {
                        println!("🔁 Job {} re-enqueued due to failure", job_id);
                    }
                }
            }
            Err(err) => {
                eprintln!("⚠️ Failed to fetch job: {}", err);
                println!("⏳ Sleeping for 5 seconds...");
                sleep(Duration::from_secs(5)).await;
            }
        }
    }
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
