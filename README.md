# Package Manager CLI Tool
This documentation provides instructions on using our package manager CLI tool. The tool allows you to manage software packages by executing commands from a list of available scripts.

### Usage
#### Executing Commands
- Use arrow keys (←, →) to navigate through available commands in the list.
- Press 'q' to quit without executing any command.
- Press 'w' to move up, pressing enter on the same line to execute.
- Press 's' to move down, pressing enter to execute.


### System Compatibility
OS: Cross-platform (tested on Linux and windows)
Rust Version: Requires Rust 1.35+


## Build from source:

- Ubuntu(Debian): ```cargo deb```
- Windows: ```cargo build --release --target x86_64-pc-windows-gnu```

## Dependencies
This tool depends on:

- serde for JSON parsing.

If you encounter any issues or have suggestions, please open an issue in our repository!