use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub url: Option<String>,
    pub username: Option<String>,
    pub timezone: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            url: None,
            username: None,
            timezone: Some("UTC".to_string()), // Default to UTC
        }
    }
}

impl Config {
    const CONFIG_FILE: &'static str = "qbittui_config.json";

    pub fn load() -> Self {
        if Path::new(Self::CONFIG_FILE).exists() {
            match fs::read_to_string(Self::CONFIG_FILE) {
                Ok(content) => {
                    match serde_json::from_str(&content) {
                        Ok(config) => config,
                        Err(e) => {
                            eprintln!("Failed to parse config file: {}", e);
                            Self::default()
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to read config file: {}", e);
                    Self::default()
                }
            }
        } else {
            Self::default()
        }
    }

    pub fn save(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(Self::CONFIG_FILE, content)?;
        Ok(())
    }

    pub fn update_connection_info(&mut self, url: &str, username: &str) -> Result<()> {
        self.url = Some(url.to_string());
        self.username = Some(username.to_string());
        self.save()
    }

    pub fn get_last_url(&self) -> Option<String> {
        self.url.clone()
    }

    pub fn get_last_username(&self) -> Option<String> {
        self.username.clone()
    }

    pub fn get_timezone(&self) -> String {
        self.timezone.clone().unwrap_or_else(|| "UTC".to_string())
    }

    pub fn set_timezone(&mut self, timezone: &str) -> Result<()> {
        self.timezone = Some(timezone.to_string());
        self.save()
    }
}
