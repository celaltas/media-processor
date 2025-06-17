use futures::future::join_all;
use std::{fs, path::PathBuf, time::Instant};
use tokio::task;
use worker::processor::ImageProcessor;

#[tokio::main]
async fn main() {
    let images_dir = PathBuf::from("./examples/images");
    let processor = ImageProcessor::new();
    let mut paths = Vec::new();
    for entry in fs::read_dir(images_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        paths.push(path);
    }

    let start = Instant::now();
    let futures = paths.into_iter().map(|path| {
        let processor = processor.clone();
        task::spawn_blocking(move || processor.process(&path))
    });
    join_all(futures).await;
    let duration = start.elapsed();
    println!("All jobs completed in {:?}", duration);
}
