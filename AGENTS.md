# Agent Instructions

- Any dependency you add must be verified as the latest available version, and specified using only `major.minor` (no patch).
- Do not allow any file to exceed 1000 lines; refactor into modules if needed.
- `mod.rs` files should only contain `mod` declarations, `pub use` statements, module docs (`//!`), and `#[cfg(test)]` test modules.
- Do not preserve old formats or structure for backward compatibility;
- Never use `unwrap` or `expect` in library code; In tests it's allowed.
  - Exception: truly unreachable cases may use `unwrap`/`expect` **only** with a comment directly above explaining why it is unreachable.
- Format the code by running `cargo +nightly fmt --all`
- Do not allow any new clippy warnings or lint relaxations without discussing it with the user first.
- Validate clippy after every Rust code change by running `cargo clippy --all-targets --all-features`
