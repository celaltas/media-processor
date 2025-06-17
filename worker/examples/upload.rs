use std::{
    fs::{self, File},
    io::Read,
    path::PathBuf,
};

use reqwest::blocking::{self, multipart::Part};

fn main() {
    let images_dir = PathBuf::from("./examples/images");
    let url = "http://localhost:8080/upload";
    for entry in fs::read_dir(images_dir).unwrap() {
        let entry = entry.unwrap();
        let mut file = File::open(entry.path()).unwrap();
        let mut content = Vec::new();
        file.read_to_end(&mut content).unwrap();
        let filename = entry.file_name().to_str().unwrap().to_owned();
        let part = Part::bytes(content).file_name(filename);
        let form = blocking::multipart::Form::new().part("file", part);
        let response = blocking::Client::new()
            .post(url)
            .multipart(form)
            .send()
            .unwrap();
        println!("Status: {}", response.status());
    }
}
