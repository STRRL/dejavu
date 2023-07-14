use anyhow::Result;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{
    image_archive::{ImageArchiver},
    ocr::{CharacterRecognizer, RecognizeItem},
    repository::{EntityImage, EntityText, Repository},
    screenshot::Screenshot,
};

pub struct Analysis {
    ocr: Box<dyn CharacterRecognizer + Send + Sync>,
    repo: Box<dyn Repository + Send + Sync>,
    archiver: Box<dyn ImageArchiver + Send + Sync>,
}

impl Analysis {
    pub fn new(
        ocr: Box<dyn CharacterRecognizer + Send + Sync>,
        repo: Box<dyn Repository + Send + Sync>,
        archiver: Box<dyn ImageArchiver + Send + Sync>,
    ) -> Self {
        Self {
            ocr,
            repo,
            archiver,
        }
    }

    pub async fn record_screenshot(&self, screenshot: &Screenshot) -> Result<()> {
        let archive = self.archiver.archive(&screenshot).await?;
        let entity_image = EntityImage::new(0, archive.archive_type, archive.archive_detail);
        let entity_image = self.repo.save_image(&entity_image).await?;

        let ocr_result: Vec<RecognizeItem> = self.ocr.recognize(&screenshot.image).await?;
        let entity_texts: Vec<EntityText> = ocr_result
            .iter()
            .filter(|it| it.level == 5)
            .filter_map(|it: &RecognizeItem| -> Option<EntityText> { it.try_into().ok() })
            .map(|mut it| {
                it.image_id = entity_image.id;
                it
            })
            .collect();
        self.repo.save_texts(&entity_texts).await?;
        Ok(())
    }

    pub async fn search(&self, text: &str) -> Result<Vec<SearchResult>> {
        let texts = self.repo.full_text_search(text).await?;
        let result: Vec<SearchResult> = texts
            .into_iter()
            .group_by(|it| it.image_id)
            .into_iter()
            .map(|(image_id, group)| SearchResult::new(image_id, group.collect()))
            .collect();
        Ok(result)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub image_id: u32,
    pub texts: Vec<EntityText>,
}

impl SearchResult {
    pub fn new(image_id: u32, texts: Vec<EntityText>) -> Self {
        Self { image_id, texts }
    }
}
