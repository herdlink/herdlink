# Behaviors

Inspected using Playwright MCP at 1440×900, 768×900, and 390×844. Raw computed values and DOM are persisted in COMPUTED_STYLES.json, INTERACTION_STATES.json, NAV_STATES.json, and MOBILE_STATES.json.

- Native document scrolling, no Lenis/Locomotive, snapping, reveal, parallax, or scroll-selected tabs. Header remains fixed at 64px; sidebar fixed top64 bottom0 throughout the 30-card feed.
- Article cards are links, without scale, animation, or hover styling. All have single raster covers; no layered images, videos, or canvas.
- Desktop navigation: Home link and hover/focus dropdowns for API, ChatGPT, Resources. Opacity 0→1 and invisible→visible, 150ms cubic-bezier(.4,0,.2,1). Resources selected. Native links navigate to source destinations.
- Sidebar links: transparent→rgba(40,40,40,.08), 150ms color transition; All posts remains active. Article labels clamp2.
- Theme toggles light/dark and persists locally. Light page white, text #282828, heading#181818, secondary#5d5d5d. Dark page/header/sidebar black, text/heading#dcdcdc, secondary#b9b9b9, overlay panels#212121, primary buttons white.
- Mobile at <1024: desktop sidebar/nav/dashboard/theme hidden. Search and menu remain. Drawer slides from translateX(100%) to0 in300ms. API, ChatGPT, Resources tabs are click-driven; Resources defaults to Blog subsection. Theme available inside drawer. Page scrolling locked while open.
- Search icon opens a modal (z60), backdrop black35%, blur4px. Panel max896, radius28. Empty suggestions: responses create, reasoning_effort, realtime, prompt caching. Input filters content; Esc, close icon and backdrop dismiss; focus returns to trigger. Clone searches extracted blog records; source searches full site.
- Ask AI opens z80 right panel440px desktop, shrinking document/header width by440 with300ms transition; mobile bottom sheet min(78dvh,640px), radius16 top corners. Close and new-chat buttons present. Live docs agent requires remote backend; clone uses a demo state.
- Breakpoints:576px page margins32 (16 mobile);640px mobile tab type changes;768px article padding32/radius8, page padding48, subtitle18/29.25;1024px sidebar/header nav;1280px page padding16;1536px header search pill replaces search icon.
