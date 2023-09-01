use super::{EntityImage, EntityWord, Repository};
use anyhow::Result;
use async_trait::async_trait;
use futures::TryStreamExt;
use sqlx::Row;

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
            archive_info TEXT NOT NULL,
            captured_at_epoch INTEGER NOT NULL
        )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS word (
                id INTEGER PRIMARY KEY,
                image_id INTEGER NOT NULL,
                content TEXT NOT NULL,
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
        let query_result = sqlx::query(
            "INSERT INTO images (archive_type, archive_info, captured_at_epoch) VALUES (?, ?, ?)",
        )
        .bind(&entity.archive_type)
        .bind(&entity.archive_info)
        .bind(&(entity.captured_at_epoch as i64))
        .execute(&self.pool)
        .await?;
        let id = query_result.last_insert_rowid() as u32;
        Ok(EntityImage {
            id,
            archive_type: entity.archive_type.clone(),
            archive_info: entity.archive_info.clone(),
            captured_at_epoch: entity.captured_at_epoch,
        })
    }

    async fn get_image_by_id(&self, id: u32) -> Result<EntityImage> {
        let query = sqlx::query(
            "SELECT archive_type, archive_info, captured_at_epoch FROM images WHERE id = ?",
        )
        .bind(id);
        let row = query.fetch_one(&self.pool).await?;
        let archive_type: String = row.get(0);
        let archive_info: String = row.get(1);
        let captured_at_epoch: i64 = row.get(2);
        Ok(EntityImage {
            id,
            archive_type,
            archive_info,
            captured_at_epoch: captured_at_epoch.try_into()?,
        })
    }

    async fn save_word(&self, entity: &EntityWord) -> Result<EntityWord> {
        let query = sqlx::query(
            "INSERT INTO word (image_id, content, left, top, width, height) VALUES (?, ?, ?, ?, ?, ?)",
        );
        let query_result = query
            .bind(entity.image_id)
            .bind(&entity.content)
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
            .bind(&entity.content)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(EntityWord {
            id,
            image_id: entity.image_id,
            content: entity.content.clone(),
            left: entity.left,
            top: entity.top,
            width: entity.width,
            height: entity.height,
        })
    }

    async fn save_words(&self, words: &[EntityWord]) -> Result<Vec<EntityWord>> {
        let mut builder = sqlx::QueryBuilder::new(
            "INSERT INTO word (image_id, content, left, top, width, height)",
        );
        builder.push_values(words, |mut b, it| {
            b.push(it.image_id)
                // TODO: sqlx just concat the SQL string without quoting, so we have to do it manually.
                // TODO: and it's not safe at all.
                .push(format!("'{}'", it.content.clone().replace('\'', "''")))
                .push(it.left)
                .push(it.top)
                .push(it.width)
                .push(it.height);
        });
        let query = builder.build();
        let execute_result = query.execute(&self.pool).await?;
        let rows_affected = execute_result.rows_affected();
        let last_insert_rowid = execute_result.last_insert_rowid();

        let id_start = 1 + last_insert_rowid as u32 - rows_affected as u32;

        let result = words
            .iter()
            .enumerate()
            .map(|(i, it)| EntityWord {
                id: id_start + i as u32,
                image_id: it.image_id,
                // TODO: sqlx just concat the SQL string without quoting, so we have to do it manually.
                // TODO: and it's not safe at all.
                content: (format!("'{}'", it.content.clone().replace('\'', "''"))),
                left: it.left,
                top: it.top,
                width: it.width,
                height: it.height,
            })
            .collect();

        let mut builder = sqlx::QueryBuilder::new("INSERT INTO text_fts (text, text_id)");

        builder.push_values(&result, |mut b, it: &EntityWord| {
            b.push(it.content.clone()).push(it.id);
        });
        let query = builder.build();
        query.execute(&self.pool).await?;

        Ok(result)
    }

    async fn get_word_by_id(&self, id: u32) -> Result<EntityWord> {
        let query = sqlx::query(
            "SELECT image_id, content, left, top, width, height FROM word WHERE id = ?",
        )
        .bind(id);
        let row = query.fetch_one(&self.pool).await?;
        let image_id: u32 = row.get(0);
        let content: String = row.get(1);
        let left: u32 = row.get(2);
        let top: u32 = row.get(3);
        let width: u32 = row.get(4);
        let height: u32 = row.get(5);
        Ok(EntityWord {
            id,
            image_id,
            content: content,
            left,
            top,
            width,
            height,
        })
    }

    async fn full_text_search(&self, text: &str) -> Result<Vec<EntityWord>> {
        let query = sqlx::query("SELECT text_id FROM text_fts WHERE text_fts MATCH ?").bind(text);
        let mut rows = query.fetch(&self.pool);
        let mut result = vec![];
        while let Some(row) = rows.try_next().await? {
            let text_id: u32 = row.get(0);
            let entity = self.get_word_by_id(text_id).await?;
            result.push(entity);
        }
        Ok(result)
    }
}
