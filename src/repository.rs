use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct EntityImage {
    pub id: u32,
    pub archive_type: String,
    pub archive_info: String,
}

impl EntityImage {
    pub fn new(id: u32, archive_type: String, archive_info: String) -> Self {
        Self {
            id,
            archive_type,
            archive_info,
        }
    }
}

#[derive(Debug, Clone)]
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
    async fn save_text(&mut self, entity: &EntityText) -> anyhow::Result<EntityText>;
    async fn save_texts(&mut self, entities: &Vec<EntityText>) -> anyhow::Result<Vec<EntityText>>;
    async fn get_text_by_id(&self, id: u32) -> anyhow::Result<EntityText>;
    async fn full_text_search(&self, text: &str) -> anyhow::Result<Vec<EntityText>>;
}

pub struct InMemoryRepository {
    images: Vec<EntityImage>,
    texts: Vec<EntityText>,
}

impl InMemoryRepository {
    pub fn new() -> Self {
        Self {
            images: Vec::new(),
            texts: Vec::new(),
        }
    }
}

// implement Repository trait for InMemoryRepository
#[async_trait]
impl Repository for InMemoryRepository {
    async fn save_image(&self, entity: &EntityImage) -> anyhow::Result<EntityImage> {
        let mut entity = entity.clone();
        entity.id = self.images.len() as u32;
        Ok(entity)
    }

    async fn get_image_by_id(&self, id: u32) -> anyhow::Result<EntityImage> {
        let entity = self
            .images
            .iter()
            .find(|it| it.id == id)
            .ok_or(anyhow::anyhow!("not found"))?;
        Ok(entity.clone())
    }

    async fn save_text(&mut self, entity: &EntityText) -> anyhow::Result<EntityText> {
        let mut entity = entity.clone();
        entity.id = self.texts.len() as u32;
        // append to self.texts
        self.texts.push(entity.clone());
        Ok(entity)
    }

    async fn save_texts(&mut self, entities: &Vec<EntityText>) -> anyhow::Result<Vec<EntityText>> {
        let mut entities = entities.clone();
        for entity in entities.iter_mut() {
            entity.id = self.texts.len() as u32;
            // append to self.texts
            self.texts.push(entity.clone());
        }
        Ok(entities)
    }

    async fn get_text_by_id(&self, id: u32) -> anyhow::Result<EntityText> {
        let entity = self
            .texts
            .iter()
            .find(|it| it.id == id)
            .ok_or(anyhow::anyhow!("not found"))?;
        Ok(entity.clone())
    }

    /// it's not a real full text search, just a simple filter for demo
    async fn full_text_search(&self, text: &str) -> anyhow::Result<Vec<EntityText>> {
        let entities = self
            .texts
            .iter()
            .filter(|it| it.text.contains(text))
            .cloned()
            .collect();
        Ok(entities)
    }
}
