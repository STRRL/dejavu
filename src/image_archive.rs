use std::collections::HashMap;

use async_trait::async_trait;
use tokio::sync::Mutex;
use uuid::Uuid;

pub struct ImageArchive {
    pub archive_type: String,
    pub archive_detail: String,
}

#[async_trait]
pub trait ImageArchiver {
    async fn load(&self, image_archive: &ImageArchive) -> anyhow::Result<image::RgbImage>;
    async fn archive(&self, image: &image::RgbImage) -> anyhow::Result<ImageArchive>;
}

pub struct InMemoryImageArchiver {
    // storage: HashMap<UUID, image::RgbImage>,
    pub storage: Mutex<HashMap<String, image::RgbImage>>,
}

impl InMemoryImageArchiver {
    pub fn new() -> Self {
        Self {
            storage: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl ImageArchiver for InMemoryImageArchiver {
    async fn load(&self, image_archive: &ImageArchive) -> anyhow::Result<image::RgbImage> {
        let storage = self.storage.lock().await;
        let image = storage.get(&image_archive.archive_detail);
        match image {
            Some(image) => Ok(image.clone()),
            None => Err(anyhow::anyhow!("image not found")),
        }
    }

    async fn archive(&self, image: &image::RgbImage) -> anyhow::Result<ImageArchive> {
        let mut storage = self.storage.lock().await;
        let uuid = Uuid::new_v4().to_string();
        storage.insert(uuid.clone(), image.clone());
        Ok(ImageArchive {
            archive_type: "in_memory".to_string(),
            archive_detail: uuid,
        })
    }
}
