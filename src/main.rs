use std::{fs, io::Cursor};

use crate::screenshot::Capturer;
use anyhow::Ok;
use ocr::CharacterRecognizer;
mod analysis;
mod markup;
mod ocr;
mod repository;
mod screenshot;
mod image_archive;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let capturer = screenshot::DefaultCapturer::new();
    let captures = capturer.capture().await?;

    println!("Captured {} images", captures.len());
    let ocr = ocr::TesseractOCR::new();
    let decorator: markup::ImageMarkupDecorator = markup::ImageMarkupDecorator::new();

    for (index, image) in captures.into_iter().enumerate() {
        let ocr_result = ocr.recognize(image.clone()).await?;
        let markups: Vec<ocr::MarkupBox> = ocr_result
            .iter()
            .filter(|it| it.level == 5)
            .map(|it| it.markup)
            .collect();
        println!("markups: {}", markups.len());
        let marked = decorator.markup_recognition(&image, &markups)?;
        let mut buffer: Vec<u8> = Vec::new();
        marked.write_to(&mut Cursor::new(&mut buffer), image::ImageOutputFormat::Png)?;
        fs::write(format!("target/out-{}.png", index), buffer).unwrap();
    }

    Ok(())
}
