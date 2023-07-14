use anyhow::Ok;
use async_trait::async_trait;
use image::DynamicImage;
use tracing::trace;

#[derive(Debug, Clone)]
pub struct RecognizeItem {
    pub text: String,
    pub markup: MarkupBox,
    pub level: u32,
}

impl RecognizeItem {
    pub fn new(text: String, markup: MarkupBox, level: u32) -> Self {
        Self {
            text,
            markup,
            level,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MarkupBox {
    pub left: u32,
    pub top: u32,
    pub width: u32,
    pub height: u32,
}

impl MarkupBox {
    pub fn new(left: u32, top: u32, width: u32, height: u32) -> Self {
        Self {
            left,
            top,
            width,
            height,
        }
    }
    pub fn new_i32(left: i32, top: i32, width: i32, height: i32) -> Self {
        Self {
            left: left as u32,
            top: top as u32,
            width: width as u32,
            height: height as u32,
        }
    }
}

#[async_trait]
pub trait CharacterRecognizer {
    async fn recognize(&self, image: image::RgbImage) -> anyhow::Result<Vec<RecognizeItem>>;
}

pub struct TesseractOCR {}

impl TesseractOCR {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl CharacterRecognizer for TesseractOCR {
    async fn recognize(&self, image: image::RgbImage) -> anyhow::Result<Vec<RecognizeItem>> {
        trace!("OCR Start");
        let default_args = rusty_tesseract::Args::default();
        let di = DynamicImage::ImageRgb8(image);
        let ri = rusty_tesseract::Image::from_dynamic_image(&di)?;
        let output = rusty_tesseract::image_to_data(&ri, &default_args).unwrap();
        trace!("OCR Complete");
        let result: Vec<RecognizeItem> = output
            .data
            .iter()
            .map(|x| {
                let text = x.text.clone();
                let markup = MarkupBox::new_i32(x.left, x.top, x.width, x.height);
                RecognizeItem::new(text, markup, x.level as u32)
            })
            .collect();
        Ok(result)
    }
}
