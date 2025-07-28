# qBittUI

A modern Terminal User Interface (TUI) wrapper for qBittorrent WebUI, built with Rust.

![qBittUI Screenshot](https://via.placeholder.com/800x400/2d3748/white?text=qBittUI+Terminal+Interface)

## Features

- 🖥️ **Terminal-based interface** - Clean, responsive TUI that works in any terminal
- 🔗 **qBittorrent WebUI integration** - Connects to existing qBittorrent WebUI
- 📊 **Real-time monitoring** - Live updates of download/upload speeds and progress
- 🔍 **Search functionality** - Quickly find torrents with Ctrl+F
- ⏸️ **Torrent management** - Pause, resume, and delete torrents
- 📁 **Add torrents** - Support for adding new torrents via file path
- 💾 **Configuration persistence** - Saves connection settings automatically
- 🎨 **Color-coded states** - Visual indicators for different torrent states
- 📐 **Responsive design** - Adapts to different terminal sizes
- 🔐 **Secure authentication** - Handles login with username/password

## Installation

### Prerequisites

- Rust 1.70+ (rustc with 2024 edition support)
- qBittorrent with WebUI enabled

### From Source

```bash
git clone https://github.com/PedroBuffon/qbittui.git
cd qbittui
cargo build --release
```

The binary will be available at `target/release/qbittui`.

### Install with Cargo

```bash
cargo install --path .
```

## Usage

### Basic Usage

```bash
./qbittui
```

On first run, you'll be prompted to:

1. Enter your qBittorrent WebUI URL (e.g., `http://localhost:8080`)
2. Provide your username and password

The application will save your URL and username for future sessions.

### Command Line Options

```bash
qbittui --help
```

### Keyboard Shortcuts

#### Navigation

- `↑/↓` or `j/k` - Navigate through torrent list
- `Page Up/Page Down` - Navigate by page
- `Home/End` - Jump to first/last torrent

#### Actions

- `Space` - Pause/Resume selected torrent
- `d` or `Delete` - Delete selected torrent
- `a` - Add new torrent
- `r` - Refresh torrent list
- `Ctrl+F` - Search torrents
- `Esc` - Cancel current action/search
- `Ctrl+Q` - Quit application

#### Login Screen

- `Tab` - Switch between username and password fields
- `Ctrl+H` - Show/hide password
- `Enter` - Login

## Configuration

qBittUI automatically creates a configuration file (`qbittui_config.json`) that stores:

- Last used WebUI URL
- Username (passwords are never saved for security)

Example configuration:

```json
{
  "url": "http://localhost:8080",
  "username": "admin"
}
```

## Torrent States

The interface uses color coding for different torrent states:

- 🟢 **Green** - Downloading
- 🔵 **Blue** - Uploading/Stalled Upload
- 🟡 **Yellow** - Paused
- 🔴 **Red** - Error
- 🔵 **Cyan** - Queued
- ⚪ **White** - Other states

## Requirements

### Terminal Requirements

- Minimum terminal size: 80x24 characters
- Unicode support recommended
- True color support for best experience

### qBittorrent Setup

1. Enable WebUI in qBittorrent preferences
2. Note the WebUI port (default: 8080)
3. Ensure authentication is configured if required

## Troubleshooting

### Connection Issues

- Verify qBittorrent WebUI is accessible at the specified URL
- Check firewall settings
- Ensure correct username/password

### Terminal Display Issues

- Increase terminal size if you see a size warning
- Enable UTF-8 encoding in your terminal
- Try different terminal emulators if rendering issues persist

### Debug Logging

qBittUI creates debug logs in `qbittui_debug.log` for troubleshooting connection and API issues.

## Development

### Building from Source

```bash
git clone https://github.com/PedroBuffon/qbittui.git
cd qbittui
cargo build
```

### Running Tests

```bash
cargo test
```

### Development Dependencies

- `tokio` - Async runtime
- `reqwest` - HTTP client for API communication
- `ratatui` - Terminal UI framework
- `crossterm` - Cross-platform terminal manipulation
- `serde` - JSON serialization
- `anyhow` - Error handling

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

### Development Setup

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [ratatui](https://github.com/ratatui-org/ratatui) - Rust TUI library
- Inspired by the need for a lightweight qBittorrent interface
- Thanks to the qBittorrent team for the excellent WebUI API

## Support

If you encounter any issues or have questions:

1. Check the [troubleshooting section](#troubleshooting)
2. Look at existing [GitHub issues](https://github.com/PedroBuffon/qbittui/issues)
3. Create a new issue with detailed information about your problem

---

Made with ❤️ in Rust
