use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct HnItem {
    pub id: i64,
    pub item_type: String,
    pub by: Option<String>,
    pub time: DateTime<Utc>,
    pub text: Option<String>,
    pub url: Option<String>,
    pub score: Option<i64>,
    pub title: Option<String>,
    pub parent: Option<i64>,
    pub kids: Option<String>, // JSON array of child IDs
    pub descendants: Option<i64>,
    pub deleted: bool,
    pub dead: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct HnUser {
    pub id: String,
    pub created: DateTime<Utc>,
    pub karma: Option<i64>,
    pub about: Option<String>,
    pub submitted: Option<String>, // JSON array of submitted item IDs
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadStats {
    pub total_items: u64,
    pub total_users: u64,
    pub last_download_time: Option<DateTime<Utc>>,
    pub items_downloaded_today: u64,
    pub download_errors: u64,
    pub is_downloading: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Entity {
    pub id: i64,
    pub item_id: i64,
    pub entity_type: String, // Person, Company, Location, Date, Financial Amount, etc.
    pub entity_value: String, // Trimmed, clean entity text
    pub original_text: String, // Original text where entity was found
    pub start_offset: i64,
    pub end_offset: i64,
    pub confidence: Option<f64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionStats {
    pub total_entities: u64,
    pub entities_by_type: std::collections::HashMap<String, u64>,
    pub total_items_processed: u64,
    pub items_remaining: u64,
    pub is_extracting: bool,
    pub last_extraction_time: Option<DateTime<Utc>>,
}

pub struct Database {
    pub pool: SqlitePool,
}

impl Database {
    pub async fn new(db_path: &Path) -> Result<Self, sqlx::Error> {
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                sqlx::Error::Io(std::io::Error::other(format!(
                    "Failed to create database directory: {e}"
                )))
            })?;
        }

        let database_url = format!("sqlite:{}?mode=rwc", db_path.display());
        let pool = SqlitePool::connect(&database_url).await?;

        let db = Database { pool };
        db.migrate().await?;
        Ok(db)
    }

    async fn migrate(&self) -> Result<(), sqlx::Error> {
        // Create items table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS hn_items (
                id INTEGER PRIMARY KEY,
                item_type TEXT NOT NULL,
                by TEXT,
                time DATETIME NOT NULL,
                text TEXT,
                url TEXT,
                score INTEGER,
                title TEXT,
                parent INTEGER,
                kids TEXT, -- JSON array
                descendants INTEGER,
                deleted BOOLEAN NOT NULL DEFAULT FALSE,
                dead BOOLEAN NOT NULL DEFAULT FALSE,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create users table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS hn_users (
                id TEXT PRIMARY KEY,
                created DATETIME NOT NULL,
                karma INTEGER,
                about TEXT,
                submitted TEXT, -- JSON array
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create download_log table for tracking downloads
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS download_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                download_type TEXT NOT NULL, -- 'items' or 'users'
                items_count INTEGER NOT NULL DEFAULT 0,
                errors_count INTEGER NOT NULL DEFAULT 0,
                started_at DATETIME NOT NULL,
                completed_at DATETIME,
                status TEXT NOT NULL DEFAULT 'running' -- 'running', 'completed', 'failed'
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create entities table for storing extracted entities
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS entities (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                item_id INTEGER NOT NULL,
                entity_type TEXT NOT NULL,
                entity_value TEXT NOT NULL,
                original_text TEXT NOT NULL,
                start_offset INTEGER NOT NULL,
                end_offset INTEGER NOT NULL,
                confidence REAL,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (item_id) REFERENCES hn_items (id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create extraction_log table for tracking entity extraction sessions
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS extraction_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                entities_count INTEGER NOT NULL DEFAULT 0,
                items_processed INTEGER NOT NULL DEFAULT 0,
                started_at DATETIME NOT NULL,
                completed_at DATETIME,
                status TEXT NOT NULL DEFAULT 'running' -- 'running', 'completed', 'failed', 'paused'
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indexes for better performance
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_items_type ON hn_items(item_type)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_items_time ON hn_items(time)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_items_by ON hn_items(by)")
            .execute(&self.pool)
            .await?;

        // Create indexes for entities table
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_entities_item_id ON entities(item_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_entities_type ON entities(entity_type)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_entities_value ON entities(entity_value)")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn insert_item(&self, item: &HnItem) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO hn_items 
            (id, item_type, by, time, text, url, score, title, parent, kids, descendants, deleted, dead, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(item.id)
        .bind(&item.item_type)
        .bind(&item.by)
        .bind(item.time)
        .bind(&item.text)
        .bind(&item.url)
        .bind(item.score)
        .bind(&item.title)
        .bind(item.parent)
        .bind(&item.kids)
        .bind(item.descendants)
        .bind(item.deleted)
        .bind(item.dead)
        .bind(item.created_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn insert_user(&self, user: &HnUser) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO hn_users 
            (id, created, karma, about, submitted, created_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&user.id)
        .bind(user.created)
        .bind(user.karma)
        .bind(&user.about)
        .bind(&user.submitted)
        .bind(user.created_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_stats(&self) -> Result<DownloadStats, sqlx::Error> {
        let total_items: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM hn_items")
            .fetch_one(&self.pool)
            .await?;

        let total_users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM hn_users")
            .fetch_one(&self.pool)
            .await?;

        let last_download_time: Option<DateTime<Utc>> = sqlx::query_scalar(
            "SELECT MAX(started_at) FROM download_log WHERE status = 'completed'",
        )
        .fetch_optional(&self.pool)
        .await?
        .flatten();

        let today = Utc::now().date_naive();
        let items_downloaded_today: i64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(items_count), 0) FROM download_log WHERE DATE(started_at) = ? AND status = 'completed'"
        )
        .bind(today)
        .fetch_one(&self.pool)
        .await?;

        let download_errors: i64 =
            sqlx::query_scalar("SELECT COALESCE(SUM(errors_count), 0) FROM download_log")
                .fetch_one(&self.pool)
                .await?;

        let is_downloading: bool =
            sqlx::query_scalar("SELECT COUNT(*) > 0 FROM download_log WHERE status = 'running'")
                .fetch_one(&self.pool)
                .await?;

        Ok(DownloadStats {
            total_items: total_items as u64,
            total_users: total_users as u64,
            last_download_time,
            items_downloaded_today: items_downloaded_today as u64,
            download_errors: download_errors as u64,
            is_downloading,
        })
    }

    pub async fn start_download_session(&self, download_type: &str) -> Result<i64, sqlx::Error> {
        let result =
            sqlx::query("INSERT INTO download_log (download_type, started_at) VALUES (?, ?)")
                .bind(download_type)
                .bind(Utc::now())
                .execute(&self.pool)
                .await?;

        Ok(result.last_insert_rowid())
    }

    pub async fn update_download_session(
        &self,
        session_id: i64,
        items_count: u64,
        errors_count: u64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE download_log SET items_count = ?, errors_count = ? WHERE id = ?")
            .bind(items_count as i64)
            .bind(errors_count as i64)
            .bind(session_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn complete_download_session(
        &self,
        session_id: i64,
        status: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE download_log SET completed_at = ?, status = ? WHERE id = ?")
            .bind(Utc::now())
            .bind(status)
            .bind(session_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn get_item_count(&self) -> Result<u64, sqlx::Error> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM hn_items")
            .fetch_one(&self.pool)
            .await?;
        Ok(count as u64)
    }

    #[allow(dead_code)]
    pub async fn item_exists(&self, id: i64) -> Result<bool, sqlx::Error> {
        let exists: bool = sqlx::query_scalar("SELECT COUNT(*) > 0 FROM hn_items WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        Ok(exists)
    }

    pub async fn stop_all_downloads(&self) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE download_log SET status = 'stopped', completed_at = CURRENT_TIMESTAMP WHERE status = 'running'")
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // Entity extraction methods
    pub async fn insert_entity(&self, entity: &Entity) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO entities 
            (item_id, entity_type, entity_value, original_text, start_offset, end_offset, confidence, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(entity.item_id)
        .bind(&entity.entity_type)
        .bind(&entity.entity_value)
        .bind(&entity.original_text)
        .bind(entity.start_offset)
        .bind(entity.end_offset)
        .bind(entity.confidence)
        .bind(entity.created_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_extraction_stats(&self) -> Result<ExtractionStats, sqlx::Error> {
        let total_entities: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM entities")
            .fetch_one(&self.pool)
            .await?;

        let total_items_processed: i64 =
            sqlx::query_scalar("SELECT COUNT(DISTINCT item_id) FROM entities")
                .fetch_one(&self.pool)
                .await?;

        let total_items: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM hn_items")
            .fetch_one(&self.pool)
            .await?;

        let items_remaining = total_items - total_items_processed;

        let is_extracting: bool =
            sqlx::query_scalar("SELECT COUNT(*) > 0 FROM extraction_log WHERE status = 'running'")
                .fetch_one(&self.pool)
                .await?;

        let last_extraction_time: Option<DateTime<Utc>> = sqlx::query_scalar(
            "SELECT MAX(completed_at) FROM extraction_log WHERE status = 'completed'",
        )
        .fetch_optional(&self.pool)
        .await?
        .flatten();

        // Get entities by type
        let entity_type_rows: Vec<(String, i64)> = sqlx::query_as(
            "SELECT entity_type, COUNT(*) as count FROM entities GROUP BY entity_type",
        )
        .fetch_all(&self.pool)
        .await?;

        let entities_by_type = entity_type_rows
            .into_iter()
            .map(|(entity_type, count)| (entity_type, count as u64))
            .collect();

        Ok(ExtractionStats {
            total_entities: total_entities as u64,
            entities_by_type,
            total_items_processed: total_items_processed as u64,
            items_remaining: items_remaining as u64,
            is_extracting,
            last_extraction_time,
        })
    }

    pub async fn start_extraction_session(&self) -> Result<i64, sqlx::Error> {
        let result = sqlx::query("INSERT INTO extraction_log (started_at) VALUES (?)")
            .bind(Utc::now())
            .execute(&self.pool)
            .await?;

        Ok(result.last_insert_rowid())
    }

    pub async fn update_extraction_session(
        &self,
        session_id: i64,
        entities_count: u64,
        items_processed: u64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE extraction_log SET entities_count = ?, items_processed = ? WHERE id = ?",
        )
        .bind(entities_count as i64)
        .bind(items_processed as i64)
        .bind(session_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn complete_extraction_session(
        &self,
        session_id: i64,
        status: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE extraction_log SET completed_at = ?, status = ? WHERE id = ?")
            .bind(Utc::now())
            .bind(status)
            .bind(session_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn stop_all_extractions(&self) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE extraction_log SET status = 'paused', completed_at = CURRENT_TIMESTAMP WHERE status = 'running'")
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_items_for_extraction(&self, limit: i64) -> Result<Vec<HnItem>, sqlx::Error> {
        let items = sqlx::query_as::<_, HnItem>(
            r#"
            SELECT * FROM hn_items 
            WHERE id NOT IN (SELECT DISTINCT item_id FROM entities)
            AND (text IS NOT NULL OR title IS NOT NULL)
            LIMIT ?
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(items)
    }
}
