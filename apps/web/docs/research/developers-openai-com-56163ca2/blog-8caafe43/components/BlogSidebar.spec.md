# BlogSidebar Specification
## Overview
- Target: src/components/sites/developers-openai-com-56163ca2/blog-8caafe43/BlogSidebar.tsx
- Screenshot: docs/design-references/developers-openai-com-56163ca2/blog-8caafe43/original-desktop-top.png.
- Interaction model: fixed native navigation links with hover.
## DOM Structure
Fixed aside > scrollable nav > All posts list; Recent heading and five recent article links; Topics heading and six links. Import articles and topics from ./content.
## Computed Styles
- Aside: display flex column >=1024px, hidden below; fixed top 64px bottom 0 left 0, width 218px, z-index 40, padding 8px 12px 24px, background white (black dark).
- Nav: flex 1, overflow-y auto; first group margin-top 24px.
- Lists: flex column, gap 1px, margin/padding 0, no markers.
- Link: display block width 100%, radius 8px, padding 6px 12px; recent/topic links left padding 20px; 14px/20px weight 400 tracking -0.14px, #282828.
- Link text spans clamp to 2 lines; available recent text width 162px.
- Headings: 14px/20px weight 600, margin 24px 0 8px 12px; #181818.
- Active All posts background: oklab(0.159065 0.00000723451 0.00000317395 / 0.08), approx #ececec.
## States & Behaviors
- Links hover from transparent to same active background; transition color/background 150ms cubic-bezier(0.4,0,0.2,1).
- All posts href /; other article/topic links use original absolute source URLs. No scroll-driven selection changes.
- Theme colors var(--site-text), var(--site-secondary), var(--site-hover); sidebar dark background #000.
## Per-State Content
N/A; only All posts remains selected.
## Assets
N/A; no icons or images.
## Text Content (verbatim)
All posts; Recent; first five article titles from ./content; Topics; General; API; Apps SDK; Audio; Codex; Life sciences.
Topic hrefs https://developers.openai.com/blog/topic/{general,api,apps-sdk,audio,codex,life-sciences}.
## Responsive Behavior
- Desktop 1440: fixed 218px sidebar.
- Tablet 768 and mobile 390: hidden. Mobile drawer is a separate component.
- Breakpoint: 1024px.
