use crate::database::{Database, Entity, HnItem};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
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

    pub async fn extract_entities_from_items(
        items: Vec<HnItem>,
        database: &Database,
        session_id: i64,
        is_extracting: Arc<Mutex<bool>>,
        model_path: Option<String>,
    ) -> Result<(u64, u64), Box<dyn std::error::Error + Send + Sync>> {
        if model_path.is_none() {
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

                for (entity_type, entity_value, start, end) in mock_entities {
                    let entity = Entity {
                        id: 0, // Will be auto-generated
                        item_id: item.id,
                        entity_type: entity_type.clone(),
                        entity_value: entity_value.clone(),
                        original_text: text[start..end].to_string(),
                        start_offset: start as i64,
                        end_offset: end as i64,
                        confidence: Some(0.85), // Mock confidence
                        created_at: Utc::now(),
                    };

                    if let Err(e) = database.insert_entity(&entity).await {
                        eprintln!("Failed to insert entity: {e}");
                    } else {
                        entities_extracted += 1;
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
}

impl Default for EntityExtractor {
    fn default() -> Self {
        Self::new()
    }
}
