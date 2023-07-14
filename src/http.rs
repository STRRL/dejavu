use crate::analysis::{Analysis, SearchResult};
use axum::{
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use serde::{Deserialize, Serialize};



use std::sync::Arc;

#[derive(Deserialize, Serialize)]
pub struct SearchQuery {
    text: String,
}

pub async fn search(
    Extension(analysis): Extension<Arc<Analysis>>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Vec<SearchResult>>, HttpError> {
    let result = analysis.clone().search(&query.text).await?;
    Ok(Json(result))
}

pub struct HttpError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

impl<E> From<E> for HttpError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
