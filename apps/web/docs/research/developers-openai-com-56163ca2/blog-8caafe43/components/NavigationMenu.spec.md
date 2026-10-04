# NavigationMenu Specification
## Overview
- Target: src/components/sites/developers-openai-com-56163ca2/blog-8caafe43/NavigationMenu.tsx
- Screenshot: docs/design-references/developers-openai-com-56163ca2/blog-8caafe43/original-menu-Resources.png (also API and ChatGPT variants).
- Interaction model: hover/focus dropdown, native top navigation link.
## DOM Structure
Relative group contains top-level anchor with optional ChevronDownIcon, and absolute dropdown containing menu item anchors. Named export NavigationMenu takes label: string, href: string, items?: NavigationItem[], active?: boolean. Import NavigationItem from ./navigation and ChevronDownIcon from ../shared/icons.
## Computed Styles
- Top anchor: flex align center gap 4px; padding 4px 10px; font 14px/20px weight 400 tracking -0.14px, radius 8px; no underline.
- Chevron 14px square. Home has no dropdown/chevron.
- Wrapper position relative, flex-shrink 0.
- Dropdown: absolute left 0 top 100%, margin-top 8px, width max-content min-width 100%, z-index 50; opacity 0, visibility hidden normally; visible opacity 1 on group hover/focus-within; opacity transition 150ms cubic-bezier(0.4,0,0.2,1). Add an invisible 8px bridge to retain pointer hover.
- Inner menu: radius 8px, border 1px solid rgba(40,40,40,.05), background white, ring rgba(0,0,0,.05), shadow 0 2px 4px -1px rgba(0,0,0,.08).
- Item anchors: block, padding 12px 16px; font 14px/20px, color #282828. Inner flex column gap 4px; label weight 500, description weight 400 #5d5d5d. Preserve description unwrapped on desktop.
- Extracted menu widths at 1440: API 332.172px, ChatGPT 486.609px, Resources 370.422px. Width intrinsic from actual text and fonts.
## States & Behaviors
- Active Resources and hovered top anchor background rgba(40,40,40,.08), inactive transparent; item hover rgba(40,40,40,.04) with 150ms color transition.
- Top anchor click navigates to href. Keyboard focus opens dropdown; dropdown links navigate to item.href (Blog maps locally to /).
- Use site CSS variables for dark support; no scroll-driven transitions.
## Per-State Content
Real label/description/href records already extracted in navigationMenus in ./navigation.ts. Parent supplies its items.
## Assets
ChevronDownIcon extracted from source SVG in ../shared/icons.tsx. No images.
## Text Content (verbatim)
Home; API; ChatGPT; Resources. Resource menu: Showcase — Demo apps to get inspired; Blog — Learnings and experiences from developers; Cookbook — Notebook examples for building with OpenAI models; Learn — Docs, videos, and demo apps for building with OpenAI; Community — Programs, meetups, and support for builders.
## Responsive Behavior
- Desktop >=1024px: displayed inside SiteHeader nav. Parent hides nav below 1024px.
- Tablet/mobile: no instance visible; mobile drawer is separate.
