# AGENTS.md

## Project

`dbettier` is a Rust desktop database manager inspired by DataGrip. It succeeds the original Go TUI, with PostgreSQL as the initial database target.

Use [GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui) for all GUI work. GPUI is pre-1.0, so verify unfamiliar APIs against `Cargo.lock`, the [GPUI examples](https://github.com/zed-industries/zed/tree/main/crates/gpui/examples), and the [Zed source](https://github.com/zed-industries/zed).

## Rules

- Prioritize correctness and clarity.
- Avoid `unwrap`, unchecked indexing, and other operations that may panic.
- Propagate or handle errors. Never silently discard a fallible result with `let _ =`.
- Ensure asynchronous failures reach the UI with meaningful context.
- Use full words for names and comment only to explain non-obvious reasons.

## GPUI

- Keep shared UI state in GPUI entities. Use `Render` for views and `RenderOnce` for components created only as elements.
- Name context parameters `cx` and window parameters `window`. Pass `window` before `cx`.
- Inside entity update closures, use the inner `cx`. Never update an entity while it is already being updated.
- Call `cx.notify()` after state changes that affect rendering.
- Use actions and key contexts for commands. Use `cx.listener` when handlers update the current entity.
- Entity access and rendering run on the foreground thread. Use `cx.background_spawn` for database queries and other blocking work, then update state on the foreground thread.
- A dropped GPUI `Task` is cancelled. Await it, detach it intentionally, or store it.
- Keep rendering free of database and filesystem I/O.
- Virtualize large trees, tables, and query results.

## Validation

```bash
cargo fmt --check
cargo check
cargo test
cargo clippy --all-targets --all-features
```

In GPUI tests, use GPUI executor timers instead of `smol::Timer` when driving `run_until_parked()`.
