#[cfg(feature = "in-memory")]
use {async_trait::async_trait, tokio::sync::Mutex, super::{EntityImage, EntityWord, Repository}};

#[cfg(feature = "in-memory")]
pub struct InMemoryRepository {
    images: Mutex<Vec<EntityImage>>,
    texts: Mutex<Vec<EntityText>>,
}

#[cfg(feature = "in-memory")]
impl InMemoryRepository {
    pub fn new() -> Self {
        Self {
            images: Mutex::new(vec![]),
            texts: Mutex::new(vec![]),
        }
    }
}

// implement Repository trait for InMemoryRepository
#[cfg(feature = "in-memory")]
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

    async fn save_texts(&self, entities: &[EntityText]) -> anyhow::Result<Vec<EntityText>> {
        let mut result = Vec::new();
        for entity in entities.iter() {
            let mut guard = self.texts.lock().await;
            let mut entity = entity.clone();
            entity.id = guard.len() as u32;
            guard.push(entity.clone());
            result.push(entity);
        }
        Ok(result)
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
