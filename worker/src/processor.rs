use image::{ImageReader, imageops::FilterType};

use std::path::Path;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ImageProcessorError {
    #[error("Failed to open or decode image: {0}")]
    ImageLoadError(String),
    #[error("Failed to generate output path for image")]
    PathGenerationError,
    #[error("Failed to save processed image to path: {0}")]
    ImageSaveError(PathBuf),
}

#[derive(Clone, Copy)]
pub struct ImageProcessor;

impl ImageProcessor {
    pub fn new() -> Self {
        Self
    }

    pub fn process(&self, image_path: &Path) -> Result<(), ImageProcessorError> {
        let img = ImageReader::open(image_path)
            .map_err(|_e| ImageProcessorError::ImageLoadError(image_path.display().to_string()))?
            .decode()
            .map_err(|e| ImageProcessorError::ImageLoadError(e.to_string()))?;

        let (width, height) = (img.width(), img.height());
        let processed = img
            .blur(0.9)
            .resize(width * 2, height * 2, FilterType::Gaussian);

        let output_path = self
            .add_prefix_to_filename(image_path, "processed")
            .ok_or(ImageProcessorError::PathGenerationError)?;

        processed
            .save(&output_path)
            .map_err(|_| ImageProcessorError::ImageSaveError(output_path))?;

        println!("✨ Processed and saved successfully!");
        Ok(())
    }

    fn add_prefix_to_filename(&self, path: &Path, prefix: &str) -> Option<std::path::PathBuf> {
        let file_stem = path.file_stem()?.to_str()?;
        let ext = path.extension().and_then(|e| e.to_str());

        let new_name = match ext {
            Some(ext) => format!("{prefix}{file_stem}.{ext}"),
            None => format!("{prefix}{file_stem}"),
        };

        path.parent().map(|p| p.join(new_name))
    }
}
