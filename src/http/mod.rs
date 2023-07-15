use self::{error::HttpError, service::Service};
use crate::analysis::SearchResult;
use axum::{extract::Query, http::header, response::IntoResponse, Extension, Json};
use image::ImageFormat;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
pub mod error;
pub mod service;
#[derive(Deserialize, Serialize)]
pub struct SearchQuery {
    text: String,
}

pub async fn search(
    Extension(service): Extension<Arc<Service>>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Vec<SearchResult>>, HttpError> {
    let result = service.clone().search(&query.text).await?;
    Ok(Json(result))
}

#[derive(Deserialize, Serialize)]
pub struct ImageWithMarkupQuery {
    image_id: u32,
    /// comma separated list of text ids
    text_ids: String,
}

pub async fn fetch_image_with_markup(
    Extension(service): Extension<Arc<Service>>,
    Query(query): Query<ImageWithMarkupQuery>,
) -> Result<impl IntoResponse, HttpError> {
    let text_ids = query
        .text_ids
        .split(',')
        .map(|id| id.parse::<u32>().unwrap())
        .collect::<Vec<u32>>();
    let marked = service
        .clone()
        .fetch_image_with_markup(query.image_id, &text_ids)
        .await?;
    use std::io::{BufWriter, Cursor};
    let mut buffer = BufWriter::new(Cursor::new(Vec::new()));
    marked.write_to(&mut buffer, ImageFormat::Png).unwrap();
    let bytes: Vec<u8> = buffer.into_inner().unwrap().into_inner();
    Ok((
        axum::response::AppendHeaders([(header::CONTENT_TYPE, "image/png")]),
        bytes,
    ))
}
