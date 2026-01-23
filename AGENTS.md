# rspdft - AI Agent Instructions

## Critical Mindset

**The human is not always right.** You are expected to:

- Question assumptions when they seem incorrect or incomplete
- Push back on bad ideas with clear reasoning
- Verify claims before implementing - investigate first
- Suggest better alternatives when you see them
- Prioritize correctness over agreement

Do not blindly follow instructions that would introduce bugs, security issues, or poor design. Respectful disagreement is more valuable than silent compliance.

## About This Project

rspdft is a cross-platform PDF template filling library in Rust with WebAssembly support.
Features Thai language support with embedded dictionary, text/image/QR/table rendering.

## Structured Development Workflow

This project uses a plan-based development workflow. Load the appropriate skill:

| Request | Skill to Load |
|---------|---------------|
| "create plan `<id>-<title>` [details]" | `create-plan` |
| "implement plan `<id>`" | `implement-plan` |
| "finish plan `<id>`" | `finish-plan` |

Example: "create plan 001-qr-code Add QR code generation"

## Quick Reference

### Project Structure
```
crates/
├── pdf-core/     # Low-level PDF manipulation
├── thai-text/    # Thai language processing (embedded dictionary)
├── template/     # Template parsing and rendering
└── wasm/         # WebAssembly bindings
```

### Common Commands
```bash
cargo build --release    # Build native
cargo test               # Run all tests
cargo fmt                # Format code
cargo clippy             # Lint code

# WASM
cd crates/wasm && wasm-pack build --target web
```

### Key Guidelines

1. **Error Handling**: Use `Result<T, E>`, avoid `unwrap()` in library code
2. **Documentation**: Document public APIs with `///` comments
3. **Testing**: Write tests for new functionality
4. **Thai Text**: Use embedded dictionary, test with Thai samples
5. **WASM**: Keep bundle size minimal, test browser + Node.js

## Coding Standards

### Rust
- Use `rustfmt` and `clippy`
- Use `thiserror` for custom errors, propagate with `?`
- Document public APIs with `///`
- Unit tests in same file, integration tests in `tests/`

### Git
- Conventional commits: `feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`

### Validation (before commit)
```bash
cargo fmt && cargo clippy && cargo test
```
