<!-- BEGIN:nextjs-agent-rules -->

# This is NOT the Next.js you know

This version has breaking changes — APIs, conventions, and file structure may all differ from your training data. Read the relevant guide in `node_modules/next/dist/docs/` (resolved from this file's directory; in monorepos the `next` package may not be visible from the repo root) before writing any code. Heed deprecation notices.

This block is written and re-added by `next dev` — verify at `node_modules/next/dist/server/lib/generate-agent-files.js`. Removing it from a diff only re-creates the uncommitted change; committing it with your work keeps the tree clean.

<!-- END:nextjs-agent-rules -->

# Herdlink Frontend

The frontend provides graph exploration, conversations, communities, and surveys.
See `README.md` and the repository README for local setup.

## Stack

Next.js 16 App Router, React 19, TypeScript, Tailwind CSS v4, shadcn/ui,
Lucide React, and Sigma.js. Use Bun and the committed `bun.lock`.

## Commands

Run these from the repository root:

- `just frontend-install` — install locked dependencies.
- `just frontend` — start the frontend on port 3001.
- `just frontend-check` — run lint, type checking, frontend tests, and build.
- `just frontend-build` — create a production build.

## Code and design

Use TypeScript strict mode, named component exports, PascalCase components,
camelCase utilities, and two-space indentation. Preserve the existing Herdlink
layout and light/dark themes. Use real API data and keep interactions responsive
and accessible.

Routes live in `src/app/`, components in `src/components/`, and API helpers and
shared types in `src/lib/`. Static assets live in `public/`.
