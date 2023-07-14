use crate::screenshot::Capturer;

use anyhow::Ok;
use anyhow::Result;
use axum::extract::MatchedPath;
use axum::http::Request;
use axum::routing::get;
use axum::{Extension, Router};
use tracing::trace;
use core::panic;

use sqlx_sqlite::SqlitePoolOptions;
use std::sync::Arc;
use std::time::Duration;
use tokio::signal;

use tower_http::{trace::TraceLayer};
use tracing::{info_span};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
mod analysis;
mod http;
mod image_archive;
mod markup;
mod ocr;
mod repository;
mod screenshot;

#[tokio::main]
async fn main() -> Result<()> {
    let pool = SqlitePoolOptions::new().connect("test.db").await?;
    let repo = repository::sqlite::SqliteRepository::new(pool);
    let tesseract_ocr = ocr::TesseractOCR::new();
    let archiver = image_archive::InMemoryImageArchiver::new();
    let analysis =
        analysis::Analysis::new(Box::new(tesseract_ocr), Box::new(repo), Box::new(archiver));

    let analysis: Arc<analysis::Analysis> = Arc::new(analysis);
    let analysis_upload_task = analysis.clone();
    let analysis_web = analysis.clone();

    let (shutdown_sender, _) = tokio::sync::broadcast::channel::<()>(1);

    let mut shutdown_capture = shutdown_sender.subscribe();
    let mut shutdown_http = shutdown_sender.subscribe();

    let shutdown_guard = tokio::spawn(async move {
        signal::ctrl_c().await.unwrap();
        shutdown_sender.send(()).unwrap();
    });

    let capture_task = tokio::task::spawn(async move {
        let analysis = analysis_upload_task.clone();
        let capturer = screenshot::DefaultCapturer::new();
        let mut capture_interval = tokio::time::interval(Duration::from_secs(2));
        loop {
            tokio::select! {
                _ = shutdown_capture.recv() => {
                    trace!("Ctrl-C received, capture task exiting...");
                    break;
                },
                _ = capture_interval.tick()=>{
                    // print current time
                    trace!("capture task running at {:?}", std::time::SystemTime::now());
                    let captures = capturer.capture().await.unwrap();
                    trace!("capture task captured {} screens", captures.len());
                    for item in captures{
                        analysis.record_screenshot(&item).await.unwrap();
                    }
                },
            }
        }
    });

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                // axum logs rejections from built-in extractors with the `axum::rejection`
                // target, at `TRACE` level. `axum::rejection=trace` enables showing those events
                // "example_tracing_aka_logging=debug,tower_http=debug,axum::rejection=trace".into()
                "trace".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let router = Router::new()
        .route("/search", get(http::search))
        .layer(Extension(analysis_web))
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
                // Log the matched route's path (with placeholders not filled in).
                // Use request.uri() or OriginalUri if you want the real path.
                let matched_path = request
                    .extensions()
                    .get::<MatchedPath>()
                    .map(MatchedPath::as_str);

                info_span!(
                    "http_request",
                    method = ?request.method(),
                    matched_path,
                    some_other_field = tracing::field::Empty,
                )
            }),
        );
    tokio::task::spawn(async move {
        axum::Server::bind(&"0.0.0.0:12333".parse().unwrap())
            .serve(router.into_make_service())
            .with_graceful_shutdown(async {
                shutdown_http.recv().await.ok();
            })
            .await
            .unwrap();
    });
    shutdown_guard.await.unwrap();
    capture_task.await.unwrap();
    Ok(())
}
