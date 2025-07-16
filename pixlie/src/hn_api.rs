use crate::database::{HnItem, HnUser};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Deserialize;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{error, info, instrument, warn};

const HN_API_BASE: &str = "https://hacker-news.firebaseio.com/v0";
const REQUEST_DELAY_MS: u64 = 100; // Rate limiting: 10 requests per second

#[derive(Debug, Deserialize)]
struct HnApiItem {
    id: i64,
    #[serde(rename = "type")]
    item_type: Option<String>,
    by: Option<String>,
    time: Option<i64>,
    text: Option<String>,
    url: Option<String>,
    score: Option<i64>,
    title: Option<String>,
    parent: Option<i64>,
    kids: Option<Vec<i64>>,
    descendants: Option<i64>,
    deleted: Option<bool>,
    dead: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct HnApiUser {
    id: String,
    created: i64,
    karma: Option<i64>,
    about: Option<String>,
    submitted: Option<Vec<i64>>,
}

pub struct HnApiClient {
    client: Client,
}

impl Default for HnApiClient {
    fn default() -> Self {
        Self::new()
    }
}

impl HnApiClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    pub async fn get_max_item_id(&self) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{HN_API_BASE}/maxitem.json");
        let response = self.client.get(&url).send().await?;
        let max_id: i64 = response.json().await?;
        Ok(max_id)
    }

    pub async fn get_top_stories(
        &self,
    ) -> Result<Vec<i64>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{HN_API_BASE}/topstories.json");
        let response = self.client.get(&url).send().await?;
        let story_ids: Vec<i64> = response.json().await?;
        Ok(story_ids)
    }

    #[allow(dead_code)]
    pub async fn get_new_stories(
        &self,
    ) -> Result<Vec<i64>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{HN_API_BASE}/newstories.json");
        let response = self.client.get(&url).send().await?;
        let story_ids: Vec<i64> = response.json().await?;
        Ok(story_ids)
    }

    #[allow(dead_code)]
    pub async fn get_best_stories(
        &self,
    ) -> Result<Vec<i64>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{HN_API_BASE}/beststories.json");
        let response = self.client.get(&url).send().await?;
        let story_ids: Vec<i64> = response.json().await?;
        Ok(story_ids)
    }

    #[allow(dead_code)]
    pub async fn get_ask_stories(
        &self,
    ) -> Result<Vec<i64>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{HN_API_BASE}/askstories.json");
        let response = self.client.get(&url).send().await?;
        let story_ids: Vec<i64> = response.json().await?;
        Ok(story_ids)
    }

    #[allow(dead_code)]
    pub async fn get_show_stories(
        &self,
    ) -> Result<Vec<i64>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{HN_API_BASE}/showstories.json");
        let response = self.client.get(&url).send().await?;
        let story_ids: Vec<i64> = response.json().await?;
        Ok(story_ids)
    }

    #[allow(dead_code)]
    pub async fn get_job_stories(
        &self,
    ) -> Result<Vec<i64>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{HN_API_BASE}/jobstories.json");
        let response = self.client.get(&url).send().await?;
        let story_ids: Vec<i64> = response.json().await?;
        Ok(story_ids)
    }

    pub async fn get_item(
        &self,
        id: i64,
    ) -> Result<Option<HnItem>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{HN_API_BASE}/item/{id}.json");
        let response = self.client.get(&url).send().await?;

        if response.status() == 404 {
            return Ok(None);
        }

        let api_item: HnApiItem = response.json().await?;

        // Convert API response to our database model
        let item = HnItem {
            id: api_item.id,
            item_type: api_item.item_type.unwrap_or_else(|| "unknown".to_string()),
            by: api_item.by,
            time: api_item
                .time
                .and_then(|t| DateTime::from_timestamp(t, 0))
                .unwrap_or_else(Utc::now),
            text: api_item.text,
            url: api_item.url,
            score: api_item.score,
            title: api_item.title,
            parent: api_item.parent,
            kids: api_item
                .kids
                .map(|kids| serde_json::to_string(&kids).unwrap_or_default()),
            descendants: api_item.descendants,
            deleted: api_item.deleted.unwrap_or(false),
            dead: api_item.dead.unwrap_or(false),
            created_at: Utc::now(),
        };

        sleep(Duration::from_millis(REQUEST_DELAY_MS)).await;
        Ok(Some(item))
    }

    #[allow(dead_code)]
    pub async fn get_user(
        &self,
        username: &str,
    ) -> Result<Option<HnUser>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{HN_API_BASE}/user/{username}.json");
        let response = self.client.get(&url).send().await?;

        if response.status() == 404 {
            return Ok(None);
        }

        let api_user: HnApiUser = response.json().await?;

        // Convert API response to our database model
        let user = HnUser {
            id: api_user.id,
            created: DateTime::from_timestamp(api_user.created, 0).unwrap_or_else(Utc::now),
            karma: api_user.karma,
            about: api_user.about,
            submitted: api_user
                .submitted
                .map(|submitted| serde_json::to_string(&submitted).unwrap_or_default()),
            created_at: Utc::now(),
        };

        sleep(Duration::from_millis(REQUEST_DELAY_MS)).await;
        Ok(Some(user))
    }

    #[allow(dead_code)]
    pub async fn download_items_range(
        &self,
        start_id: i64,
        end_id: i64,
        callback: impl Fn(HnItem, u64, u64) + Send + Sync,
    ) -> Result<(u64, u64), Box<dyn std::error::Error>> {
        let mut downloaded = 0u64;
        let mut errors = 0u64;
        let total = (end_id - start_id + 1) as u64;

        for id in start_id..=end_id {
            match self.get_item(id).await {
                Ok(Some(item)) => {
                    downloaded += 1;
                    callback(item, downloaded, errors);
                }
                Ok(None) => {
                    // Item doesn't exist, not an error
                }
                Err(e) => {
                    errors += 1;
                    eprintln!("Error fetching item {id}: {e}");
                }
            }

            // Progress update every 100 items
            if (id - start_id) % 100 == 0 {
                let progress = ((id - start_id) as f64 / total as f64 * 100.0) as u32;
                println!("Progress: {}% ({}/{})", progress, id - start_id + 1, total);
            }
        }

        Ok((downloaded, errors))
    }

    #[allow(dead_code)]
    pub async fn download_stories_batch(
        &self,
        story_ids: Vec<i64>,
        callback: impl Fn(HnItem, u64, u64) + Send + Sync,
    ) -> Result<(u64, u64), Box<dyn std::error::Error>> {
        let mut downloaded = 0u64;
        let mut errors = 0u64;
        let total = story_ids.len();

        for (index, &story_id) in story_ids.iter().enumerate() {
            match self.get_item(story_id).await {
                Ok(Some(item)) => {
                    downloaded += 1;
                    callback(item, downloaded, errors);
                }
                Ok(None) => {
                    errors += 1;
                    eprintln!("Story {story_id} not found");
                }
                Err(e) => {
                    errors += 1;
                    eprintln!("Error fetching story {story_id}: {e}");
                }
            }

            // Progress update every 50 stories
            if index % 50 == 0 {
                let progress = (index as f64 / total as f64 * 100.0) as u32;
                println!("Stories progress: {}% ({}/{})", progress, index + 1, total);
            }
        }

        Ok((downloaded, errors))
    }
}
