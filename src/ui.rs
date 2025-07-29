use crate::app::{App, AppState, InputMode};
use humansize::{BINARY, format_size};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

pub fn draw(f: &mut Frame, app: &mut App) {
    let size = f.area();

    // Update app with current terminal size
    app.terminal_width = size.width;
    app.terminal_height = size.height;

    // Check minimum terminal size
    if size.width < 80 || size.height < 24 {
        let warning = Paragraph::new(vec![
            Line::from("Terminal too small!"),
            Line::from(""),
            Line::from("Minimum size required:"),
            Line::from("Width: 80 characters"),
            Line::from("Height: 24 lines"),
            Line::from(""),
            Line::from(format!("Current: {}x{}", size.width, size.height)),
            Line::from(""),
            Line::from("Please resize your terminal and try again."),
            Line::from("Press Ctrl+Q to quit."),
        ])
        .style(Style::default().fg(Color::Red))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .title("Terminal Size Warning")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Red)),
        );

        let warning_area = centered_rect_percent(50, 12, size);
        f.render_widget(Clear, warning_area);
        f.render_widget(warning, warning_area);
        return;
    }
    match app.state {
        AppState::UrlConfig => draw_url_config(f, app),
        AppState::Login => draw_login(f, app),
        AppState::Main => draw_main(f, app),
        AppState::AddTorrent => draw_add_torrent(f, app),
        AppState::Search => draw_search(f, app),
        AppState::ConfirmDelete => draw_confirm_delete(f, app),
        AppState::Error(ref message) => draw_error(f, message),
    }
}

fn draw_url_config(f: &mut Frame, app: &App) {
    let size = f.area();

    // Clear the entire screen first
    f.render_widget(Clear, size);

    // Create a responsive centered configuration form
    let popup_width = (size.width * 80 / 100).clamp(50, 80); // 80% of width, but between 50-80 chars
    let popup_height = (size.height * 40 / 100).clamp(8, 12); // 40% of height, but between 8-12 lines

    let popup_area = centered_rect(popup_width, popup_height, size);

    let block = Block::default()
        .title(" qBittorrent WebUI Configuration ")
        .title_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    f.render_widget(block, popup_area);

    let inner = popup_area.inner(Margin {
        vertical: 3,
        horizontal: 2,
    });

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Description
            Constraint::Length(3), // URL input
            Constraint::Min(1),    // Instructions (flexible)
        ])
        .split(inner);

    // Description
    let description_text = if let Some(last_url) = app.config.get_last_url() {
        format!("Enter qBittorrent WebUI URL (Last used: {last_url})")
    } else {
        "Enter the qBittorrent WebUI URL:".to_string()
    };

    let description = Paragraph::new(description_text)
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Left);
    f.render_widget(description, chunks[0]);

    // URL input field
    let url_block = Block::default()
        .title(" WebUI URL (Active) ")
        .title_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    // Make sure the text is visible with good contrast
    let url_display = if app.url_input.is_empty() {
        // Show placeholder in gray
        Paragraph::new("http://localhost:8080").block(url_block.clone())
    } else {
        // Show actual input in white
        Paragraph::new(app.url_input.clone()).block(url_block.clone())
    };

    f.render_widget(url_display, chunks[1]);

    // Instructions
    let instructions = Paragraph::new(vec![Line::from(
        "Enter: Continue to login | Esc: Quit | Ctrl+Q: Force quit",
    )])
    .style(Style::default().fg(Color::Gray))
    .alignment(Alignment::Center)
    .wrap(Wrap { trim: true });
    f.render_widget(instructions, chunks[2]);

    // Cursor positioning - use actual input length, not display length
    let cursor_x = if app.url_input.is_empty() {
        chunks[1].x + 1 // Start of field if empty
    } else {
        chunks[1].x + app.url_input.len() as u16 + 1 // End of actual input
    };

    f.set_cursor_position((cursor_x, chunks[1].y + 1));
}

