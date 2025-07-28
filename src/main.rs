mod api;
mod app;
mod config;
mod ui;
mod event;
mod utils;

use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;
use url::Url;

use app::App;
use event::EventHandler;
use ui::draw;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// qBittorrent WebUI URL
    #[arg(short, long, default_value = "http://localhost:8080")]
    url: String,

    /// Username for authentication
    #[arg(long)]
    username: Option<String>,

    /// Password for authentication
    #[arg(short, long)]
    password: Option<String>,

    /// Set timezone for logs (e.g., UTC, US/Eastern, Europe/London)
    #[arg(long)]
    timezone: Option<String>,

    /// List available timezones
    #[arg(long)]
    list_timezones: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Handle list timezones command
    if args.list_timezones {
        println!("Available timezones:");
        for tz in utils::get_common_timezones() {
            println!("  {}", tz);
        }
        println!("\nYou can use any valid timezone from the IANA Time Zone Database.");
        println!("Example: qbittui --timezone US/Eastern");
        return Ok(());
    }

    // Load config
    let mut config = config::Config::load();

    // Set timezone if provided
    if let Some(timezone) = &args.timezone {
        if utils::is_valid_timezone(timezone) {
            config.set_timezone(timezone)?;
            println!("Timezone set to: {}", timezone);
        } else {
            eprintln!("Invalid timezone: {}. Use --list-timezones to see available options.", timezone);
            return Ok(());
        }
    }

    // Validate URL
    let base_url = Url::parse(&args.url)?;

    // Initialize terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and event handler
    let mut app = App::new_with_config(base_url, args.username, args.password, config).await?;
    let mut event_handler = EventHandler::new();

    // Main loop
    let result = run_app(&mut terminal, &mut app, &mut event_handler).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    event_handler: &mut EventHandler,
) -> Result<()> {
    let mut last_size = terminal.size()?;

    loop {
        // Check for terminal size changes
        let current_size = terminal.size()?;
        if current_size != last_size {
            // Terminal was resized, clear and redraw
            terminal.clear()?;
            last_size = current_size;
            app.handle_resize(current_size.width, current_size.height);
        }

        // Draw UI
        terminal.draw(|f| draw(f, app))?;

        // Handle events
        if let Some(event) = event_handler.next().await {
            // Handle resize events specifically
            if let crossterm::event::Event::Resize(width, height) = event {
                app.handle_resize(width, height);
                terminal.clear()?;
                continue;
            }

            if app.handle_event(event).await? {
                break;
            }
        }
    }
    Ok(())
}
