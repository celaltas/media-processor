use crossbeam_channel::{Receiver, Sender, select};
use r2d2_redis::{
    RedisConnectionManager,
    r2d2::{self, Pool},
};
use std::{
    path::Path,
    sync::Arc,
    thread::{self, sleep},
    time::Duration,
};
use worker::{
    processor::ImageProcessor, redis_service_pool::RedisServicePooledCon, threadpool::ThreadPool,
};

fn main() {
    let (shutdown_tx, shutdown_rx) = crossbeam_channel::bounded::<()>(1);
    let (tx, rx) = crossbeam_channel::bounded::<String>(100);
    setup_signal_handlers(shutdown_tx);
    let num_of_threads = thread::available_parallelism()
        .map(|r| r.get())
        .unwrap_or(1);
    let manager =
        RedisConnectionManager::new("redis://default:secret_passwd@localhost:6379/0").unwrap();
    let connection_pool = Arc::new(
        r2d2::Pool::builder()
            .max_size(num_of_threads as u32)
            .build(manager)
            .unwrap(),
    );
    let pool_ref_1 = Arc::clone(&connection_pool);
    let pool_ref_2 = Arc::clone(&connection_pool);
    start_listener_thread(pool_ref_1, tx, shutdown_rx);
    start_worker_threads(pool_ref_2, rx, num_of_threads);
}

fn setup_signal_handlers(shutdown_tx: Sender<()>) {
    ctrlc::set_handler(move || {
        println!("🛑 Received Ctrl+C signal, initiating graceful shutdown...");
        if let Err(e) = shutdown_tx.send(()) {
            eprintln!("❌ Failed to send shutdown signal: {}", e);
        }
    })
    .expect("Error setting Ctrl-C handler");
    println!("📡 Signal handlers registered. Press Ctrl+C to shutdown gracefully.");
}

fn start_worker_threads(
    pool: Arc<Pool<RedisConnectionManager>>,
    receiver: Receiver<String>,
    num_of_threads: usize,
) {
    let image_processor = ImageProcessor::new();
    let threadpool = ThreadPool::new(num_of_threads);
    loop {
        let message = receiver.recv();
        match message {
            Ok(job_id) => {
                let pool_clone = Arc::clone(&pool);
                threadpool.execute(move || {
                    let mut connection = match pool_clone.get() {
                        Ok(conn) => conn,
                        Err(e) => {
                            eprintln!("❌ Listener failed to get Redis connection: {}", e);
                            return;
                        }
                    };
                    let mut redis_service = RedisServicePooledCon::new(&mut connection);
                    if let Err(err) = handle_job(&image_processor, &mut redis_service, &job_id) {
                        eprintln!("❌ Error processing job {}: {}", job_id, err);

                        if let Err(e) = redis_service.enqueue_job(&job_id) {
                            eprintln!("❌ Failed to re-enqueue job {}: {}", job_id, e);
                        } else {
                            println!("🔁 Job {} re-enqueued due to failure", job_id);
                        }
                    }
                });
            }
            Err(_) => {
                println!("Disconnected; shutting down.");
                break;
            }
        }
    }
}

fn start_listener_thread(
    pool: Arc<Pool<RedisConnectionManager>>,
    sender: Sender<String>,
    shutdown_rx: Receiver<()>,
) {
    thread::spawn(move || {
        let mut connection = match pool.get() {
            Ok(conn) => conn,
            Err(e) => {
                eprintln!("❌ Listener failed to get Redis connection: {}", e);
                return;
            }
        };

        let mut redis_service = RedisServicePooledCon::new(&mut connection);

        loop {
            select! {
                recv(shutdown_rx) -> _ => {
                    println!("🛑 Listener received shutdown signal, stopping...");
                    drop(sender);
                    break;
                }
                default => {
                    // Continue with normal job processing
                    match redis_service.dequeue_job() {
                        Ok(job_id) => {
                            println!("📦 Getting job: {}", job_id);
                            if let Err(_) = sender.send(job_id) {
                                println!("📤 Job channel closed, listener shutting down...");
                                break;
                            }
                        }
                        Err(err) => {
                            eprintln!("⚠️ Failed to fetch job: {}", err);
                            println!("⏳ Sleeping for 5 seconds...");
                            sleep(Duration::from_secs(5));
                        }
                    }
                }
            }
        }
    });
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
