use std::collections::HashMap;
use std::sync::Arc;
use std::{fs, io::Cursor, time::Duration};

use crate::screenshot::Capturer;
use anyhow::Ok;
mod analysis;
mod image_archive;
mod markup;
mod ocr;
mod repository;
mod screenshot;
use tokio::signal;
use tokio::sync::{mpsc, Mutex};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let tesseract_ocr = ocr::TesseractOCR::new();
    let repo = repository::InMemoryRepository::new();
    let archiver = image_archive::InMemoryImageArchiver::new();
    let analysis =
        analysis::Analysis::new(Box::new(tesseract_ocr), Box::new(repo), Box::new(archiver));

    let analysis: Arc<Mutex<analysis::Analysis>> = Arc::new(Mutex::new(analysis));
    let analysis_upload_task = analysis.clone();
    let analysis_search_task = analysis.clone();

    let capture_task = tokio::task::spawn(async move {
        let analysis = analysis_upload_task.clone();
        let capturer = screenshot::DefaultCapturer::new();
        let mut capture_interval = tokio::time::interval(Duration::from_secs(2));
        loop {
            tokio::select! {
                _=signal::ctrl_c() => {
                    println!("Ctrl-C received, exiting...");
                    break;
                },
                _= capture_interval.tick()=>{
                    let captures = capturer.capture().await.unwrap();
                    let mut analysis = analysis.lock().await;
                    for item in captures{
                        analysis.record_screenshot(&item).await.unwrap();
                    }
                },
            }
        }
    });
    let search_task = tokio::task::spawn(async move {
        let analysis = analysis_search_task.clone();
        let mut search_interval = tokio::time::interval(Duration::from_secs(2));
        loop {
            tokio::select! {
                _=signal::ctrl_c() => {
                    println!("Ctrl-C received, exiting...");
                    break;
                },
                _= search_interval.tick()=>{
                    let analysis = analysis.lock().await;
                    let result = analysis.search("hello").await.unwrap();
                    println!("search result: {:?}", result);
                },
            }
        }
    });

    capture_task.await.unwrap();
    Ok(())
}
