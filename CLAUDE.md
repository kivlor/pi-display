# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A Rust terminal UI application using ratatui for rendering and crossterm for terminal event handling. Intended for Raspberry Pi display use.

## Build Commands

```bash
cargo build          # Build the project
cargo build --release # Build optimized release binary
cargo run            # Build and run
cargo test           # Run tests
cargo clippy         # Run linter
cargo fmt            # Format code
```

## Architecture

- **src/main.rs**: Entry point using ratatui's `run()` helper with a render loop that handles keyboard input to exit
- Uses Rust 2024 edition
- Dependencies: ratatui (TUI framework), crossterm (terminal backend)
