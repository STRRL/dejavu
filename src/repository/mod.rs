use async_trait::async_trait;
use serde::{Deserialize, Serialize};
pub mod in_memory;
pub mod sqlite;

#[derive(Debug, Clone)]
pub struct EntityImage {
    pub id: u32,
    pub archive_type: String,
    pub archive_info: String,
    pub captured_at_epoch: u64,
}

impl EntityImage {
    pub fn new(
        id: u32,
        archive_type: String,
        archive_info: String,
        captured_at_epoch: u64,
    ) -> Self {
        Self {
            id,
            archive_type,
            archive_info,
            captured_at_epoch,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityText {
    pub id: u32,
    pub image_id: u32,
    pub text: String,
    pub left: u32,
    pub top: u32,
    pub width: u32,
    pub height: u32,
}

impl EntityText {
    pub fn new(
        id: u32,
        image_id: u32,
        text: String,
        left: u32,
        top: u32,
        width: u32,
        height: u32,
    ) -> Self {
        Self {
            id,
            image_id,
            text,
            left,
            top,
            width,
            height,
        }
    }
}

impl TryFrom<&crate::ocr::RecognizeItem> for EntityText {
    type Error = anyhow::Error;

    fn try_from(value: &crate::ocr::RecognizeItem) -> anyhow::Result<Self> {
        let value = value.clone();
        Ok(Self::new(
            0,
            0,
            value.text,
            value.markup.left,
            value.markup.top,
            value.markup.width,
            value.markup.height,
        ))
    }
}

#[async_trait]
pub trait Repository {
    async fn save_image(&self, entity: &EntityImage) -> anyhow::Result<EntityImage>;
    async fn get_image_by_id(&self, id: u32) -> anyhow::Result<EntityImage>;
    async fn save_text(&self, entity: &EntityText) -> anyhow::Result<EntityText>;
    async fn save_texts(&self, entities: &[EntityText]) -> anyhow::Result<Vec<EntityText>>;
    async fn get_text_by_id(&self, id: u32) -> anyhow::Result<EntityText>;
    async fn full_text_search(&self, text: &str) -> anyhow::Result<Vec<EntityText>>;
}