fn draw_login(f: &mut Frame, app: &App) {
    let size = f.area();

    // Clear the entire screen first
    f.render_widget(Clear, size);

    // Create a responsive centered login form
    let popup_width = (size.width * 70 / 100).clamp(40, 70); // 70% of width, but between 40-70 chars
    let popup_height = (size.height * 50 / 100).clamp(9, 12); // 50% of height, but between 9-12 lines

    let popup_area = centered_rect(popup_width, popup_height, size);

    let block = Block::default()
        .title(" qBittorrent Login ")
        .title_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        // Outside border
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    f.render_widget(block, popup_area);

    let inner = popup_area.inner(Margin {
        vertical: 1,
        horizontal: 2,
    });

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Username
            Constraint::Length(3), // Password
            Constraint::Min(2),    // Instructions (flexible)
        ])
        .split(inner);

    // Username field
    let username_title = if app.config.get_last_username().is_some()
        && app.config.get_last_username().as_ref() == Some(&app.username_input)
    {
        "Username (Last used)"
    } else {
        "Username"
    };

    let username_block = Block::default()
        .title(username_title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let username_text = format!("{} ", app.username_input); // Add space for cursor
    let username_paragraph = Paragraph::new(username_text).block(username_block);
    f.render_widget(username_paragraph, chunks[0]);

    // Password field
    let password_block = Block::default()
        .title("Password")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let password_display = if app.show_password {
        format!("{} ", app.password_input)
    } else {
        format!("{} ", "●".repeat(app.password_input.len()))
    };

    let password_paragraph = Paragraph::new(password_display).block(password_block);
    f.render_widget(password_paragraph, chunks[1]);

    // Instructions
    let instructions = Paragraph::new(
        "Tab: Switch | Enter: Login | Esc: Quit | Ctrl+H: Show/Hide | Ctrl+Q: Force quit",
    )
    .style(Style::default().fg(Color::Gray))
    .alignment(Alignment::Center)
    .wrap(Wrap { trim: true });
    f.render_widget(instructions, chunks[2]);

    // Cursor positioning - simpler approach
    match app.input_mode {
        InputMode::Username => {
            f.set_cursor_position((
                chunks[0].x + app.username_input.len() as u16 + 1,
                chunks[0].y + 1,
            ));
        }
        InputMode::Password => {
            f.set_cursor_position((
                chunks[1].x + app.password_input.len() as u16 + 1,
                chunks[1].y + 1,
            ));
        }
        _ => {}
    }
}

fn draw_main(f: &mut Frame, app: &mut App) {
    let size = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(size);

    // Header with server info
    draw_header(f, chunks[0], app);

    // Torrent list
    draw_torrent_list(f, chunks[1], app);

    // Footer with controls
    draw_footer(f, chunks[2]);
}

fn draw_header(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .title("qBittorrent TUI")
        .borders(Borders::ALL);

    let inner = block.inner(area);
    f.render_widget(block, area);

    if let Some(state) = &app.server_state {
        let info_text = vec![Line::from(vec![
            Span::styled("Status: ", Style::default().fg(Color::Cyan)),
            Span::raw(&state.connection_status),
            Span::raw("  |  "),
            Span::styled("Down: ", Style::default().fg(Color::Green)),
            Span::raw(format_size(state.dl_info_speed as u64, BINARY) + "/s"),
            Span::raw("  |  "),
            Span::styled("Up: ", Style::default().fg(Color::Red)),
            Span::raw(format_size(state.up_info_speed as u64, BINARY) + "/s"),
            Span::raw("  |  "),
            Span::styled("Torrents: ", Style::default().fg(Color::Yellow)),
            Span::raw(app.torrents.len().to_string()),
        ])];

        let paragraph = Paragraph::new(info_text).alignment(Alignment::Center);
        f.render_widget(paragraph, inner);
    }
}

