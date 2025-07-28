use crate::api::{QBittorrentClient, ServerState, Torrent};
use crate::config::Config;
use crate::utils::log_debug;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::time::{Duration, Instant};
use url::Url;

#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    UrlConfig,
    Login,
    Main,
    AddTorrent,
    Search,
    ConfirmDelete,
    Error(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Url,
    Username,
    Password,
    TorrentPath,
    Search,
    None,
}

pub struct App {
    pub client: QBittorrentClient,
    pub config: Config,
    pub state: AppState,
    pub input_mode: InputMode,
    pub url_input: String,
    pub username_input: String,
    pub password_input: String,
    pub torrent_path_input: String,
    pub search_input: String,
    pub torrents: Vec<Torrent>,
    pub filtered_torrents: Vec<Torrent>,
    pub selected_torrent: usize,
    pub server_state: Option<ServerState>,
    pub last_update: Instant,
    pub should_quit: bool,
    pub error_message: Option<String>,
    pub show_password: bool,
    pub scroll_offset: usize,
    pub delete_confirmation_hash: Option<String>,
    pub max_visible_rows: usize,
    pub terminal_width: u16,
    pub terminal_height: u16,
    pub is_searching: bool,
}

impl App {
    pub async fn new(
        base_url: Url,
        username: Option<String>,
        password: Option<String>,
    ) -> Result<Self> {
        let config = Config::load();
        Self::new_with_config(base_url, username, password, config).await
    }

    pub async fn new_with_config(
        base_url: Url,
        username: Option<String>,
        password: Option<String>,
        config: Config,
    ) -> Result<Self> {
        let client = QBittorrentClient::new(base_url.clone());

        // Use saved config if no CLI args provided
        let (initial_url, initial_username) = if username.is_none() && password.is_none() {
            (
                config
                    .get_last_url()
                    .unwrap_or_else(|| base_url.to_string()),
                config.get_last_username().unwrap_or_default(),
            )
        } else {
            (base_url.to_string(), String::new())
        };

        let mut app = Self {
            client,
            config,
            state: if username.is_some() && password.is_some() {
                AppState::Login // Skip URL config if CLI args provided
            } else {
                AppState::UrlConfig // Start with URL configuration
            },
            input_mode: InputMode::Url,
            url_input: initial_url,
            username_input: initial_username,
            password_input: String::new(),
            torrent_path_input: String::new(),
            search_input: String::new(),
            torrents: Vec::new(),
            filtered_torrents: Vec::new(),
            selected_torrent: 0,
            server_state: None,
            last_update: Instant::now(),
            should_quit: false,
            error_message: None,
            show_password: false,
            scroll_offset: 0,
            delete_confirmation_hash: None,
            max_visible_rows: 20,
            terminal_width: 80, // Default values
            terminal_height: 24,
            is_searching: false,
        };

        // If credentials were provided, try to login automatically
        if let (Some(user), Some(pass)) = (username, password) {
            app.username_input = user;
            app.password_input = pass;
            app.input_mode = InputMode::Username;
            app.attempt_login().await?;
        }

        Ok(app)
    }

    pub async fn handle_event(&mut self, event: crossterm::event::Event) -> Result<bool> {
        if let crossterm::event::Event::Key(key) = event {
            // Handle global Ctrl+Q quit - only with Ctrl modifier
            if key.code == KeyCode::Char('q') && key.modifiers.contains(KeyModifiers::CONTROL) {
                self.should_quit = true;
                return Ok(self.should_quit);
            }

            match self.state {
                AppState::UrlConfig => self.handle_url_config_input(key).await?,
                AppState::Login => self.handle_login_input(key).await?,
                AppState::Main => self.handle_main_input(key).await?,
                AppState::AddTorrent => self.handle_add_torrent_input(key).await?,
                AppState::Search => self.handle_search_input(key).await?,
                AppState::ConfirmDelete => self.handle_confirm_delete_input(key).await?,
                AppState::Error(_) => {
                    if key.code == KeyCode::Enter || key.code == KeyCode::Esc {
                        self.state = AppState::Main;
                        self.error_message = None;
                    }
                }
            }
        } // Auto-refresh torrents every 2 seconds when in main state
        if self.state == AppState::Main && self.last_update.elapsed() > Duration::from_secs(2) {
            self.refresh_data().await?;
        }

        Ok(self.should_quit)
    }

