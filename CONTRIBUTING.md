# Contributing

## Getting Started

1. Fork the repo and create a branch from `main`.
2. Run `cargo test` to verify everything passes before making changes.
3. Make your changes, add tests for new behavior.
4. Run `cargo test`, `cargo clippy`, and `cargo fmt --check` before submitting.
5. Open a pull request with a clear description of what and why.

## Pull Requests

- Keep PRs focused - one logical change per PR.
- Link any related issues in the PR description.
- All CI checks must pass before merge.
- A maintainer review is required before merge.

## Code Style

- Run `cargo fmt` before committing.
- Fix all `cargo clippy` warnings unless there's a documented reason not to.
- Unsafe code requires a comment explaining why it's sound.

## Commit Messages

Use [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add Tristate::is_undefined helper
fix: correct serde skip logic for None variant
docs: clarify apply_to_option behavior
```

## Reporting Issues

Include a minimal reproducible example. Specify your Rust version (`rustc --version`) and serde version.

## AI Disclosure Policy

AI assistance is permitted in contributions to this project. If any part of your pull request was written or significantly shaped by an AI model, you **must** disclose this in the PR description using the following format:

```
### AI Disclosure

- **Model:** <name and version, e.g. Claude Sonnet 4.6, GPT-4o>
- **Scope:** <what was AI-assisted - e.g. "entire implementation", "test cases only", "docstrings", "refactor of X function">
- **Review:** <how you verified the output - e.g. "manually reviewed line-by-line", "ran full test suite", "reviewed logic but not tests">
```

### Rules

- Omitting disclosure when AI was used is grounds for PR rejection.
- "AI-assisted" includes generated code, generated tests, generated docs, and AI-suggested rewrites you accepted.
- Lightly autocompleted lines (IDE suggestions, Copilot single-liners) do not require disclosure.
- You are fully responsible for the correctness and safety of all code in your PR, regardless of how it was produced.
