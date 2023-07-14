use std::sync::Arc;

use super::{EntityImage, EntityText, Repository};
use anyhow::Result;
use async_trait::async_trait;
use futures::TryStreamExt;
use itertools::Itertools;
use sqlx::{Executor, Row};
use tracing::trace;

pub struct SqliteRepository {
    pool: sqlx::Pool<sqlx_sqlite::Sqlite>,
}

impl SqliteRepository {
    pub fn new(pool: sqlx::Pool<sqlx_sqlite::Sqlite>) -> Self {
        Self { pool }
    }

    pub async fn initialize(&self) -> Result<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS images (
            id INTEGER PRIMARY KEY,
            archive_type TEXT NOT NULL,
            archive_info TEXT NOT NULL
        )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS texts (
                id INTEGER PRIMARY KEY,
                image_id INTEGER NOT NULL,
                text TEXT NOT NULL,
                left INTEGER NOT NULL,
                top INTEGER NOT NULL,
                width INTEGER NOT NULL,
                height INTEGER NOT NULL
            )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE VIRTUAL TABLE IF NOT EXISTS text_fts USING fts5(text, text_id UNINDEXED)",
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
#[async_trait]
impl Repository for SqliteRepository {
    async fn save_image(&self, entity: &EntityImage) -> Result<EntityImage> {
        let query_result =
            sqlx::query("INSERT INTO images (archive_type, archive_info) VALUES (?, ?)")
                .bind(&entity.archive_type)
                .bind(&entity.archive_info)
                .execute(&self.pool)
                .await?;
        let id = query_result.last_insert_rowid() as u32;
        Ok(EntityImage {
            id,
            archive_type: entity.archive_type.clone(),
            archive_info: entity.archive_info.clone(),
        })
    }

    async fn get_image_by_id(&self, id: u32) -> Result<EntityImage> {
        let query =
            sqlx::query("SELECT archive_type, archive_info FROM images WHERE id = ?").bind(id);
        let row = query.fetch_one(&self.pool).await?;
        let archive_type: String = row.get(0);
        let archive_info: String = row.get(1);
        Ok(EntityImage {
            id,
            archive_type,
            archive_info,
        })
    }

    async fn save_text(&self, entity: &EntityText) -> Result<EntityText> {
        let query = sqlx::query(
            "INSERT INTO texts (image_id, text, left, top, width, height) VALUES (?, ?, ?, ?, ?, ?)",
        );
        let query_result = query
            .bind(entity.image_id)
            .bind(&entity.text)
            .bind(entity.left)
            .bind(entity.top)
            .bind(entity.width)
            .bind(entity.height)
            .execute(&self.pool)
            .await?;
        let id = query_result.last_insert_rowid() as u32;
        // insert into table text_fts
        let query = sqlx::query("INSERT INTO text_fts (text, text_id) VALUES (?, ?)");
        query
            .bind(&entity.text)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(EntityText {
            id,
            image_id: entity.image_id,
            text: entity.text.clone(),
            left: entity.left,
            top: entity.top,
            width: entity.width,
            height: entity.height,
        })
    }

    async fn save_texts(&self, entities: &Vec<EntityText>) -> Result<Vec<EntityText>> {
        // TODO: batch insert
        let values = entities
            .into_iter()
            .map(|it| {
                format!(
                    "({}, '{}', {}, {}, {}, {})",
                    it.image_id,
                    it.text.replace("'", "''"),
                    it.left,
                    it.top,
                    it.width,
                    it.height
                )
                .to_string()
            })
            .join(", ");
        let query = format!(
            "INSERT INTO texts (image_id, text, left, top, width, height) VALUES {}",
            values
        );
        let query = sqlx::query(query.as_str());
        let execute_result = self.pool.execute(query).await?;
        let rows_affected = execute_result.rows_affected();
        let last_insert_rowid = execute_result.last_insert_rowid();
        let mut result = Vec::new();
        let mut id_start = 1 + last_insert_rowid as u32 - rows_affected as u32;

        // batch insert into text_fts
        let values = entities
            .into_iter()
            .enumerate()
            .map(|(index, it)| {
                format!(
                    "('{}', {})",
                    it.text.replace("'", "''"),
                    id_start + index as u32
                )
                .to_string()
            })
            .join(", ");
        let query = format!("INSERT INTO text_fts (text, text_id) VALUES {}", values);
        let query = sqlx::query(query.as_str());
        self.pool.execute(query).await?;


        for entity in entities {
            result.push(EntityText {
                id: id_start,
                image_id: entity.image_id,
                text: entity.text.clone(),
                left: entity.left,
                top: entity.top,
                width: entity.width,
                height: entity.height,
            });
            id_start += 1;
        }

        Ok(result)
    }

    async fn get_text_by_id(&self, id: u32) -> Result<EntityText> {
        let query =
            sqlx::query("SELECT image_id, text, left, top, width, height FROM texts WHERE id = ?")
                .bind(id);
        let row = query.fetch_one(&self.pool).await?;
        let image_id: u32 = row.get(0);
        let text: String = row.get(1);
        let left: u32 = row.get(2);
        let top: u32 = row.get(3);
        let width: u32 = row.get(4);
        let height: u32 = row.get(5);
        Ok(EntityText {
            id,
            image_id,
            text,
            left,
            top,
            width,
            height,
        })
    }

    async fn full_text_search(&self, text: &str) -> Result<Vec<EntityText>> {
        let query = sqlx::query("SELECT text_id FROM text_fts WHERE text_fts MATCH ?1").bind(text);
        let mut rows = query.fetch(&self.pool);
        let mut result = vec![];
        while let Some(row) = rows.try_next().await? {
            let text_id: u32 = row.get(0);
            let entity = self.get_text_by_id(text_id).await?;
            result.push(entity);
        }
        Ok(result)
    }
}
