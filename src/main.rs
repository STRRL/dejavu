

use anyhow::Ok;
use anyhow::Result;
use axum::extract::MatchedPath;
use axum::http::Request;
use axum::routing::get;
use axum::{Extension, Router};
use crate::screenshot::Capturer;
use tokio::task::JoinHandle;
use core::panic;
use markup::ImageMarkupDecorator;
use sqlx_sqlite::SqlitePoolOptions;
use std::sync::Arc;
use std::time::Duration;
use tokio::signal;

use tokio_util::sync::CancellationToken;
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing::info_span;
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
    let image_dir = format!(
        "{}/{}/{}",
        dirs::data_dir()
            .expect("fetch data dir")
            .to_str()
            .expect("data dir path to string"),
        "dejavu",
        "images"
    );
    tokio::fs::create_dir_all(image_dir.clone()).await?;

    let pool = SqlitePoolOptions::new()
        .connect(
            format!(
                "{}/{}/{}",
                dirs::data_dir()
                    .expect("fetch data dir")
                    .to_str()
                    .expect("data dir path to string"),
                "dejavu",
                "dejavu.db?mode=rwc"
            )
            .as_str(),
        )
        .await?;
    let repo = repository::sqlite::SqliteRepository::new(pool);
    repo.initialize().await?;
    let repo_arc = Arc::new(repo);
    let ocr_arc = Arc::new(ocr::TesseractOCR::new());
    let archiver_arc = Arc::new(image_archive::fs::FileSystemImageArchiver::new(image_dir));

    let analysis_arc: Arc<analysis::Analysis> = {
        let repo_arc = repo_arc.clone();
        let archiver_arc = archiver_arc.clone();
        Arc::new(analysis::Analysis::new(ocr_arc, repo_arc, archiver_arc))
    };
    let token = CancellationToken::new();
    let cloned_token = token.clone();

    let capture_task = {
        let analysis_arc = analysis_arc.clone();
        tokio::task::spawn(async move {
            let capturer = screenshot::DefaultCapturer::new();
            let mut capture_interval = tokio::time::interval(Duration::from_secs(2));
            loop {
                if cloned_token.is_cancelled() {
                    break;
                }
                tokio::select! {
                    _ = cloned_token.cancelled() => {
                        info!("shutting down capture task");
                        break;
                    },
                    _ = capture_interval.tick()=>{
                        let captures = capturer.capture().await.unwrap();
                            let mut tasks : Vec<JoinHandle<()>> = Vec::new();
                            for item in captures {
                            let analysis = analysis_arc.clone();
                            let task = tokio::task::spawn(async move {
                                let result = analysis.record_screenshot(&item).await;
                                if let Err(e) = result {
                                    info!("failed to record screenshot: {}", e);
                                }
                            });
                            tasks.push(task);
                        }
                        for task in tasks {
                            task.await.unwrap();
                        }
                    },
                }
            }
        })
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                // axum logs rejections from built-in extractors with the `axum::rejection`
                // target, at `TRACE` level. `axum::rejection=trace` enables showing those events
                // "example_tracing_aka_logging=debug,tower_http=debug,axum::rejection=trace".into()
                "info".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    let service_arc = {
        let analysis_arc = analysis_arc.clone();
        Arc::new(http::service::Service::new(
            analysis_arc,
            Arc::new(ImageMarkupDecorator::new()),
            repo_arc.clone(),
            archiver_arc.clone(),
        ))
    };

    let api_router = Router::new()
        .route("/search", get(http::search))
        .route("/image", get(http::fetch_image_with_markup));

    let router = Router::new()
        .nest("/api", api_router)
        .fallback(http::frontend::static_handler)
        .layer(Extension(service_arc.clone()))
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

    let cloned_token = token.clone();
    tokio::task::spawn(async move {
        axum::Server::bind(&"0.0.0.0:12333".parse().unwrap())
            .serve(router.into_make_service())
            .with_graceful_shutdown(async {
                cloned_token.cancelled().await;
            })
            .await
            .unwrap();
    });

    let shutdown_guard = tokio::spawn(async move {
        signal::ctrl_c().await.unwrap();
        info!("Ctrl-C received, shutting down");
        token.cancel();
    });
    shutdown_guard.await.unwrap();
    capture_task.await.unwrap();
    Ok(())
}
