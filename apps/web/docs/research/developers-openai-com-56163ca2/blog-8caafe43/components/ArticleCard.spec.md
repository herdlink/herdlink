# ArticleCard Specification
## Overview
- Target: src/components/sites/developers-openai-com-56163ca2/blog-8caafe43/ArticleCard.tsx
- Screenshot: docs/design-references/developers-openai-com-56163ca2/blog-8caafe43/original-desktop-top.png and original-mobile-top.png.
- Interaction model: native link; static layout, no card hover transformation.
## DOM Structure
Anchor flex column contains image wrapper/image, date div, title div with nested two-line-clamped text, description paragraph, topic div. Props: article: BlogArticle from ./types.
## Computed Styles
- Anchor: width 100%, display flex, flex-direction column, overflow hidden, padding 0 (mobile), 0 32px (>=768px). No border, shadow, or background.
- Image: display block, width 100%, height auto using actual natural aspect ratio; desktop radius 8px at >=768px, mobile 0. Source attrs width 800 height 295 but actual image ratios vary slightly. Use ordinary img to preserve original rendering, extracted dimensions, and local source.
- Date: 16px/24px, weight 400, tracking -0.16px, #5d5d5d; padding 16px 16px 0 on mobile, 16px 0 0 on >=768px.
- Title: 18px/24px, weight 500, tracking -0.18px, #282828; padding 0 16px 8px mobile, 0 0 8px desktop; nested text clamps to two lines.
- Description: 16px/24px, 400, tracking -0.16px, #282828; padding 0 16px mobile, 0 desktop; margin 0; clamps to three lines; empty paragraph has height 0.
- Topic: 14px/20px, 400, tracking -0.14px, #5d5d5d; padding 8px 16px 0 mobile, 8px 0 0 desktop.
## States & Behaviors
- Card hover: no change in color, image scale, or layout. Entire anchor navigates to article.href.
- Dark text uses var(--site-text), muted uses var(--site-secondary). Parent supplies font and theme.
## Per-State Content
N/A; each record is a static article. All 30 real records already in ./content.ts.
## Assets
article.image is a namespaced downloaded local WebP; article.alt is extracted verbatim. No overlay, videos, or icons.
## Text Content (verbatim)
First card: Sep 23; Bringing my LED display to life with GPT-Live-1 and Codex; How I used Codex, GPT-Live-1, and a Raspberry Pi to turn an LED display into a voice-controlled assistant.; Codex.
Render props verbatim for remaining cards.
## Responsive Behavior
- 1440: anchor 1120px, content/image 1056px, first image height 389.390625px, first card 513.390625px.
- 768: image/content width 598px; no sidebar. Card horizontal padding remains 32px.
- 390: available viewport content 380px (10px scrollbar), anchor/image full-width, text inset 16px, first card 336.125px.
- Breakpoint: 768px.
