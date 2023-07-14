use async_trait::async_trait;

use crate::screenshot::Screenshot;

pub mod fs;
pub mod in_memory;

pub struct ImageArchive {
    pub archive_type: String,
    pub archive_detail: String,
}

#[async_trait]
pub trait ImageArchiver {
    async fn load(&self, image_archive: &ImageArchive) -> anyhow::Result<image::DynamicImage>;
    async fn archive(&self, screenshot: &Screenshot) -> anyhow::Result<ImageArchive>;
}
