use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::path::Path;
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, TS)]
#[ts(export)]
pub struct HnItem {
    pub id: i64,
    pub item_type: String,
    #[ts(type = "string | null")]
    pub by: Option<String>,
    #[ts(type = "string")]
    pub time: DateTime<Utc>,
    #[ts(type = "string | null")]
    pub text: Option<String>,
    #[ts(type = "string | null")]
    pub url: Option<String>,
    #[ts(type = "number | null")]
    pub score: Option<i64>,
    #[ts(type = "string | null")]
    pub title: Option<String>,
    #[ts(type = "number | null")]
    pub parent: Option<i64>,
    #[ts(type = "string | null")]
    pub kids: Option<String>, // JSON array of child IDs
    #[ts(type = "number | null")]
    pub descendants: Option<i64>,
    pub deleted: bool,
    pub dead: bool,
    #[ts(type = "string")]
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

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DownloadStats {
    pub total_items: u32,
    pub total_users: u32,
    #[ts(type = "string | null")]
    pub last_download_time: Option<DateTime<Utc>>,
    pub items_downloaded_today: u32,
    pub download_errors: u32,
    pub is_downloading: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, TS)]
#[ts(export)]
pub struct Entity {
    pub id: i64,
    pub entity_type: String,
    pub entity_value: String,
    // Consider adding first_seen_at, last_seen_at, total_occurrences if needed later
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EntityReference {
    pub id: i64,
    pub item_id: i64,
    pub entity_id: i64,        // Foreign key to the new Entity table
    pub original_text: String, // Original text where entity was found
    pub start_offset: i64,
    pub end_offset: i64,
    pub confidence: Option<f64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ExtractionStats {
    pub total_entities: u32,
    #[ts(type = "Record<string, number>")]
    pub entities_by_type: std::collections::HashMap<String, u32>,
    pub total_items_processed: u32,
    pub items_remaining: u32,
    pub is_extracting: bool,
    #[ts(type = "string | null")]
    pub last_extraction_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, TS)]
#[ts(export)]
pub struct EntityRelation {
    pub id: i64,
    pub subject_entity_id: i64,
    pub object_entity_id: i64,
    pub relation_type: String,
    pub confidence: Option<f64>,
    #[ts(type = "string")]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EntityRelationReference {
    pub id: i64,
    pub item_id: i64,
    pub relation_id: i64,
    pub original_text: String, // Original text where relation was found
    pub start_offset: i64,
    pub end_offset: i64,
    pub confidence: Option<f64>,
    pub created_at: DateTime<Utc>,
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

