# OpenAI Developer Blog clone QA

Source: https://developers.openai.com/blog  
Destination: http://localhost:3000/  
Route: src/app/page.tsx

## Scope and preservation

The untouched root scaffold was replaced under the clone skill's single-page routing default. There were no other authored page routes to replace. Existing package changes and scaffold UI utilities were preserved. Source article, topic, and documentation destinations remain external; local Blog and All posts links return to /.

Seven sections were assembled: header, sidebar, introduction, article feed, mobile navigation, search dialog, and docs-agent panel. There are nine UI components and eight extracted SVG icon components, with 17 corresponding specifications.

Downloaded assets: 30 article covers, three OpenAI Sans font weights, one logo SVG, and one favicon (35 files). Eight inline SVG icons were separately extracted into React components.

## Validation

- npm run check passed: ESLint, TypeScript, and production build.
- Production routes: / and Next.js's /_not-found.
- 43 browser checks passed with zero console errors or page exceptions in the clean local interaction run.
- All 30 covers loaded; no horizontal overflow at 1440, 768, or 390 pixels.
- All 30 article anchor bounds matched the source at all three widths within 0.05 pixels.
- Fully loaded page heights matched: 17,055px desktop, 12,518px tablet, 10,910px mobile.
- Same-viewport top, middle, and bottom screenshots were compared on desktop, tablet, and mobile.
- Desktop hover menus, persistent theme, search focus/escape/shortcut/suggestions/results/empty state, all three mobile tabs and 15 nested menu variants, mobile search handoff, local All posts navigation, and desktop/mobile docs-agent controls were exercised.

Corrections from QA included the stable 10px scrollbar gutter, explicit 8px image radius, dashboard font/icon size, dropdown tracking, mobile tab alignment, suggestion dimensions, navigation spacing, and dark drawer colors.

The source scrolls its body; the clone uses native document scrolling. Both retain the fixed header/sidebar and identical feed geometry. No smooth-scroll library, scroll snapping, or entrance animations were observed.

## Deliberate demo limitations

Search filters the 30 extracted blog posts rather than OpenAI's full documentation index. The docs-agent panel supplies local demo article suggestions; it does not call ChatKit or OpenAI services, and its conversational content differs from the live backend. Only the blog index route was cloned; article, topic, and documentation pages open their original destinations.

## Evidence

- QA_GEOMETRY.json: source/clone measurements for all cards and all three widths.
- QA_INTERACTIONS.json: 43 successful browser checks.
- QA_VISUAL.json: final page heights, overflow checks, and local All posts navigation.
- QA_DRAWER_SOURCE.json: re-extracted mobile light/dark computed styles.
- Screenshots: ../../../design-references/developers-openai-com-56163ca2/blog-8caafe43/
