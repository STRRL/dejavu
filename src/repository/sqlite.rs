use std::{
    panic,
    sync::{Mutex, Once},
};

use super::{EntityImage, EntityText, Repository};
use anyhow::anyhow;
use async_trait::async_trait;
use rusqlite::{params, Connection};

pub struct SqliteRepository {
    conn: Mutex<Connection>,
    initialize_once: Once,
}

impl SqliteRepository {
    fn initialize(&self) {
        self.initialize_once.call_once(|| {
            let conn = self.conn.lock().unwrap();
            conn.execute(
                "CREATE TABLE IF NOT EXISTS images (
                    id INTEGER PRIMARY KEY,
                    archive_type TEXT NOT NULL,
                    archive_info TEXT NOT NULL
                )",
                [],
            )
            .unwrap();
            conn.execute(
                "CREATE TABLE IF NOT EXISTS texts (
                    id INTEGER PRIMARY KEY,
                    image_id INTEGER NOT NULL,
                    text TEXT NOT NULL,
                    left INTEGER NOT NULL,
                    top INTEGER NOT NULL,
                    width INTEGER NOT NULL,
                    height INTEGER NOT NULL
                )",
                [],
            )
            .unwrap();
            // create text_fts table, with field text and text_id
            conn.execute(
                "CREATE VIRTUAL TABLE IF NOT EXISTS text_fts USING fts5(text, text_id UNINDEXED)",
                [],
            )
            .unwrap();
        });
    }

    pub fn new(conn: Connection) -> Self {
        let result = Self {
            conn: Mutex::new(conn),
            initialize_once: Once::new(),
        };
        result.initialize();
        result
    }
}

#[async_trait]
impl Repository for SqliteRepository {
    async fn save_image(&self, entity: &EntityImage) -> anyhow::Result<EntityImage> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO images (archive_type, archive_info) VALUES (?1, ?2)",
            params![entity.archive_type.as_str(), entity.archive_info.as_str()],
        )
        .unwrap();
        let id = conn.last_insert_rowid() as u32;
        Ok(EntityImage::new(
            id,
            entity.archive_type.clone(),
            entity.archive_info.clone(),
        ))
    }
    async fn get_image_by_id(&self, id: u32) -> anyhow::Result<EntityImage> {
        let conn = self.conn.lock().unwrap();
        let mut stmt =
            conn.prepare("SELECT archive_type, archive_info FROM images WHERE id = ?1")?;
        let mut rows = stmt.query([id])?;
        let row = rows
            .next()?
            .ok_or(anyhow!("not found image with id {:?}", id))?;
        let archive_type: String = row.get(0).unwrap();
        let archive_info: String = row.get(1).unwrap();
        Ok(EntityImage::new(id, archive_type, archive_info))
    }

    async fn save_text(&self, entity: &EntityText) -> anyhow::Result<EntityText> {
        let conn = self.conn.lock().unwrap();
        // insert into text table
        conn.execute(
            "INSERT INTO texts (image_id, text, left, top, width, height) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                entity.image_id,
                entity.text.as_str(),
                entity.left,
                entity.top,
                entity.width,
                entity.height,
            ],
        )?;
        let id = conn.last_insert_rowid() as u32;
        let result = Ok(EntityText::new(
            id,
            entity.image_id,
            entity.text.clone(),
            entity.left,
            entity.top,
            entity.width,
            entity.height,
        ));
        // insert into fts table
        conn.execute(
            "INSERT INTO text_fts (text, text_id) VALUES (?1, ?2)",
            params![entity.text.as_str(), id,],
        )?;
        result
    }

    async fn save_texts(&self, entities: &Vec<EntityText>) -> anyhow::Result<Vec<EntityText>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "INSERT INTO texts (image_id, text, left, top, width, height) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )?;
        let mut result = Vec::new();
        for entity in entities {
            // insert into text table
            stmt.execute(params![
                entity.image_id,
                entity.text.as_str(),
                entity.left,
                entity.top,
                entity.width,
                entity.height,
            ])?;
            let id = conn.last_insert_rowid() as u32;
            // insert into fts table
            conn.execute(
                "INSERT INTO text_fts (text, text_id) VALUES (?1, ?2)",
                params![entity.text.as_str(), id,],
            )?;
            result.push(EntityText::new(
                id,
                entity.image_id,
                entity.text.clone(),
                entity.left,
                entity.top,
                entity.width,
                entity.height,
            ));
        }
        Ok(result)
    }
    async fn get_text_by_id(&self, id: u32) -> anyhow::Result<EntityText> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT image_id, text, left, top, width, height FROM texts WHERE id = ?1")
            .unwrap();
        let mut rows = stmt.query([id]).unwrap();
        let row = rows.next().unwrap().unwrap();
        let image_id: u32 = row.get(0).unwrap();
        let text: String = row.get(1).unwrap();
        let left: u32 = row.get(2).unwrap();
        let top: u32 = row.get(3).unwrap();
        let width: u32 = row.get(4).unwrap();
        let height: u32 = row.get(5).unwrap();
        Ok(EntityText::new(
            id, image_id, text, left, top, width, height,
        ))
    }

    async fn full_text_search(&self, text: &str) -> anyhow::Result<Vec<EntityText>> {
        // search from text_fts table
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT text_id FROM text_fts WHERE text_fts MATCH ?1")?;
        let mut rows = stmt.query([text])?;
        let mut result = Vec::new();
        while let Some(row) = rows.next()? {
            let text_id: u32 = row.get(0).unwrap();
            let mut stmt = conn.prepare(
                "SELECT image_id, text, left, top, width, height FROM texts WHERE id = ?1",
            )?;
            let mut rows = stmt.query([text_id])?;
            let row = rows
                .next()?
                .ok_or(anyhow!("not found text with id {:?}", text_id))?;
            let image_id: u32 = row.get(0).unwrap();
            let text: String = row.get(1).unwrap();
            let left: u32 = row.get(2).unwrap();
            let top: u32 = row.get(3).unwrap();
            let width: u32 = row.get(4).unwrap();
            let height: u32 = row.get(5).unwrap();
            result.push(EntityText::new(
                text_id, image_id, text, left, top, width, height,
            ));
        }
        Ok(result)
    }
}