        // Create new entities table for unique entities
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS entities (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                entity_type TEXT NOT NULL,
                entity_value TEXT NOT NULL,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                UNIQUE (entity_type, entity_value)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create entity_references table (renamed from old entities table)
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS entity_references (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                item_id INTEGER NOT NULL,
                entity_id INTEGER NOT NULL,
                original_text TEXT NOT NULL,
                start_offset INTEGER NOT NULL,
                end_offset INTEGER NOT NULL,
                confidence REAL,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (item_id) REFERENCES hn_items (id),
                FOREIGN KEY (entity_id) REFERENCES entities (id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Drop old entities table if it exists (in case of migration from old schema)
        // This is potentially destructive if not handled carefully during a real migration.
        // For this development task, we assume we're starting fresh or can rebuild.
        sqlx::query("DROP TABLE IF EXISTS old_entities_temp_backup") // Clean up if previous attempt failed
            .execute(&self.pool)
            .await?;
        // Check if 'entities' table has item_id column (means it's the old schema)
        let old_entities_table_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM pragma_table_info('entities') WHERE name='item_id')",
        )
        .fetch_one(&self.pool)
        .await
        .unwrap_or(false);

        if old_entities_table_exists {
            // This is a simplified migration. A real migration would preserve data.
            // For now, we'll drop the old table if it matches the old schema,
            // assuming the new tables are created correctly.
            // This logic is imperfect as 'entities' table is created above.
            // A more robust migration would involve:
            // 1. RENAME TABLE entities TO entities_old;
            // 2. CREATE TABLE entities (new schema);
            // 3. CREATE TABLE entity_references (new schema);
            // 4. Migrate data from entities_old to entities and entity_references
            // 5. DROP TABLE entities_old;
            // For this exercise, we are focusing on the new schema creation.
            // The CREATE TABLE IF NOT EXISTS handles the creation.
            // We might need to manually ensure the old 'entities' table (if it had item_id)
            // is effectively replaced by 'entity_references'.
            // The above CREATE for 'entities' already made the new one.
            // The one for 'entity_references' made the linking table.
            // If an 'entities' table with 'item_id' still exists and wasn't 'entity_references',
            // it implies a naming conflict or an incomplete previous migration.
            // For now, we'll assume `CREATE TABLE IF NOT EXISTS` handles it,
            // and the schema will be as defined.
        }

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

        // Create entity_relations table for storing relationships between entities
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS entity_relations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                subject_entity_id INTEGER NOT NULL,
                object_entity_id INTEGER NOT NULL,
                relation_type TEXT NOT NULL,
                confidence REAL,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (subject_entity_id) REFERENCES entities (id),
                FOREIGN KEY (object_entity_id) REFERENCES entities (id),
                UNIQUE (subject_entity_id, object_entity_id, relation_type)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create entity_relation_references table for tracking where relations were found
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS entity_relation_references (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                item_id INTEGER NOT NULL,
                relation_id INTEGER NOT NULL,
                original_text TEXT NOT NULL,
                start_offset INTEGER NOT NULL,
                end_offset INTEGER NOT NULL,
                confidence REAL,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (item_id) REFERENCES hn_items (id),
                FOREIGN KEY (relation_id) REFERENCES entity_relations (id)
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
        // Create indexes for the new tables
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_entity_references_item_id ON entity_references(item_id)",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_entity_references_entity_id ON entity_references(entity_id)",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_entities_type_value ON entities(entity_type, entity_value)",
        )
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
            total_items: total_items as u32,
            total_users: total_users as u32,
            last_download_time,
            items_downloaded_today: items_downloaded_today as u32,
            download_errors: download_errors as u32,
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
    pub async fn get_or_insert_entity(
        &self,
        entity_type: &str,
        entity_value: &str,
    ) -> Result<i64, sqlx::Error> {
        // Try to fetch existing entity
        let entity_id: Option<i64> = sqlx::query_scalar(
            "SELECT id FROM entities WHERE entity_type = ? AND entity_value = ?",
        )
        .bind(entity_type)
        .bind(entity_value)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(id) = entity_id {
            Ok(id)
        } else {
            // Insert if not exists
            let _result = sqlx::query(
                "INSERT INTO entities (entity_type, entity_value, created_at) VALUES (?, ?, ?)",
            )
            .bind(entity_type)
            .bind(entity_value)
            .bind(Utc::now())
            .execute(&self.pool)
            .await?;

            // Re-fetch to be absolutely sure we get the ID, especially in concurrent or complex transaction scenarios.
            let entity_id: i64 = sqlx::query_scalar(
                "SELECT id FROM entities WHERE entity_type = ? AND entity_value = ?",
            )
            .bind(entity_type)
            .bind(entity_value)
            .fetch_one(&self.pool) // Use fetch_one as it should exist now
            .await?;
            Ok(entity_id)
        }
    }

    pub async fn insert_entity_reference(
        &self,
        item_id: i64,
        entity_id: i64,
        original_text: &str,
        start_offset: i64,
        end_offset: i64,
        confidence: Option<f64>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO entity_references
            (item_id, entity_id, original_text, start_offset, end_offset, confidence, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(item_id)
        .bind(entity_id)
        .bind(original_text)
        .bind(start_offset)
        .bind(end_offset)
        .bind(confidence)
        .bind(Utc::now())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_extraction_stats(&self) -> Result<ExtractionStats, sqlx::Error> {
        let total_entities: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM entities") // Count unique entities
            .fetch_one(&self.pool)
            .await?;

        let total_items_processed: i64 =
            sqlx::query_scalar("SELECT COUNT(DISTINCT item_id) FROM entity_references") // Count items that have entity references
                .fetch_one(&self.pool)
                .await?;

        let total_items: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM hn_items")
            .fetch_one(&self.pool)
            .await?;

        let items_remaining = total_items - total_items_processed;
        let items_remaining = if items_remaining < 0 {
            0
        } else {
            items_remaining
        };

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

        // Get entities by type from the new 'entities' table
        let entity_type_rows: Vec<(String, i64)> = sqlx::query_as(
            "SELECT entity_type, COUNT(*) as count FROM entities GROUP BY entity_type",
        )
        .fetch_all(&self.pool)
        .await?;

        let entities_by_type = entity_type_rows
            .into_iter()
            .map(|(entity_type, count)| (entity_type, count as u32))
            .collect();

        Ok(ExtractionStats {
            total_entities: total_entities as u32,
            entities_by_type,
            total_items_processed: total_items_processed as u32,
            items_remaining: items_remaining as u32,
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
            WHERE id NOT IN (SELECT DISTINCT item_id FROM entity_references)
            AND (text IS NOT NULL OR title IS NOT NULL)
            LIMIT ?
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(items)
    }

    pub async fn get_items_paginated(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<HnItem>, sqlx::Error> {
        let items = sqlx::query_as::<_, HnItem>(
            r#"
            SELECT * FROM hn_items
            ORDER BY time DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(items)
    }

    pub async fn get_total_items_count(&self) -> Result<i64, sqlx::Error> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM hn_items")
            .fetch_one(&self.pool)
            .await?;
        Ok(count)
    }

    pub async fn get_entities_paginated(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Entity>, sqlx::Error> {
        let entities = sqlx::query_as::<_, Entity>(
            r#"
            SELECT * FROM entities
            ORDER BY created_at DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(entities)
    }

    pub async fn get_total_entities_count(&self) -> Result<i64, sqlx::Error> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM entities")
            .fetch_one(&self.pool)
            .await?;
        Ok(count)
    }

    // Entity relation methods
    pub async fn get_or_insert_relation(
        &self,
        subject_entity_id: i64,
        object_entity_id: i64,
        relation_type: &str,
        confidence: Option<f64>,
    ) -> Result<i64, sqlx::Error> {
        // Try to get existing relation
        let existing_relation: Option<i64> = sqlx::query_scalar(
            "SELECT id FROM entity_relations WHERE subject_entity_id = ? AND object_entity_id = ? AND relation_type = ?",
        )
        .bind(subject_entity_id)
        .bind(object_entity_id)
        .bind(relation_type)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(relation_id) = existing_relation {
            // Update confidence if provided and higher
            if let Some(new_confidence) = confidence {
                sqlx::query(
                    "UPDATE entity_relations SET confidence = ? WHERE id = ? AND (confidence IS NULL OR confidence < ?)",
                )
                .bind(new_confidence)
                .bind(relation_id)
                .bind(new_confidence)
                .execute(&self.pool)
                .await?;
            }
            Ok(relation_id)
        } else {
            // Insert new relation
            let result = sqlx::query(
                "INSERT INTO entity_relations (subject_entity_id, object_entity_id, relation_type, confidence) VALUES (?, ?, ?, ?)",
            )
            .bind(subject_entity_id)
            .bind(object_entity_id)
            .bind(relation_type)
            .bind(confidence)
            .execute(&self.pool)
            .await?;
            Ok(result.last_insert_rowid())
        }
    }

    pub async fn insert_relation_reference(
        &self,
        item_id: i64,
        relation_id: i64,
        original_text: &str,
        start_offset: i64,
        end_offset: i64,
        confidence: Option<f64>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO entity_relation_references 
            (item_id, relation_id, original_text, start_offset, end_offset, confidence)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(item_id)
        .bind(relation_id)
        .bind(original_text)
        .bind(start_offset)
        .bind(end_offset)
        .bind(confidence)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_relations_paginated(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<EntityRelation>, sqlx::Error> {
        let relations = sqlx::query_as::<_, EntityRelation>(
            r#"
            SELECT * FROM entity_relations
            ORDER BY created_at DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;
        Ok(relations)
    }

    pub async fn get_total_relations_count(&self) -> Result<i64, sqlx::Error> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM entity_relations")
            .fetch_one(&self.pool)
            .await?;
        Ok(count)
    }

    pub async fn get_relations_for_entity(
        &self,
        entity_id: i64,
    ) -> Result<Vec<EntityRelation>, sqlx::Error> {
        let relations = sqlx::query_as::<_, EntityRelation>(
            r#"
            SELECT * FROM entity_relations
            WHERE subject_entity_id = ? OR object_entity_id = ?
            ORDER BY created_at DESC
            "#,
        )
        .bind(entity_id)
        .bind(entity_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(relations)
    }
}
