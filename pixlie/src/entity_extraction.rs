use crate::database::{Database, HnItem};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tracing::{info, instrument, error};
use ts_rs::TS;

// Relation types for startup/investment domain
#[allow(dead_code)]
pub const RELATION_TYPES: &[&str] = &[
    "founded",        // person founded company
    "co_founded",     // person co-founded company
    "invested_in",    // person/company invested in company
    "works_at",       // person works at company
    "acquired",       // company acquired company
    "created",        // person/company created product
    "developed",      // person/company developed technology
    "leads",          // person leads company/organization
    "partnered_with", // company partnered with company
    "competes_with",  // company competes with company
    "used_by",        // technology used by company
    "backed_by",      // startup backed by investor
];

// Relation patterns for mock extraction
pub const RELATION_PATTERNS: &[(&str, &str)] = &[
    ("founded", r"(?i)\b(founded|started|launched|established)\b"),
    ("co_founded", r"(?i)\b(co-?founded|co-?started)\b"),
    (
        "invested_in",
        r"(?i)\b(invested in|funding|backed|financing)\b",
    ),
    (
        "works_at",
        r"(?i)\b(works at|employed at|engineer at|CEO of)\b",
    ),
    ("acquired", r"(?i)\b(acquired|bought|purchased|merger)\b"),
    ("created", r"(?i)\b(created|built|made|designed)\b"),
    ("developed", r"(?i)\b(developed|built|engineered)\b"),
    ("leads", r"(?i)\b(leads|heading|CEO of|CTO of|founder of)\b"),
    (
        "partnered_with",
        r"(?i)\b(partnered with|partnership|collaboration)\b",
    ),
    ("competes_with", r"(?i)\b(competes with|rival|competitor)\b"),
    ("used_by", r"(?i)\b(uses|powered by|built with|using)\b"),
    ("backed_by", r"(?i)\b(backed by|funded by|supported by)\b"),
];

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ModelInfo {
    pub name: String,
    pub size_mb: u64,
    pub download_url: String,
    pub is_downloaded: bool,
    pub local_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDownloadStatus {
    pub is_downloading: bool,
    pub progress_percent: Option<f32>,
    pub download_speed_mbps: Option<f32>,
    pub error_message: Option<String>,
}

pub struct EntityExtractor {
    pub model_path: Option<String>,
    pub is_extracting: Arc<Mutex<bool>>,
}

impl EntityExtractor {
    pub fn new() -> Self {
        Self {
            model_path: None,
            is_extracting: Arc::new(Mutex::new(false)),
        }
    }

    pub fn get_available_models() -> Vec<ModelInfo> {
        vec![ModelInfo {
            name: "gliner-multi-v2.1-onnx".to_string(),
            size_mb: 1123, // 1,177,789,050 bytes = ~1.12 GB
            download_url:
                "https://huggingface.co/juampahc/gliner_multi-v2.1-onnx/resolve/main/model.onnx"
                    .to_string(),
            is_downloaded: false, // Will be checked dynamically
            local_path: None,
        }]
    }

    pub fn get_available_models_with_status(models_dir: &Path) -> Vec<ModelInfo> {
        Self::get_available_models()
            .into_iter()
            .map(|mut model| {
                let model_file_path = models_dir.join(&model.name).join("model.onnx");
                model.is_downloaded = model_file_path.exists();
                if model.is_downloaded {
                    model.local_path = Some(model_file_path.to_string_lossy().to_string());
                }
                model
            })
            .collect()
    }

    pub async fn download_model(
        model_name: &str,
        models_dir: &Path,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Ensure models directory exists
        let model_dir = models_dir.join(model_name);
        tokio::fs::create_dir_all(&model_dir).await?;

        let model_file_path = model_dir.join("model.onnx");

        // Check if model already exists
        if model_file_path.exists() {
            return Ok(model_file_path.to_string_lossy().to_string());
        }

        let models = Self::get_available_models();
        let model_info = models
            .iter()
            .find(|m| m.name == model_name)
            .ok_or("Model not found")?;

        // Download the model
        println!(
            "Downloading model {} from {}",
            model_name, model_info.download_url
        );

        let response = reqwest::get(&model_info.download_url).await?;
        let model_bytes = response.bytes().await?;

        tokio::fs::write(&model_file_path, &model_bytes).await?;

        println!(
            "Model {} downloaded successfully to {}",
            model_name,
            model_file_path.display()
        );

        Ok(model_file_path.to_string_lossy().to_string())
    }

    pub fn load_model(
        &mut self,
        model_path: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // For now, just store the path. In a real implementation, we would load the GLiNER model here
        self.model_path = Some(model_path.to_string());
        println!("Model loaded from: {model_path}");
        Ok(())
    }

    pub fn is_model_loaded(&self) -> bool {
        self.model_path.is_some()
    }

    pub fn is_extracting(&self) -> bool {
        *self.is_extracting.lock().unwrap()
    }

    #[instrument(skip(items, database, is_extracting), fields(session_id, items_count = items.len()))]
    pub async fn extract_entities_from_items(
        items: Vec<HnItem>,
        database: &Database,
        session_id: i64,
        is_extracting: Arc<Mutex<bool>>,
        model_path: Option<String>,
    ) -> Result<(u64, u64), Box<dyn std::error::Error + Send + Sync>> {
        let start_time = Instant::now();
        
        info!("Starting entity extraction for session {} with {} items using model: {}",
              session_id,
              items.len(),
              model_path.as_deref().unwrap_or("none"));

        if model_path.is_none() {
            error!("Entity extraction failed: model not loaded");
            return Err("Model not loaded".into());
        }

        // Set extraction status
        {
            let mut extracting = is_extracting.lock().unwrap();
            *extracting = true;
        }

        let mut entities_extracted = 0u64;
        let mut items_processed = 0u64;

        // Define entity types we want to extract
        let entity_types = vec![
            "person",
            "company",
            "organization",
            "location",
            "date",
            "money",
            "product",
            "technology",
        ];

        for item in items {
            // Check if we should stop
            if !*is_extracting.lock().unwrap() {
                break;
            }

            // Extract entities from both title and text
            let mut text_to_process = Vec::new();

            if let Some(title) = &item.title {
                if !title.trim().is_empty() {
                    text_to_process.push(("title", title.clone()));
                }
            }

            if let Some(text) = &item.text {
                if !text.trim().is_empty() {
                    text_to_process.push(("text", text.clone()));
                }
            }

            for (_source, text) in text_to_process {
                // For demonstration, we'll use a simple mock extraction
                // In a real implementation, this would use gline-rs
                let mock_entities = Self::mock_extract_entities(&text, &entity_types);
                let mut text_entities = Vec::new();

                for (entity_type, entity_value, start, end) in mock_entities {
                    // The entity_value from mock_extract_entities is already effectively "trimmed"
                    // as it uses predefined strings. If it were from raw text, trimming here would be crucial.
                    let trimmed_entity_value = entity_value.trim().to_string();

                    // Get or insert the unique entity
                    let entity_id_result = database
                        .get_or_insert_entity(&entity_type, &trimmed_entity_value)
                        .await;

                    match entity_id_result {
                        Ok(entity_id) => {
                            // Store entity info for relation extraction
                            text_entities.push((
                                entity_id,
                                entity_type.clone(),
                                trimmed_entity_value.clone(),
                                start,
                                end,
                            ));

                            // Insert the entity reference
                            if let Err(e) = database
                                .insert_entity_reference(
                                    item.id,
                                    entity_id,
                                    &text[start..end],
                                    start as i64,
                                    end as i64,
                                    Some(0.85), // Mock confidence
                                )
                                .await
                            {
                                eprintln!("Failed to insert entity reference: {e}");
                            } else {
                                entities_extracted += 1;
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to get or insert entity: {e}");
                        }
                    }
                }

                // Extract relations between entities in this text
                let mock_relations = Self::mock_extract_relations(&text, &text_entities);
                for (subject_id, object_id, relation_type, start, end, confidence) in mock_relations
                {
                    match database
                        .get_or_insert_relation(
                            subject_id,
                            object_id,
                            &relation_type,
                            Some(confidence),
                        )
                        .await
                    {
                        Ok(relation_id) => {
                            if let Err(e) = database
                                .insert_relation_reference(
                                    item.id,
                                    relation_id,
                                    &text[start..end],
                                    start as i64,
                                    end as i64,
                                    Some(confidence),
                                )
                                .await
                            {
                                eprintln!("Failed to insert relation reference: {e}");
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to get or insert relation: {e}");
                        }
                    }
                }
            }

            items_processed += 1;

            // Update progress every 10 items
            if items_processed % 10 == 0 {
                let _ = database
                    .update_extraction_session(session_id, entities_extracted, items_processed)
                    .await;
            }
        }

        // Set extraction status to false
        {
            let mut extracting = is_extracting.lock().unwrap();
            *extracting = false;
        }

        let duration = start_time.elapsed();
        info!("Entity extraction completed for session {} - extracted {} entities from {} items in {}ms",
              session_id,
              entities_extracted,
              items_processed,
              duration.as_millis());

        Ok((entities_extracted, items_processed))
    }

    pub fn stop_extraction(&self) {
        let mut extracting = self.is_extracting.lock().unwrap();
        *extracting = false;
    }

    // Mock entity extraction for demonstration
    // In a real implementation, this would use gline-rs
    fn mock_extract_entities(
        text: &str,
        _entity_types: &[&str],
    ) -> Vec<(String, String, usize, usize)> {
        let mut entities = Vec::new();

        // Simple pattern matching for demonstration
        // Company patterns
        let company_patterns = [
            "Apple",
            "Google",
            "Microsoft",
            "Amazon",
            "Facebook",
            "Meta",
            "Tesla",
            "OpenAI",
            "GitHub",
            "Netflix",
            "Uber",
            "Airbnb",
            "Spotify",
            "Twitter",
            "LinkedIn",
            "Instagram",
        ];

        // Person patterns (common names in tech)
        let person_patterns = [
            "Elon Musk",
            "Jeff Bezos",
            "Bill Gates",
            "Steve Jobs",
            "Mark Zuckerberg",
            "Satya Nadella",
            "Tim Cook",
            "Larry Page",
            "Sergey Brin",
            "Jack Dorsey",
        ];

        // Technology patterns
        let tech_patterns = [
            "JavaScript",
            "Python",
            "Rust",
            "Go",
            "TypeScript",
            "React",
            "Node.js",
            "Docker",
            "Kubernetes",
            "AWS",
            "Azure",
            "GCP",
            "AI",
            "ML",
            "blockchain",
        ];

        let text_lower = text.to_lowercase();

        // Find companies
        for pattern in &company_patterns {
            if let Some(start) = text_lower.find(&pattern.to_lowercase()) {
                entities.push((
                    "company".to_string(),
                    pattern.to_string(),
                    start,
                    start + pattern.len(),
                ));
            }
        }

        // Find persons
        for pattern in &person_patterns {
            if let Some(start) = text_lower.find(&pattern.to_lowercase()) {
                entities.push((
                    "person".to_string(),
                    pattern.to_string(),
                    start,
                    start + pattern.len(),
                ));
            }
        }

        // Find technologies
        for pattern in &tech_patterns {
            if let Some(start) = text_lower.find(&pattern.to_lowercase()) {
                entities.push((
                    "technology".to_string(),
                    pattern.to_string(),
                    start,
                    start + pattern.len(),
                ));
            }
        }

        entities
    }

    // Mock relation extraction for demonstration
    // In a real implementation, this would use gline-rs relation extraction
    fn mock_extract_relations(
        text: &str,
        entities: &[(i64, String, String, usize, usize)], // (entity_id, entity_type, entity_value, start, end)
    ) -> Vec<(i64, i64, String, usize, usize, f64)> {
        // (subject_id, object_id, relation_type, start, end, confidence)
        let mut relations = Vec::new();
        let text_lower = text.to_lowercase();

        // Simple pattern-based relation extraction
        for (relation_type, _pattern) in RELATION_PATTERNS {
            let relation_keywords = match *relation_type {
                "founded" => vec!["founded", "started", "launched", "established"],
                "co_founded" => vec!["co-founded", "co-started", "cofounded"],
                "invested_in" => vec!["invested in", "funding", "backed", "financing"],
                "works_at" => vec!["works at", "employed at", "engineer at", "ceo of"],
                "acquired" => vec!["acquired", "bought", "purchased", "merger"],
                "created" => vec!["created", "built", "made", "designed"],
                "developed" => vec!["developed", "built", "engineered"],
                "leads" => vec!["leads", "heading", "ceo of", "cto of", "founder of"],
                _ => continue,
            };

            for keyword in relation_keywords {
                if let Some(keyword_pos) = text_lower.find(keyword) {
                    // Look for entity pairs around this relation keyword
                    for (
                        i,
                        (subject_id, subject_type, _subject_value, subject_start, subject_end),
                    ) in entities.iter().enumerate()
                    {
                        for (object_id, object_type, _object_value, object_start, object_end) in
                            entities.iter().skip(i + 1)
                        {
                            // Check if this relation makes sense for these entity types
                            if Self::is_valid_relation_for_types(
                                relation_type,
                                subject_type,
                                object_type,
                            ) {
                                // Check if entities are close to the relation keyword
                                let distance_to_subject = if *subject_end < keyword_pos {
                                    keyword_pos - subject_end
                                } else {
                                    subject_start.saturating_sub(keyword_pos + keyword.len())
                                };

                                let distance_to_object = if *object_end < keyword_pos {
                                    keyword_pos - object_end
                                } else {
                                    object_start.saturating_sub(keyword_pos + keyword.len())
                                };

                                // If both entities are reasonably close to the relation keyword (within 50 characters)
                                if distance_to_subject < 50 && distance_to_object < 50 {
                                    let start_pos =
                                        (*subject_start).min(*object_start).min(keyword_pos);
                                    let end_pos = (*subject_end)
                                        .max(*object_end)
                                        .max(keyword_pos + keyword.len());

                                    // Base confidence on proximity and entity types
                                    let proximity_score = 1.0
                                        - (distance_to_subject + distance_to_object) as f64 / 100.0;
                                    let confidence = (0.7 + proximity_score * 0.3).min(0.95);

                                    relations.push((
                                        *subject_id,
                                        *object_id,
                                        relation_type.to_string(),
                                        start_pos,
                                        end_pos,
                                        confidence,
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }

        relations
    }

    // Check if a relation type is valid for given entity types
    fn is_valid_relation_for_types(
        relation_type: &str,
        subject_type: &str,
        object_type: &str,
    ) -> bool {
        match relation_type {
            "founded" | "co_founded" => subject_type == "person" && object_type == "company",
            "invested_in" => {
                (subject_type == "person" || subject_type == "company") && object_type == "company"
            }
            "works_at" => subject_type == "person" && object_type == "company",
            "acquired" => subject_type == "company" && object_type == "company",
            "created" | "developed" => {
                (subject_type == "person" || subject_type == "company")
                    && (object_type == "product" || object_type == "technology")
            }
            "leads" => {
                subject_type == "person"
                    && (object_type == "company" || object_type == "organization")
            }
            "partnered_with" | "competes_with" => {
                subject_type == "company" && object_type == "company"
            }
            "used_by" => subject_type == "technology" && object_type == "company",
            "backed_by" => {
                subject_type == "company" && (object_type == "person" || object_type == "company")
            }
            _ => false,
        }
    }
}

impl Default for EntityExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::Database;
    use chrono::Utc;

    use tempfile::tempdir;

    async fn setup_test_db() -> (Database, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::new(&db_path).await.unwrap();
        (db, dir)
    }

    #[tokio::test]
    async fn test_entity_uniqueness_and_referencing() {
        let (db, _dir) = setup_test_db().await;
        let is_extracting = Arc::new(Mutex::new(false));
        let model_path = Some("mock_model_path".to_string());
        let session_id = db.start_extraction_session().await.unwrap();

        // Create a mock HnItem with text that contains entities
        let item = HnItem {
            id: 1,
            item_type: "story".to_string(),
            by: Some("testuser".to_string()),
            time: Utc::now(),
            text: Some("Big news from Apple and Microsoft. Also, Rust is cool.".to_string()),
            url: None,
            score: None,
            title: Some("Tech News".to_string()),
            parent: None,
            kids: None,
            descendants: None,
            deleted: false,
            dead: false,
            created_at: Utc::now(),
        };
        db.insert_item(&item).await.unwrap(); // Ensure item exists in DB

        // First extraction pass
        let result1 = EntityExtractor::extract_entities_from_items(
            vec![item.clone()],
            &db,
            session_id,
            is_extracting.clone(),
            model_path.clone(),
        )
        .await
        .unwrap();

        assert_eq!(result1.0, 3); // 3 entities found: Apple, Google, Rust
        assert_eq!(result1.1, 1); // 1 item processed

        // Verify database state
        let stats1 = db.get_extraction_stats().await.unwrap();
        assert_eq!(stats1.total_entities, 3);
        assert_eq!(stats1.total_items_processed, 1);

        // Check unique entities table
        let entities: Vec<crate::database::Entity> = sqlx::query_as("SELECT * FROM entities")
            .fetch_all(&db.pool)
            .await
            .unwrap();
        assert_eq!(entities.len(), 3);

        // Check entity references table
        let refs1: Vec<crate::database::EntityReference> =
            sqlx::query_as("SELECT * FROM entity_references")
                .fetch_all(&db.pool)
                .await
                .unwrap();
        assert_eq!(refs1.len(), 3);

        // Second extraction pass with the same item
        // Because get_items_for_extraction now excludes processed items,
        // we need to manually call extract_entities_from_items again to simulate re-processing.
        // In a real run, this item wouldn't be fetched again.
        let _result2 = EntityExtractor::extract_entities_from_items(
            vec![item],
            &db,
            session_id,
            is_extracting.clone(),
            model_path.clone(),
        )
        .await
        .unwrap();

        // Since the item is processed again, it will re-insert references. Let's adjust the test logic.
        // A better test would be to check if get_items_for_extraction returns this item.
        // For now, let's confirm that no new *unique* entities are created.

        let stats2 = db.get_extraction_stats().await.unwrap();
        assert_eq!(stats2.total_entities, 3); // Still 3 unique entities

        // Check that the references have doubled, since we re-processed the same item.
        // This confirms the logic of `extract_entities_from_items` itself.
        let refs2: Vec<crate::database::EntityReference> =
            sqlx::query_as("SELECT * FROM entity_references")
                .fetch_all(&db.pool)
                .await
                .unwrap();
        assert_eq!(refs2.len(), 6); // 3 new references were added
    }

    #[tokio::test]
    async fn test_relation_extraction() {
        let (db, _dir) = setup_test_db().await;
        let is_extracting = Arc::new(Mutex::new(false));
        let model_path = Some("mock_model_path".to_string());
        let session_id = db.start_extraction_session().await.unwrap();

        // Create a mock HnItem with text that contains entities and relations
        let item = HnItem {
            id: 1,
            item_type: "story".to_string(),
            by: Some("testuser".to_string()),
            time: Utc::now(),
            text: Some(
                "Elon Musk founded Tesla in 2003. Microsoft acquired GitHub for $7.5 billion."
                    .to_string(),
            ),
            url: None,
            score: None,
            title: Some("Tech News".to_string()),
            created_at: Utc::now(),
            parent: None,
            kids: None,
            descendants: None,
            deleted: false,
            dead: false,
        };
        db.insert_item(&item).await.unwrap();

        // Run extraction
        let result = EntityExtractor::extract_entities_from_items(
            vec![item],
            &db,
            session_id,
            is_extracting,
            model_path,
        )
        .await
        .unwrap();

        // Should extract entities: Elon Musk (person), Tesla (company), Microsoft (company), GitHub (company)
        assert!(result.0 >= 4); // At least 4 entities extracted
        assert_eq!(result.1, 1); // 1 item processed

        // Check that relations were created
        let relations_count = db.get_total_relations_count().await.unwrap();
        assert!(relations_count >= 1); // At least one relation should be found

        // Get all relations and verify types
        let relations = db.get_relations_paginated(10, 0).await.unwrap();
        assert!(!relations.is_empty());

        // Should find "founded" or "acquired" relations
        let relation_types: Vec<&str> =
            relations.iter().map(|r| r.relation_type.as_str()).collect();
        assert!(
            relation_types
                .iter()
                .any(|&t| t == "founded" || t == "acquired")
        );
    }
}