fn draw_torrent_list(f: &mut Frame, area: Rect, app: &mut App) {
    let scroll_info = if app.torrents.len() > app.get_max_visible_rows() {
        format!(
            " [{}-{}/{}]",
            app.scroll_offset + 1,
            (app.scroll_offset + app.get_max_visible_rows()).min(app.torrents.len()),
            app.torrents.len()
        )
    } else {
        String::new()
    };

    let block = Block::default()
        .title(format!("Torrents ({}){}", app.torrents.len(), scroll_info))
        .borders(Borders::ALL);

    let inner = block.inner(area);
    f.render_widget(block, area);

    if app.torrents.is_empty() {
        let no_torrents = Paragraph::new("No torrents found\n\nPress 'a' to add a torrent")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        f.render_widget(no_torrents, inner);
        return;
    }

    // Calculate available space for torrents
    let header_height = 2;
    let available_height = inner.height.saturating_sub(header_height);

    // Update app with the actual number of visible rows
    app.set_max_visible_rows(available_height as usize);

    let header_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: header_height,
    };

    let list_area = Rect {
        x: inner.x,
        y: inner.y + header_height,
        width: inner.width,
        height: available_height,
    };

    // Calculate the same widths as used in the data rows
    let available_width = inner.width.saturating_sub(8 + 12 + 12 + 12 + 15 + 8 + 7) as usize; // Progress + Size + Down + Up + State + ETA + spacing
    let name_width = available_width.max(20); // Minimum 20 chars for name, same as in data rows

    // Draw header
    let header_text = vec![
        Line::from(vec![Span::styled(
            format!(
                "{:<width$} {:>8} {:>12} {:>12} {:>12} {:<15} {:>8}",
                "Name",
                "Progress",
                "Size",
                "Down Speed",
                "Up Speed",
                "State",
                "ETA",
                width = name_width
            ),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::raw("─".repeat(inner.width as usize))]),
    ];

    let header_paragraph = Paragraph::new(header_text);
    f.render_widget(header_paragraph, header_area);

    // Create torrent list items
    let visible_torrents = app.get_visible_torrents();
    let items: Vec<ListItem> = visible_torrents
        .iter()
        .map(|torrent| {
            let progress = (torrent.progress * 100.0) as u8;
            let size_str = format_size(torrent.size as u64, BINARY);
            let dl_speed_str = if torrent.dlspeed > 0 {
                format_size(torrent.dlspeed as u64, BINARY) + "/s"
            } else {
                "".to_string()
            };
            let ul_speed_str = if torrent.upspeed > 0 {
                format_size(torrent.upspeed as u64, BINARY) + "/s"
            } else {
                "".to_string()
            };

            // Calculate available width for name (total width - other columns - spacing)
            let available_width =
                inner.width.saturating_sub(8 + 12 + 12 + 12 + 15 + 8 + 7) as usize; // Progress + Size + Down + Up + State + ETA + spacing
            let name_width = available_width.max(20); // Minimum 20 chars for name

            let name = if torrent.name.len() > name_width {
                format!("{}...", &torrent.name[..name_width.saturating_sub(3)])
            } else {
                torrent.name.clone()
            };

            let state_color = match torrent.state.as_str() {
                "downloading" => Color::Green,
                "uploading" | "stalledUP" => Color::Blue,
                "pausedDL" | "pausedUP" => Color::Yellow,
                "error" => Color::Red,
                "queuedDL" | "queuedUP" => Color::Cyan,
                _ => Color::White,
            };

            let line = Line::from(vec![
                Span::raw(format!("{name:<name_width$}")),
                Span::raw(" "),
                Span::styled(format!("{progress:>7}%"), Style::default().fg(Color::Green)),
                Span::raw(" "),
                Span::raw(format!("{size_str:>11}")),
                Span::raw(" "),
                Span::raw(format!("{dl_speed_str:>11}")),
                Span::raw(" "),
                Span::raw(format!("{ul_speed_str:>11}")),
                Span::raw(" "),
                Span::styled(
                    format!("{:<14}", torrent.state),
                    Style::default().fg(state_color),
                ),
                Span::raw(" "),
                Span::styled(
                    format!(
                        "{:>7}",
                        match torrent.state.as_str() {
                            "downloading" | "stalledDL" | "queuedDL" => {
                                torrent.eta.map_or("∞".to_string(), |e| {
                                    if e < 0 {
                                        "∞".to_string()
                                    } else if e == 0 {
                                        "0s".to_string()
                                    } else if e < 60 {
                                        format!("{e}s")
                                    } else if e < 3600 {
                                        format!("{}m", e / 60)
                                    } else if e < 86400 {
                                        format!("{}h{}m", e / 3600, (e % 3600) / 60)
                                    } else {
                                        format!("{}d{}h", e / 86400, (e % 86400) / 3600)
                                    }
                                })
                            }
                            _ => "-".to_string(), // For uploading, stalled upload, completed, etc.
                        }
                    ),
                    Style::default().fg(Color::Magenta),
                ),
            ]);

            ListItem::new(line)
        })
        .collect();

    let selected_style = Style::default()
        .bg(Color::DarkGray)
        .add_modifier(Modifier::BOLD);

    let list = List::new(items)
        .highlight_style(selected_style)
        .highlight_symbol("→ ");

    let mut list_state = ratatui::widgets::ListState::default();
    list_state.select(Some(app.get_relative_selected_index()));

    f.render_stateful_widget(list, list_area, &mut list_state);
}

