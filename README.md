# nmrs

![Vibe Coding](https://img.shields.io/badge/vibe-coding-blue?style=for-the-badge&logo=sparkles&logoColor=white)
![Educational](https://img.shields.io/badge/educational-purpose-green?style=for-the-badge&logo=graduation-cap&logoColor=white)

A fast and interactive CLI tool for managing node_modules directories - Rust port of the Node.js nm tool.

> **Note**: This repository is for educational purposes - a learning project to learn Rust with the help of vibe coding. Based on the original [nm tool](https://github.com/bluenex/nm).

## Features

- **Fast scanning**: Uses parallel processing to quickly find and analyze node_modules directories
- **Interactive removal**: Select multiple directories for removal with checkboxes
- **Smart caching**: 10-minute cache to avoid re-scanning recently analyzed directories
- **Progress indicators**: Real-time feedback during scanning and size calculation
- **Cross-platform**: Works on macOS, Linux, and Windows
- **Size calculation**: Matches `du -sh` output exactly

## Installation

```bash
cargo install --path .
```

## Usage

### List node_modules directories

```bash
nmrs ls <path>
```

### Remove node_modules directories interactively

```bash
nmrs rm <path>
```

### Clear cached results

```bash
nmrs clear-cache
```

## Commands

- `nmrs ls <path>` - Scans directory for node_modules, shows sizes, caches results
- `nmrs rm <path>` - Interactive removal with checkboxes, uses cached results when available
- `nmrs clear-cache` - Clears cached scan results

## Performance

- **Target**: 15-18s scan time (vs 23s for original Node.js version)
- **Optimization**: Parallel directory scanning and size calculation
- **Caching**: Instant results for repeat operations within 10 minutes

## Architecture

```txt
src/
├── main.rs           # CLI entry point
├── commands/         # Command implementations
│   ├── ls.rs         # List command
│   ├── rm.rs         # Remove command
│   └── cache.rs      # Clear cache command
├── scanner/          # Directory scanning
│   └── finder.rs     # Core scanning logic
├── cache/            # Caching system
│   └── storage.rs    # Cache storage
└── utils/            # Utilities
    └── format.rs     # Size formatting
```

## Dependencies

- **clap**: CLI argument parsing
- **walkdir**: Directory traversal
- **rayon**: Parallel processing
- **serde**: Serialization for caching
- **dirs**: Cross-platform cache directory
- **indicatif**: Progress indicators
- **inquire**: Interactive prompts
- **tokio**: Async runtime

## License

MIT
