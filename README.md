# dbettier

A desktop database management GUI built with Rust and [GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui). The project aims to provide a fast, keyboard-friendly database workflow inspired by tools such as DataGrip.

This is the GUI successor to the original [dbettier terminal application](https://github.com/SavingFrame/dbettier).

## Status

This project is in early development. The current application is a GPUI prototype, and the database management features below are planned.

## Goals

- Manage database connections
- Browse databases, schemas, tables, views, and other objects
- Inspect and edit table data
- Write and execute SQL with syntax highlighting
- Display query results in a fast, scrollable data grid
- Support multiple editor and result tabs
- Provide keyboard shortcuts for common workflows
- Show query progress, errors, and connection status

PostgreSQL is the initial database target, following the original TUI project.

## Tech Stack

- [Rust](https://www.rust-lang.org/)
- [GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui), Zed's GPU-accelerated UI framework
- `gpui_platform` with Wayland and X11 support on Linux

GPUI is pre-1.0 and changes frequently. This project currently follows GPUI directly from the Zed repository.

## Development

Install the latest stable Rust toolchain, then run:

```bash
cargo run
```

Check the project with:

```bash
cargo fmt --check
cargo check
cargo test
cargo clippy --all-targets --all-features
```

See the [GPUI README](https://github.com/zed-industries/zed/blob/main/crates/gpui/README.md) for framework details and platform dependencies.

## License

A license has not been added yet.
