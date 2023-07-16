use async_trait::async_trait;
use std::io::Cursor;

use crate::screenshot::Screenshot;

use super::{ImageArchive, ImageArchiver};
pub struct FileSystemImageArchiver {
    storage_path: String,
}
impl FileSystemImageArchiver {
    pub fn new(storage_path: String) -> Self {
        Self { storage_path }
    }
}

#[async_trait]
impl ImageArchiver for FileSystemImageArchiver {
    async fn load(&self, image_archive: &ImageArchive) -> anyhow::Result<image::DynamicImage> {
        let path = format!("{}/{}", self.storage_path, image_archive.archive_detail);
        let image = image::open(path)?;
        Ok(image)
    }

    async fn archive(&self, screenshot: &Screenshot) -> anyhow::Result<ImageArchive> {
        // filename format YYYY-MM-DD-HH-MM-SS
        let filename = chrono::Local::now().format("%Y-%m-%d-%H-%M-%S").to_string();
        let filename =
            format!("{}-{}.jpg", filename.clone(), screenshot.metadata.screen_id).to_string();
        let path = format!("{}/{}", self.storage_path, filename);
        // encode the image as JPG
        let mut buffer = Cursor::new(Vec::new());
        screenshot
            .image
            .write_to(&mut buffer, image::ImageOutputFormat::Jpeg(80))?;
        let buffer = buffer.into_inner();
        tokio::fs::write(path, buffer).await?;
        Ok(ImageArchive {
            archive_type: "file_system".to_string(),
            archive_detail: filename,
        })
    }
}
