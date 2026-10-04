# Output plan

- Source: https://developers.openai.com/blog (normalized origin https://developers.openai.com, pathname /blog).
- Application root: /home/tamon/HACKNATION/herdlink/apps/web.
- Site key: developers-openai-com-56163ca2; page key: blog-8caafe43 (SHA-256 prefixes).
- Destination: `/`, replacing only the untouched template `src/app/page.tsx`, as permitted for the first single-page clone.
- Research: docs/research/developers-openai-com-56163ca2/blog-8caafe43/.
- Screenshots: docs/design-references/developers-openai-com-56163ca2/blog-8caafe43/.
- Components: src/components/sites/developers-openai-com-56163ca2/blog-8caafe43/.
- Shared components: src/components/sites/developers-openai-com-56163ca2/shared/.
- Assets: public/sites/developers-openai-com-56163ca2/blog-8caafe43/; site fonts/icons in public/sites/developers-openai-com-56163ca2/shared/.
- Downloader: scripts/download-assets-developers-openai-com-56163ca2-blog-8caafe43.mjs.
- Foundation: root layout will use extracted local fonts and metadata; globals.css will retain scaffold tokens and add site-scoped styles.
- Existing work: only scaffold route exists. Preserve docs/assets, existing comparison screenshot, UI primitives, and the user's existing package.json modification.
- No route or namespace collisions, stateful query/fragment targets, or multiple origins.
- Article/topic/navigation links retain original source destinations; only the requested blog index is cloned. Local interactions: theme, mobile navigation, search, docs-agent panel.