fn draw_footer(f: &mut Frame, area: Rect) {
    let block = Block::default().title("Controls").borders(Borders::ALL);

    let controls = Paragraph::new(
        "Ctrl+Q: Quit | r: Refresh | ↑↓: Navigate | PgUp/PgDn: Page | Home/End: First/Last | Space: Pause/Resume | Del: Delete | Ctrl+A: Add | Ctrl+F: Search"
    )
    .block(block)
    .style(Style::default().fg(Color::Gray))
    .alignment(Alignment::Center);

    f.render_widget(controls, area);
}

fn draw_add_torrent(f: &mut Frame, app: &App) {
    let size = f.area();
    let popup_area = centered_rect(60, 10, size);

    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .title("Add Torrent")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black));

    f.render_widget(block, popup_area);

    let inner = popup_area.inner(Margin {
        vertical: 1,
        horizontal: 1,
    });

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(2)])
        .split(inner);

    let input_block = Block::default()
        .title("Torrent File Path")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Yellow));

    let input_paragraph = Paragraph::new(app.torrent_path_input.as_str()).block(input_block);
    f.render_widget(input_paragraph, chunks[0]);

    let instructions = Paragraph::new("Enter: Add torrent | Esc: Cancel")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    f.render_widget(instructions, chunks[1]);

    f.set_cursor_position((
        chunks[0].x + app.torrent_path_input.len() as u16 + 1,
        chunks[0].y + 1,
    ));
}

fn draw_confirm_delete(f: &mut Frame, _app: &App) {
    let size = f.area();
    let popup_area = centered_rect(50, 8, size);

    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .title("Confirm Delete")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black).fg(Color::Red));

    f.render_widget(block, popup_area);

    let inner = popup_area.inner(Margin {
        vertical: 1,
        horizontal: 1,
    });

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Length(2)])
        .split(inner);

    let question = Paragraph::new("Are you sure you want to delete this torrent?")
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    f.render_widget(question, chunks[0]);

    let instructions = Paragraph::new("Y: Delete | Shift+Y: Delete with files | N/Esc: Cancel")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    f.render_widget(instructions, chunks[1]);
}

fn draw_error(f: &mut Frame, message: &str) {
    let size = f.area();
    let popup_area = centered_rect(60, 15, size);

    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .title("Error")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black).fg(Color::Red));

    f.render_widget(block, popup_area);

    let inner = popup_area.inner(Margin {
        vertical: 1,
        horizontal: 1,
    });

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)])
        .split(inner);

    let error_text = Paragraph::new(message)
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    f.render_widget(error_text, chunks[0]);

    let instructions = Paragraph::new("Press Enter or Esc to continue")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    f.render_widget(instructions, chunks[1]);
}

fn centered_rect(width: u16, height: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length((r.height.saturating_sub(height)) / 2),
            Constraint::Length(height),
            Constraint::Length((r.height.saturating_sub(height)) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length((r.width.saturating_sub(width)) / 2),
            Constraint::Length(width),
            Constraint::Length((r.width.saturating_sub(width)) / 2),
        ])
        .split(popup_layout[1])[1]
}

// Helper function for percentage-based centering
fn centered_rect_percent(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn draw_search(f: &mut Frame, app: &mut App) {
    // First draw the main torrent list as background
    draw_main(f, app);

    let size = f.area();

    // Calculate search popup dimensions
    let popup_width = size.width.saturating_sub(4).min(60);
    let popup_height = 3;

    // Center the search popup
    let popup_area = Rect {
        x: (size.width.saturating_sub(popup_width)) / 2,
        y: (size.height.saturating_sub(popup_height)) / 2,
        width: popup_width,
        height: popup_height,
    };

    // Clear the area and draw the search input
    f.render_widget(Clear, popup_area);

    let search_title = if app.is_searching {
        format!("Search Torrents ({})", app.filtered_torrents.len())
    } else {
        "Search Torrents".to_string()
    };

    let search_input = Paragraph::new(app.search_input.as_str())
        .style(Style::default().fg(Color::White).bg(Color::Blue))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(search_title)
                .style(Style::default().fg(Color::Yellow)),
        );

    f.render_widget(search_input, popup_area);

    // Position cursor in the search input
    let cursor_x = popup_area.x + app.search_input.len() as u16 + 1;
    let cursor_y = popup_area.y + 1;
    f.set_cursor_position((cursor_x, cursor_y));
}
