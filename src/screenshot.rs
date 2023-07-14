use std::sync::{Arc, Mutex};

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
        let result: Vec<RgbImage> = Vec::new();

        let result_mutex = Arc::new(Mutex::new(result));
        // capture all screens concurrently
        let mut tasks = Vec::new();

        for screen in screens {
            let result_mutex = result_mutex.clone();
            let t = tokio::task::spawn_blocking(move || {
                let capture = screen.capture().unwrap();
                let image = screen_image_2_image_image(capture).unwrap();
                result_mutex.lock().unwrap().push(image);
            });
            tasks.push(t);
        }

        // join all tasks
        for task in tasks {
            task.await?;
        }
    
        let result = Arc::try_unwrap(result_mutex).unwrap().into_inner().unwrap();
        Ok(result)
    }
}

fn screen_image_2_image_image(screen_image: Image) -> anyhow::Result<RgbImage> {
    let buffer = screen_image.to_png()?;
    let image = image::load_from_memory(&buffer)?.into_rgb8();
    Ok(image)
}