    pub fn handle_resize(&mut self, width: u16, height: u16) {
        self.terminal_width = width;
        self.terminal_height = height;

        // Recalculate max visible rows based on new height
        // Reserve space for header (3), footer (3), and some padding
        let available_height = height.saturating_sub(6);
        self.max_visible_rows = available_height.max(1) as usize;

        // Adjust scroll offset if necessary
        if self.scroll_offset + self.max_visible_rows > self.torrents.len() {
            self.scroll_offset = self.torrents.len().saturating_sub(self.max_visible_rows);
        }

        // Reset selected torrent if it's out of bounds
        if self.selected_torrent >= self.torrents.len() && !self.torrents.is_empty() {
            self.selected_torrent = self.torrents.len() - 1;
        }
    }

    async fn handle_url_config_input(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Enter => {
                if !self.url_input.is_empty() {
                    match Url::parse(&self.url_input) {
                        Ok(url) => {
                            self.client = QBittorrentClient::new(url);
                            self.state = AppState::Login;
                            self.input_mode = InputMode::Username;
                        }
                        Err(_) => {
                            self.error_message = Some("Invalid URL format. Please enter a valid URL (e.g., http://localhost:8080)".to_string());
                            self.state = AppState::Error("Invalid URL".to_string());
                        }
                    }
                }
            }
            KeyCode::Esc => {
                self.should_quit = true;
            }
            KeyCode::Char(c) => {
                self.url_input.push(c);
            }
            KeyCode::Backspace => {
                self.url_input.pop();
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_login_input(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Tab => {
                self.input_mode = match self.input_mode {
                    InputMode::Username => InputMode::Password,
                    InputMode::Password => InputMode::Username,
                    _ => InputMode::Username,
                };
            }
            KeyCode::Enter => {
                if !self.username_input.is_empty() && !self.password_input.is_empty() {
                    self.attempt_login().await?;
                }
            }
            KeyCode::Esc => {
                self.should_quit = true;
            }
            KeyCode::Char('h')
                if key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL) =>
            {
                self.show_password = !self.show_password;
            }
            _ => match self.input_mode {
                InputMode::Username => match key.code {
                    KeyCode::Char(c) => {
                        self.username_input.push(c);
                    }
                    KeyCode::Backspace => {
                        self.username_input.pop();
                    }
                    _ => {}
                },
                InputMode::Password => match key.code {
                    KeyCode::Char(c) => {
                        self.password_input.push(c);
                    }
                    KeyCode::Backspace => {
                        self.password_input.pop();
                    }
                    _ => {}
                },
                _ => {}
            },
        }
        Ok(())
    }

