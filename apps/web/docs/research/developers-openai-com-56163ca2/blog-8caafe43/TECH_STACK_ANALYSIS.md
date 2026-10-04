# Technical analysis

Source: Astro, utility CSS, self-hosted OpenAI Sans, client-hydrated search and ChatKit docs agent iframe. Native scrolling with fixed header/sidebar, CSS hover/focus dropdowns, click-driven mobile tabs. Clone: Next.js16.3.5 App Router/React19/strict TypeScript/Tailwind4, local fonts via next/font/local, source SVG paths as React components. Static article/menu records. Local article search and demo docs-agent suggestions avoid remote services. Fonts and raster assets stored locally; original article/topic/navigation URLs retained.

Build requires unrestricted subprocess execution in this environment: sandboxed Next.js receives empty output from its tsc child; same build passes outside sandbox without configuration changes.
