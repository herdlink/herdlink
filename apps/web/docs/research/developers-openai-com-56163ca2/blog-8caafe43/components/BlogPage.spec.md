# BlogPage Specification
## Overview
Target BlogPage.tsx in page namespace; screenshot original-desktop.png and original-mobile.png. Static flow plus click-driven overlays.
## DOM Structure
Site themed wrapper > document shell with SiteHeader,BlogSidebar,main > page-container > BlogIntro,feed of30ArticleCards; sibling MobileDrawer,SearchDialog,DocsAgent.
## Computed Styles
Site16px/24px,tracking-.01em. Documentpad-top64. Main min-width0,>=1024padding-left240. Pagecontainerpadding0 16px96px;margin0 16pxmobile,margin0 32px>=576;width100%max1152margin-inlineauto>=768 and padding48;>=1280padding16. Introstatic. Feedpad-top80;column gap40;widthcalc(100%+64px),margin-inline-32pxmobile;>=768width100%margin0.
## States & Behaviors
Native scrolling. Theme persists using localStorage. Escape closes modal/drawer/panel. Scroll locked for mobile drawer and search. Docs panel shifts document width440desktop300ms; no mobile shift.
## Per-State Content
30 article records verbatim; no pagination/infinite scroll/footer.
## Assets
Downloaded local images/fonts; shared extracted icons.
## Text Content (verbatim)
Entire content supplied by extracted ./content.ts. No added marketing text.
## Responsive Behavior
Widths1440,768,390 audited. Breakpoints576,768,1024,1280. Thin10px scrollbar in Chromium.
