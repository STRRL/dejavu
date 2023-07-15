#[cfg(feature = "in-memory")]
use {
    super::{ImageArchive, ImageArchiver},
    crate::screenshot::Screenshot,
    async_trait::async_trait,
    std::collections::HashMap,
    tokio::sync::Mutex,
    uuid::Uuid,
};

#[cfg(feature = "in-memory")]
pub struct InMemoryImageArchiver {
    // storage: HashMap<UUID, image::RgbImage>,
    pub storage: Mutex<HashMap<String, image::DynamicImage>>,
}

#[cfg(feature = "in-memory")]
impl InMemoryImageArchiver {
    pub fn new() -> Self {
        Self {
            storage: Mutex::new(HashMap::new()),
        }
    }
}
#[cfg(feature = "in-memory")]
#[async_trait]
impl ImageArchiver for InMemoryImageArchiver {
    async fn load(&self, image_archive: &ImageArchive) -> anyhow::Result<image::DynamicImage> {
        let storage = self.storage.lock().await;
        let image = storage.get(&image_archive.archive_detail);
        match image {
            Some(image) => Ok(image.clone()),
            None => Err(anyhow::anyhow!("image not found")),
        }
    }

    async fn archive(&self, screenshot: &Screenshot) -> anyhow::Result<ImageArchive> {
        let mut storage = self.storage.lock().await;
        let uuid = Uuid::new_v4().to_string();
        storage.insert(uuid.clone(), screenshot.image.clone());
        Ok(ImageArchive {
            archive_type: "in_memory".to_string(),
            archive_detail: uuid,
        })
    }
}
