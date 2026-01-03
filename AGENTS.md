# AGENTS.md

## Build/Test Commands

- `cargo build` - Build the project
- `cargo test` - Run all tests
- `cargo test <test_name>` - Run single test (e.g., `cargo test test_string_matcher_contains_behavior`)
- `cargo test -- <test_name>` - Alternative single test syntax
- `cargo fmt` - Format code
- `cargo clippy` - Lint code

## Code Style Guidelines

### Logging

- Add logging where appropriate
- Use the macros: `debug`, `err`, `warn`, `info`, `trace` and keep in mind where
  does a specific logging level make sense.
- Warn and err are self-explanatory
- Info should be used for general user information
- Debug and trace should be used for hunting bugs

### Imports & Dependencies

- Use `anyhow::Result<T>` for error handling with `anyhow::Context` for error context
- Import external crates first, then local modules
- Group imports by type (std, external, local)

### Naming Conventions

- Use `snake_case` for functions and variables
- Use `PascalCase` for types and structs
- Use `SCREAMING_SNAKE_CASE` for constants
- Private fields use `snake_case`

### Error Handling

- Use `anyhow::Result<T>` return types
- Add context with `.with_context(|| "description")`
- Use `?` operator for propagation
- Log warnings with `log::warn!` for non-fatal issues
- Log errors with `log::error!` for fatal issues

### Code Structure

- Module-level docs at top of files
- Public structs have `#[derive(Debug)]`
- Use `#[cfg(test)]` for test modules
- Tests use `#[test_log::test]` and `#[tokio::test]` where appropriate
- If `tokio::test` is not needed, use `test_log::test` for new tests

## Test writing

- When asked to write tests, first plan out what edge cases you will handle.
- Write unit tests.
- Reference the `test_helpers` modules to see how to mock inboxes.
- When asked to write tests, it is okay if they fail. Maybe the problem is in the implementation. You were asked to write the tests to discover bugs.
- Do NOT change the implementation unless EXPLICITLY told to do so when writing tests.
- When testing the new tests, do not run all the tests but just run the tests you wrote like `cargo test string_matcher`.

## Important files

- `DSL.md`: the specification of the DSL used for configuring the program. Read when writing tests for lexer/parser/resolver/etc.
- `src/inbox.rs`: main Inbox trait that is used for manipulating emails
