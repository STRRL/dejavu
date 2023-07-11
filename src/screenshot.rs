use async_trait::async_trait;
use image::RgbImage;
use screenshots::{Image, Screen};

#[async_trait]
pub trait Capturer {
    /// Capture the contents of all the screens, returning a vector of images.
   async fn capture(&self) -> anyhow::Result<Vec<RgbImage>>;
}

pub struct DefaultCapturer {}

impl DefaultCapturer {
    pub fn new() -> Self {
        DefaultCapturer {}
    }
}

#[async_trait]
impl Capturer for DefaultCapturer {
    async fn capture(&self) -> anyhow::Result<Vec<RgbImage>> {
        let screens = Screen::all()?;
        let mut result: Vec<RgbImage> = Vec::new();

        for screen in screens {
            let capture = screen.capture()?;
            let image = screen_image_2_image_image(capture)?;
            result.push(image);
        }

        Ok(result)
    }
}

fn screen_image_2_image_image(screen_image: Image) -> anyhow::Result<RgbImage> {
    let buffer = screen_image.to_png()?;
    let image = image::load_from_memory(&buffer)?.into_rgb8();
    Ok(image)
}
