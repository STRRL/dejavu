use async_trait::async_trait;
use tokio::sync::Mutex;

use super::{EntityImage, EntityText, Repository};

pub struct InMemoryRepository {
    images: Mutex<Vec<EntityImage>>,
    texts: Mutex<Vec<EntityText>>,
}

impl InMemoryRepository {
    pub fn new() -> Self {
        Self {
            images: Mutex::new(vec![]),
            texts: Mutex::new(vec![]),
        }
    }
}

// implement Repository trait for InMemoryRepository
#[async_trait]
impl Repository for InMemoryRepository {
    async fn save_image(&self, entity: &EntityImage) -> anyhow::Result<EntityImage> {
        let mut entity = entity.clone();
        let length = self.images.lock().await.len();
        entity.id = length as u32;
        Ok(entity)
    }

    async fn get_image_by_id(&self, id: u32) -> anyhow::Result<EntityImage> {
        let guard = self.images.lock().await;
        let entity = guard
            .iter()
            .find(|it| it.id == id)
            .ok_or(anyhow::anyhow!("not found"))?;
        Ok(entity.clone())
    }

    async fn save_text(&self, entity: &EntityText) -> anyhow::Result<EntityText> {
        let mut entity = entity.clone();
        let mut guard = self.texts.lock().await;
        entity.id = guard.len() as u32;
        guard.push(entity.clone());
        Ok(entity)
    }

    async fn save_texts(&self, entities: &Vec<EntityText>) -> anyhow::Result<Vec<EntityText>> {
        let mut entities = entities.clone();
        for entity in entities.iter_mut() {
            let mut guard = self.texts.lock().await;
            entity.id = guard.len() as u32;
            guard.push(entity.clone());
        }
        Ok(entities)
    }
    async fn get_text_by_id(&self, id: u32) -> anyhow::Result<EntityText> {
        let entity = self
            .texts
            .lock()
            .await
            .iter()
            .find(|it| it.id == id)
            .cloned()
            .ok_or(anyhow::anyhow!("not found"))?;
        Ok(entity)
    }

    /// it's not a real full text search, just a simple filter for demo
    async fn full_text_search(&self, text: &str) -> anyhow::Result<Vec<EntityText>> {
        let entities = self
            .texts
            .lock()
            .await
            .iter()
            .filter(|it| it.text.contains(text))
            .cloned()
            .collect();
        Ok(entities)
    }
}
