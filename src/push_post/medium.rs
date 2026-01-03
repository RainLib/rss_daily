use crate::models::Repository;
use crate::push_post::PostPlatform;
use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub struct MediumPlatform {
    client: Client,
    integration_token: String,
}

#[derive(Debug, Deserialize)]
struct MediumUser {
    id: String,
    username: String,
}

#[derive(Debug, Deserialize)]
struct MediumUserResponse {
    data: MediumUser,
}

#[derive(Debug, Serialize)]
struct CreatePostRequest {
    title: String,
    #[serde(rename = "contentFormat")]
    content_format: String,
    content: String,
    tags: Vec<String>,
    #[serde(rename = "publishStatus")]
    publish_status: String,
}

#[derive(Debug, Deserialize)]
struct PostData {
    id: String,
    title: String,
    url: String,
}

#[derive(Debug, Deserialize)]
struct CreatePostResponse {
    data: PostData,
}

impl MediumPlatform {
    pub fn new(integration_token: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("rss-daily/1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            integration_token,
        }
    }

    /// Get the authenticated user's ID
    async fn get_user_id(&self) -> Result<String> {
        let url = "https://api.medium.com/v1/me";

        let response = self
            .client
            .get(url)
            .bearer_auth(&self.integration_token)
            .send()
            .await
            .context("Failed to get user info")?;

        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Failed to get user info: {}", text);
        }

        let user_response: MediumUserResponse = response
            .json()
            .await
            .context("Failed to parse user response")?;

        Ok(user_response.data.id)
    }

    /// Create a new post
    async fn create_post(
        &self,
        user_id: &str,
        title: &str,
        content: &str,
        tags: Vec<String>,
    ) -> Result<String> {
        let url = format!("https://api.medium.com/v1/users/{}/posts", user_id);

        let post_request = CreatePostRequest {
            title: title.to_string(),
            content_format: "markdown".to_string(),
            content: content.to_string(),
            tags,
            publish_status: "public".to_string(), // "public", "draft", or "unlisted"
        };

        let response = self
            .client
            .post(&url)
            .bearer_auth(&self.integration_token)
            .json(&post_request)
            .send()
            .await
            .context("Failed to create post")?;

        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Failed to create post: {}", text);
        }

        let post_response: CreatePostResponse = response
            .json()
            .await
            .context("Failed to parse post response")?;

        Ok(post_response.data.url)
    }
}

#[async_trait::async_trait]
impl PostPlatform for MediumPlatform {
    fn name(&self) -> &str {
        "Medium"
    }

    async fn push_repository(&mut self, repo: &Repository, content: &str) -> Result<String> {
        // Get user ID first
        let user_id = self.get_user_id().await?;

        let title = format!("GitHub Trending: {}", repo.name);

        // Extract tags from repo
        let mut tags = vec!["GitHub".to_string(), "Open Source".to_string()];
        if let Some(lang) = &repo.language {
            tags.push(lang.clone());
        }
        // Add first topic if available
        if let Some(topic) = repo.topics.first() {
            tags.push(topic.clone());
        }
        // Medium allows max 5 tags
        tags.truncate(5);

        self.create_post(&user_id, &title, content, tags).await
    }
}
