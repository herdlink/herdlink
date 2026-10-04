# SearchDialog Specification
## Overview
Target SearchDialog.tsx in page namespace; screenshot original-search.png and original-search-results.png. Click/keyboard/input-driven modal.
## DOM Structure
Fixed backdrop and centered rounded panel with close button,input,empty suggestions or scrollable result anchors.
## Computed Styles
Overlay inset0,z60,padding80px16px40px mobile,96px24px40px>=768. Backdrop black35%,blur4. Panelwidth100%,max896,radius28,bgwhite dark#212121,shadow0 36px120px -48px rgba(15,23,42,.55),ringblack10%. Input18px/24px,padding20px24px,first row min64px,border-bottom subtle. Close32-square,right28px top28px,icon16. Empty suggestions padding20px24px,caption14px/21px muted,marginbottom10; pills14px/21px,px12 py5.6,border subtle,radiusfull,gap8.
## States & Behaviors
Input query filters extracted real blog title/description/topic records; source indexes full docs (intentionally demo scope). Empty suggestions populate query. Escape/backdrop/close dismiss; focus containment and trigger restoration. Keyboard arrows/Enter select search results.
## Per-State Content
Empty: Suggested, four pills. Query: article results; unmatched: No results found. Results use exact extracted titles and descriptions.
## Assets
CloseIcon from shared/icons; no images/videos.
## Text Content (verbatim)
Start searching; Search developer resources; Close search; Suggested; responses create; reasoning_effort; realtime; prompt caching.
## Responsive Behavior
Max896desktop; mobile available viewport minus32,top80; no sidebar in dialog; result height<=60vh.
