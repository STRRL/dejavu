use std::sync::Arc;

use image::DynamicImage;

use crate::{
    analysis::{Analysis, SearchResult},
    http::error::HttpError,
    image_archive::{ImageArchive, ImageArchiver},
    markup::ImageMarkupDecorator,
    ocr::MarkupBox,
    repository::Repository,
};

/// Adhoc service layer for web server
pub struct Service {
    analysis: Arc<Analysis>,
    markup_decorator: Arc<ImageMarkupDecorator>,
    repo: Arc<dyn Repository + Send + Sync>,
    image_archiver: Arc<dyn ImageArchiver + Send + Sync>,
}

impl Service {
    pub fn new(
        analysis: Arc<Analysis>,
        markup_decorator: Arc<ImageMarkupDecorator>,
        repo: Arc<dyn Repository + Send + Sync>,
        image_archiver: Arc<dyn ImageArchiver + Send + Sync>,
    ) -> Self {
        Self {
            analysis,
            markup_decorator,
            repo,
            image_archiver,
        }
    }

    pub async fn search(&self, text: &str) -> Result<Vec<SearchResult>, HttpError> {
        let result = self.analysis.search(text).await?;
        Ok(result)
    }

    pub async fn fetch_image_with_markup(
        &self,
        image_id: u32,
        text_ids: &Vec<u32>,
    ) -> Result<DynamicImage, HttpError> {
        let entity_image = self.repo.get_image_by_id(image_id).await?;
        let image_archive = ImageArchive::new(entity_image.archive_type, entity_image.archive_info);
        let loaded = self.image_archiver.load(&image_archive).await?;
        let mut markups = Vec::new();

        for text_id in text_ids {
            let entity_text = self.repo.get_word_by_id(*text_id).await?;
            let markup_box = MarkupBox::new(
                entity_text.left,
                entity_text.top,
                entity_text.width,
                entity_text.height,
            );
            markups.push(markup_box);
        }
        let marked = self
            .markup_decorator
            .markup_recognition(&loaded, &markups)?;

        Ok(marked)
    }
}
