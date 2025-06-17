use std::{fs, path::PathBuf, sync::Arc, thread, time::Instant};
use worker::processor::ImageProcessor;

fn main() {
    let images_dir = PathBuf::from("./examples/images");
    let processor = Arc::new(ImageProcessor::new());

    let mut paths = Vec::new();
    for entry in fs::read_dir(images_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        paths.push(path);
    }

    let start = Instant::now();
    let mut handles = Vec::new();

    for path in paths {
        let processor = Arc::clone(&processor);
        let handle = thread::spawn(move || {
            processor.process(&path);
        });

        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    let duration = start.elapsed();
    println!("All jobs completed in {:?}", duration);
}