    async fn handle_main_input(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.state = AppState::Search;
                self.input_mode = InputMode::Search;
                self.search_input.clear();
                self.is_searching = true;
                self.filter_torrents();
            }
            KeyCode::Char('r') => self.refresh_data().await?,
            KeyCode::Char('a') => {
                self.state = AppState::AddTorrent;
                self.input_mode = InputMode::TorrentPath;
                self.torrent_path_input = String::new();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected_torrent > 0 {
                    self.selected_torrent -= 1;
                    self.adjust_scroll();
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let max_len = self.get_current_torrent_list_len();
                if self.selected_torrent < max_len.saturating_sub(1) {
                    self.selected_torrent += 1;
                    self.adjust_scroll();
                }
            }
            KeyCode::PageUp => {
                let page_size = self.get_max_visible_rows().saturating_sub(1).max(1);
                self.selected_torrent = self.selected_torrent.saturating_sub(page_size);
                self.adjust_scroll();
            }
            KeyCode::PageDown => {
                let page_size = self.get_max_visible_rows().saturating_sub(1).max(1);
                let max_len = self.get_current_torrent_list_len();
                self.selected_torrent =
                    (self.selected_torrent + page_size).min(max_len.saturating_sub(1));
                self.adjust_scroll();
            }
            KeyCode::Home => {
                self.selected_torrent = 0;
                self.adjust_scroll();
            }
            KeyCode::End => {
                let max_len = self.get_current_torrent_list_len();
                self.selected_torrent = max_len.saturating_sub(1);
                self.adjust_scroll();
            }
            KeyCode::Char(' ') => {
                if let Some(torrent) = self.get_current_selected_torrent() {
                    let hash = torrent.hash.clone();
                    log_debug(
                        &format!(
                            "Torrent state: '{}', name: '{}'",
                            torrent.state, torrent.name
                        ),
                        &self.config.get_timezone(),
                    );
                    match torrent.state.as_str() {
                        "pausedDL" | "pausedUP" | "stoppedDL" | "stoppedUP" => {
                            log_debug("Attempting to resume torrent", &self.config.get_timezone());
                            self.client
                                .resume_torrent(&hash, &self.config.get_timezone())
                                .await?;
                        }
                        _ => {
                            log_debug("Attempting to pause torrent", &self.config.get_timezone());
                            self.client
                                .pause_torrent(&hash, &self.config.get_timezone())
                                .await?;
                        }
                    }
                    self.refresh_data().await?;
                }
            }
            KeyCode::Delete | KeyCode::Char('d') => {
                if let Some(torrent) = self.get_current_selected_torrent() {
                    self.delete_confirmation_hash = Some(torrent.hash.clone());
                    self.state = AppState::ConfirmDelete;
                }
            }
            KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.state = AppState::Search;
                self.input_mode = InputMode::Search;
                self.search_input.clear();
                self.filtered_torrents.clear();
                self.is_searching = false;
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_add_torrent_input(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Enter => {
                let path = self.torrent_path_input.clone();
                if !path.is_empty() {
                    match std::fs::read(path) {
                        Ok(data) => {
                            if let Err(e) = self.client.add_torrent(&data, None).await {
                                self.error_message = Some(format!("Failed to add torrent: {}", e));
                                self.state =
                                    AppState::Error(format!("Failed to add torrent: {}", e));
                            } else {
                                self.state = AppState::Main;
                                self.input_mode = InputMode::None;
                                self.refresh_data().await?;
                            }
                        }
                        Err(e) => {
                            self.error_message = Some(format!("Failed to read file: {}", e));
                            self.state = AppState::Error(format!("Failed to read file: {}", e));
                        }
                    }
                }
            }
            KeyCode::Esc => {
                self.state = AppState::Main;
                self.input_mode = InputMode::None;
            }
            _ => match key.code {
                KeyCode::Char(c) => {
                    self.torrent_path_input.push(c);
                }
                KeyCode::Backspace => {
                    self.torrent_path_input.pop();
                }
                _ => {}
            },
        }
        Ok(())
    }

    async fn handle_search_input(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Enter | KeyCode::Esc => {
                self.state = AppState::Main;
                self.input_mode = InputMode::None;
                if key.code == KeyCode::Esc {
                    // Clear search when canceling
                    self.search_input.clear();
                    self.is_searching = false;
                    self.filtered_torrents.clear();
                }
            }
            KeyCode::Char(c) => {
                self.search_input.push(c);
                self.filter_torrents();
            }
            KeyCode::Backspace => {
                self.search_input.pop();
                self.filter_torrents();
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_confirm_delete_input(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                if let Some(hash) = &self.delete_confirmation_hash {
                    let delete_files = key.modifiers.contains(KeyModifiers::SHIFT);
                    if let Err(e) = self.client.delete_torrent(hash, delete_files).await {
                        self.error_message = Some(format!("Failed to delete torrent: {}", e));
                        self.state = AppState::Error(format!("Failed to delete torrent: {}", e));
                    } else {
                        self.state = AppState::Main;
                        self.delete_confirmation_hash = None;
                        self.refresh_data().await?;
                    }
                }
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.state = AppState::Main;
                self.delete_confirmation_hash = None;
            }
            _ => {}
        }
        Ok(())
    }

