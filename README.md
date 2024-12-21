# Raquet - Terminal HTTP Client

> **Note**: This project is currently under heavy development and not ready for production use. Features may be incomplete or subject to change.

Raquet is a terminal-based HTTP client with a text user interface (TUI) built in Rust. It provides a user-friendly way to send HTTP requests, manage collections, and view responses, all from within your terminal.

## Development Status

- 🚧 Active development
- ⚠️ Breaking changes may occur
- 📝 Documentation is work in progress
- 🧪 Testing and stability improvements ongoing
- 🤝 Contributions are welcome

## Features

### Request Management
- Support for common HTTP methods (GET, POST, PUT, DELETE, PATCH)
- Custom header management with enable/disable toggles
- Request body editor
- Dynamic headers (Content-Length, Host, Random-Token)
- URL input with cursor navigation

### Collections
- Save and organize requests in collections
- Create new collections with names and descriptions
- Browse and load saved requests
- Hierarchical organization with folders
- Quick access to recent collections

### Response Handling
- Formatted response display with syntax highlighting
- Response metadata (status, time, size)
- Response headers view
- Scrollable response body
- History tracking of all requests

### User Interface
- Intuitive TUI with keyboard navigation
- Tab-based field navigation
- Vim-style key bindings
- Color-coded method indicators
- Visual feedback for active elements
- Scrollable views for long content

### Configuration
- Customizable default headers
- Configurable timeout settings
- Adjustable response size limits
- History size configuration
- Default URL setting

## Installation

### Prerequisites
- Rust toolchain (1.70 or later)
- Cargo package manager

### Building from Source 
```bash
git clone https://github.com/yourusername/raquet.git
cd raquet
cargo build --release
```


The binary will be available at `target/release/raquet`

## Usage

### Basic Navigation
- `Tab`: Move between fields
- `Shift+Tab`: Move backward between fields
- `Enter`: Activate/edit selected field
- `Esc`: Exit editing mode/close popups
- `↑/↓`: Navigate lists and select options

### Making Requests
1. Select HTTP method using arrow keys
2. Enter URL in the URL field
3. Add/edit headers in the Headers section
4. Add request body if needed
5. Press "Go" button or use keyboard shortcut

### Managing Headers
- Enter: Start editing headers
- Space: Toggle header on/off
- Tab: Switch between key and value when editing
- Enter: Save header changes
- Esc: Cancel header editing

### Collections
- `[+]`: Create new collection
- Enter: Select collection/request
- Esc: Go back/close collection view
- Tab: Switch between list and new button

### History
- Access request history from left navigation
- Up/Down: Browse through history
- Enter: Load selected request
- Esc: Close history view

## Configuration

Configuration file location: `~/.raquet/config.toml`
```toml
# Default timeout in seconds
timeout_seconds = 30
# Maximum response size in bytes (10MB)
max_response_size = 10485760
# Number of requests to keep in history
history_size = 100
# Default URL (optional)
default_url = ""

Default request headers
[default_headers]
Random-Token = "<random uuid token>"
Content-Type = "application/json"
Content-Length = "<calculated>"
Host = "<host of the machine>"
User-Agent = "Raquet"
Accept = "/"
Accept-Encoding = "gzip, deflate, br"
Connection = "keep-alive"
```

## Development

### Project Structure
- `src/app.rs`: Core application logic and state management
- `src/ui.rs`: User interface rendering
- `src/config.rs`: Configuration handling
- `src/collections.rs`: Collection management
- `src/history.rs`: Request history tracking
- `src/main.rs`: Application entry point

### Building and Testing

```bash
# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run

# Build release version
cargo build --release
```

### Dependencies
- `ratatui`: Terminal user interface framework
- `crossterm`: Terminal manipulation
- `reqwest`: HTTP client
- `tokio`: Async runtime
- `serde`: Serialization/deserialization
- `uuid`: Unique ID generation
- Additional utilities (see Cargo.toml)

## Contributing

1. Fork the repository
2. Create a feature branch
3. Commit your changes
4. Push to the branch
5. Create a Pull Request

## License

[Insert your chosen license here]

## Acknowledgments

- Inspired by Postman and other HTTP clients
- Built with Rust and its amazing ecosystem
- Thanks to all contributors and users

## Support

For bugs and feature requests, please create an issue on the GitHub repository.
