use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use url::Url;
use crate::utils::log_debug;

#[derive(Debug, Clone, Deserialize)]
pub struct Torrent {
    pub hash: String,
    pub name: String,
    pub size: i64,
    pub progress: f64,
    pub dlspeed: i64,
    pub upspeed: i64,
    #[serde(default)]
    pub eta: Option<i64>,
    pub state: String,
    #[serde(default)]
    pub priority: Option<i32>,
    #[serde(default)]
    pub num_seeds: Option<i32>,
    #[serde(default)]
    pub num_leechs: Option<i32>,
    #[serde(default)]
    pub ratio: Option<f64>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub tags: Option<String>,
    #[serde(default)]
    pub added_on: Option<i64>,
    #[serde(default)]
    pub completion_on: Option<i64>,
    #[serde(default)]
    pub downloaded: Option<i64>,
    #[serde(default)]
    pub uploaded: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerState {
    pub connection_status: String,
    #[serde(default)]
    pub dht_nodes: Option<i32>,
    pub dl_info_data: i64,
    pub dl_info_speed: i64,
    #[serde(default)]
    pub dl_rate_limit: Option<i64>,
    pub up_info_data: i64,
    pub up_info_speed: i64,
    #[serde(default)]
    pub up_rate_limit: Option<i64>,
    #[serde(default)]
    pub queueing: Option<bool>,
    #[serde(default)]
    pub use_alt_speed_limits: Option<bool>,
    #[serde(default)]
    pub refresh_interval: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Category {
    pub name: String,
    pub savePath: String,
}

pub struct QBittorrentClient {
    client: Client,
    base_url: Url,
    authenticated: bool,
}

impl QBittorrentClient {
    pub fn new(base_url: Url) -> Self {
        let client = Client::builder()
            .cookie_store(true)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url,
            authenticated: false,
        }
    }

    pub async fn login(&mut self, username: &str, password: &str) -> Result<()> {
        let login_url = self.base_url.join("/api/v2/auth/login")?;

        let mut params = HashMap::new();
        params.insert("username", username);
        params.insert("password", password);

        let response = self
            .client
            .post(login_url)
            .form(&params)
            .send()
            .await?;

        if response.status().is_success() {
            let text = response.text().await?;
            if text == "Ok." {
                self.authenticated = true;
                Ok(())
            } else {
                Err(anyhow!("Login failed: {}", text))
            }
        } else {
            Err(anyhow!("Login request failed: {}", response.status()))
        }
    }

    pub async fn get_torrents(&self) -> Result<Vec<Torrent>> {
        self.ensure_authenticated().await?;

        let url = self.base_url.join("/api/v2/torrents/info")?;
        let response = self.client.get(url).send().await?;

        if response.status().is_success() {
            let torrents: Vec<Torrent> = response.json().await?;
            Ok(torrents)
        } else {
            Err(anyhow!("Failed to get torrents: {}", response.status()))
        }
    }

    pub async fn get_server_state(&self) -> Result<ServerState> {
        self.ensure_authenticated().await?;

        let url = self.base_url.join("/api/v2/transfer/info")?;
        let response = self.client.get(url).send().await?;

        if response.status().is_success() {
            let state: ServerState = response.json().await?;
            Ok(state)
        } else {
            Err(anyhow!("Failed to get server state: {}", response.status()))
        }
    }

    pub async fn get_categories(&self) -> Result<HashMap<String, Category>> {
        self.ensure_authenticated().await?;

        let url = self.base_url.join("/api/v2/torrents/categories")?;
        let response = self.client.get(url).send().await?;

        if response.status().is_success() {
            let categories: HashMap<String, Category> = response.json().await?;
            Ok(categories)
        } else {
            Err(anyhow!("Failed to get categories: {}", response.status()))
        }
    }

    pub async fn pause_torrent(&self, hash: &str, timezone: &str) -> Result<()> {
        self.ensure_authenticated().await?;

        let url = self.base_url.join("/api/v2/torrents/stop")?;
        let mut params = HashMap::new();
        params.insert("hashes", hash);

        log_debug(&format!("Pausing torrent with hash: {}", hash), timezone);
        log_debug(&format!("Request URL: {}", url), timezone);

        let response = self.client.post(url).form(&params).send().await?;

        if response.status().is_success() {
            log_debug("Pause successful", timezone);
            Ok(())
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            log_debug(&format!("Pause failed - Status: {}, Body: {}", status, body), timezone);
            Err(anyhow!("Failed to pause torrent: {}", status))
        }
    }

    pub async fn resume_torrent(&self, hash: &str, timezone: &str) -> Result<()> {
        self.ensure_authenticated().await?;

        let url = self.base_url.join("/api/v2/torrents/start")?;
        let mut params = HashMap::new();
        params.insert("hashes", hash);

        log_debug(&format!("Resuming torrent with hash: {}", hash), timezone);
        log_debug(&format!("Request URL: {}", url), timezone);

        let response = self.client.post(url).form(&params).send().await?;

        if response.status().is_success() {
            log_debug("Resume successful", timezone);
            Ok(())
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_else(|_| "Unable to read response body".to_string());
            log_debug(&format!("Resume failed - Status: {}, Body: {}", status, body), timezone);
            Err(anyhow!("Failed to resume torrent: {} - {}", status, body))
        }
    }

    pub async fn delete_torrent(&self, hash: &str, delete_files: bool) -> Result<()> {
        self.ensure_authenticated().await?;

        let url = self.base_url.join("/api/v2/torrents/delete")?;
        let mut params = HashMap::new();
        params.insert("hashes", hash);
        params.insert("deleteFiles", if delete_files { "true" } else { "false" });

        let response = self.client.post(url).form(&params).send().await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_else(|_| "Unable to read response body".to_string());
            Err(anyhow!("Failed to delete torrent: {} - {}", status, body))
        }
    }

    pub async fn add_torrent(&self, torrent_data: &[u8], save_path: Option<&str>) -> Result<()> {
        self.ensure_authenticated().await?;

        let url = self.base_url.join("/api/v2/torrents/add")?;

        let form = reqwest::multipart::Form::new()
            .part("torrents", reqwest::multipart::Part::bytes(torrent_data.to_vec())
                .file_name("torrent.torrent")
                .mime_str("application/x-bittorrent")?);

        let form = if let Some(path) = save_path {
            form.text("savepath", path.to_string())
        } else {
            form
        };

        let response = self.client.post(url).multipart(form).send().await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(anyhow!("Failed to add torrent: {}", response.status()))
        }
    }

    async fn ensure_authenticated(&self) -> Result<()> {
        if !self.authenticated {
            return Err(anyhow!("Not authenticated"));
        }

        // Test if session is still valid by making a simple API call
        let url = self.base_url.join("/api/v2/app/version")?;
        let response = self.client.get(url).send().await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(anyhow!("Authentication session expired or invalid"))
        }
    }

    pub async fn check_authentication(&self) -> Result<bool> {
        let url = self.base_url.join("/api/v2/app/version")?;
        let response = self.client.get(url).send().await?;
        Ok(response.status().is_success())
    }

    pub fn get_base_url(&self) -> &Url {
        &self.base_url
    }
}