    async fn attempt_login(&mut self) -> Result<()> {
        match self
            .client
            .login(&self.username_input, &self.password_input)
            .await
        {
            Ok(()) => {
                // Save successful connection info to config
                let current_url = self.client.get_base_url().to_string();
                if let Err(e) = self
                    .config
                    .update_connection_info(&current_url, &self.username_input)
                {
                    log_debug(
                        &format!("Failed to save config: {}", e),
                        &self.config.get_timezone(),
                    );
                } else {
                    log_debug(
                        "Successfully saved connection info to config",
                        &self.config.get_timezone(),
                    );
                }

                self.state = AppState::Main;
                self.input_mode = InputMode::None;
                self.refresh_data().await?;
            }
            Err(e) => {
                self.error_message = Some(format!("Login failed: {}", e));
                self.state = AppState::Error(format!("Login failed: {}", e));
            }
        }
        Ok(())
    }

    async fn refresh_data(&mut self) -> Result<()> {
        match self.client.get_torrents().await {
            Ok(torrents) => {
                self.torrents = torrents;
                if self.selected_torrent >= self.torrents.len() && !self.torrents.is_empty() {
                    self.selected_torrent = self.torrents.len() - 1;
                }
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to fetch torrents: {}", e));
            }
        }

        match self.client.get_server_state().await {
            Ok(state) => {
                self.server_state = Some(state);
            }
            Err(e) => {
                // Don't show error for server state as it's not critical
                eprintln!("Failed to fetch server state: {}", e);
            }
        }

        self.last_update = Instant::now();
        Ok(())
    }

    fn adjust_scroll(&mut self) {
        // Calculate visible rows dynamically - will be set by UI
        let visible_rows = self.get_max_visible_rows();
        let torrent_count = self.get_current_torrent_list_len();

        if torrent_count == 0 {
            self.scroll_offset = 0;
            self.selected_torrent = 0;
            return;
        }

        // Ensure selected_torrent is within bounds
        self.selected_torrent = self.selected_torrent.min(torrent_count.saturating_sub(1));

        if self.selected_torrent < self.scroll_offset {
            self.scroll_offset = self.selected_torrent;
        } else if self.selected_torrent >= self.scroll_offset + visible_rows {
            self.scroll_offset = self.selected_torrent - visible_rows + 1;
        }
    }

    pub fn get_visible_torrents(&self) -> &[Torrent] {
        let visible_rows = self.get_max_visible_rows();
        let start = self.scroll_offset;

        let torrents = if self.is_searching && !self.filtered_torrents.is_empty() {
            &self.filtered_torrents
        } else {
            &self.torrents
        };

        let end = (start + visible_rows).min(torrents.len());
        &torrents[start..end]
    }

    pub fn get_relative_selected_index(&self) -> usize {
        self.selected_torrent - self.scroll_offset
    }

    pub fn get_max_visible_rows(&self) -> usize {
        // Use the stored value, with a minimum of 1
        self.max_visible_rows.max(1)
    }

    pub fn set_max_visible_rows(&mut self, rows: usize) {
        // This will be called by the UI to set the actual available rows
        self.max_visible_rows = rows;
    }

    fn filter_torrents(&mut self) {
        if self.search_input.is_empty() {
            self.filtered_torrents.clear();
            self.is_searching = false;
        } else {
            let query = self.search_input.to_lowercase();
            self.filtered_torrents = self
                .torrents
                .iter()
                .filter(|torrent| {
                    torrent.name.to_lowercase().contains(&query)
                        || torrent.state.to_lowercase().contains(&query)
                })
                .cloned()
                .collect();
            self.is_searching = true;
        }

        // Reset selection and scroll when filtering
        self.selected_torrent = 0;
        self.scroll_offset = 0;
    }

    pub fn get_current_torrent_list_len(&self) -> usize {
        if self.is_searching && !self.filtered_torrents.is_empty() {
            self.filtered_torrents.len()
        } else {
            self.torrents.len()
        }
    }

    pub fn get_current_selected_torrent(&self) -> Option<&Torrent> {
        let torrents = if self.is_searching && !self.filtered_torrents.is_empty() {
            &self.filtered_torrents
        } else {
            &self.torrents
        };

        torrents.get(self.selected_torrent)
    }
}
