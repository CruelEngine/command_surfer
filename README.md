# Package Manager CLI Tool
This documentation provides instructions on using our package manager CLI tool. The tool allows you to manage software packages by executing commands from a list of available scripts.

A CLI tool that reads `package.json` files in Node.js projects and displays available scripts in an interactive menu. It automatically detects the package manager (npm, yarn, or pnpm) and executes the selected script with the appropriate command.

## Features

- Interactive TUI (Text User Interface) for selecting and executing npm/yarn/pnpm scripts
- Automatic package manager detection (npm, yarn, pnpm)
- Script filtering functionality
- Cross-platform support (Linux, Windows, macOS)
- Keyboard navigation

## Installation

### From Source

1. Clone the repository
2. Build using Cargo:

```bash
# For Debian/Ubuntu (creates a .deb package)
cargo deb

# For Windows
cargo build --release --target x86_64-pc-windows-gnu

# For macOS (Intel)
cargo build --release --target x86_64-apple-darwin

# For macOS (Apple Silicon/ARM)
cargo build --release --target aarch64-apple-darwin
```

## Usage

Run the executable in the root directory of any Node.js project containing a `package.json` file:

```bash
node_script_list
```

### Navigation Controls

- **w**: Move up in the script list
- **s**: Move down in the script list
- **f**: Enter filter mode (type to filter scripts by name)
- **Enter**: Execute the selected script
- **q**: Quit without executing any script
- **Esc**: Exit filter mode (when in filter mode)

## Dependencies

- [pancurses](https://crates.io/crates/pancurses): Terminal UI library
- [serde](https://crates.io/crates/serde): JSON serialization/deserialization
- [serde_json](https://crates.io/crates/serde_json): JSON parsing

## System Requirements

- Rust 1.35+
- Compatible with Linux and Windows

## License

MIT

## Author

CruelEngine