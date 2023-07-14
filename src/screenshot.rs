use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use image::DynamicImage;
use screenshots::{Image, Screen};

#[async_trait]
pub trait Capturer {
    /// Capture the contents of all the screens, returning a vector of images.
    async fn capture(&self) -> anyhow::Result<Vec<Screenshot>>;
}

pub struct DefaultCapturer {}

impl DefaultCapturer {
    pub fn new() -> Self {
        DefaultCapturer {}
    }
}

#[derive(Debug)]
pub struct Metadata {
    pub screen_id: u32,
}

#[derive(Debug)]
pub struct Screenshot {
    pub image: DynamicImage,
    pub metadata: Metadata,
}

#[async_trait]
impl Capturer for DefaultCapturer {
    async fn capture(&self) -> anyhow::Result<Vec<Screenshot>> {
        let screens = Screen::all()?;
        let result: Vec<Screenshot> = Vec::new();

        let result_mutex = Arc::new(Mutex::new(result));
        // capture all screens concurrently
        let mut tasks = Vec::new();

        for screen in screens {
            let result_mutex = result_mutex.clone();
            let t = tokio::task::spawn_blocking(move || {
                let capture = screen.capture().unwrap();
                let image = screen_image_2_image_image(capture).unwrap();
                let item = Screenshot {
                    image,
                    metadata: Metadata {
                        screen_id: screen.display_info.id,
                    },
                };
                result_mutex.lock().unwrap().push(item);
            });
            tasks.push(t);
        }

        // join all tasks
        for task in tasks {
            task.await?;
        }
    
        // collect results
        let result = Arc::try_unwrap(result_mutex)
            .unwrap()
            .into_inner()
            .unwrap();
        Ok(result)
    }
}

fn screen_image_2_image_image(screen_image: Image) -> anyhow::Result<DynamicImage> {
    let buffer = screen_image.to_png()?;
    let image = image::load_from_memory(&buffer)?;
    Ok(image)
}
