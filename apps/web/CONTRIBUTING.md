# Contributing to Herdlink

See the [frontend README](README.md) and [project README](../../README.md)
for features and setup. Project instructions live in `AGENTS.md`;
Claude Code reads them through `CLAUDE.md`.

## Development

Use Bun and Node.js 24+. From the repository root:

```sh
just frontend-install
just frontend
just frontend-check
```

The frontend uses the Herdlink backend for graph exploration, conversations,
communities, and surveys. Follow the project README to run the backend locally.

## Pull requests

Create a branch from `main`, keep changes focused, and run `just frontend-check`
before opening a pull request. Describe the resulting behavior and how you
verified it. Report bugs and propose features through the
[Herdlink issues](https://github.com/herdlink/herdlink/issues).
