# SiteHeader Specification
## Overview
Target SiteHeader.tsx in page component namespace; screenshot original-desktop-top.png and original-mobile-top.png. Click controls and hover dropdown navigation.
## DOM Structure
Fixed header > grid/flex row with logo anchor, action group, desktop nav of NavigationMenu instances.
## Computed Styles
Height64px,fixed top0,width100%,z50,background#fff,dark#000,border-bottom1px rgba(40,40,40,.05). Inner height63,padding0 16px mobile,32px >=768; >=1024 grid minmax(0,1fr) auto minmax(0,1fr),gap24. Logo192×24,desktop margin-left-8px,min-height44. Action gap16 mobile,12 tablet,20 desktop. Dashboard14px/20px,weight500,px16 py8,bg#181818,textwhite,radiusfull,gap8,arrow14. Theme icon16. Search/menu44-square radiusfull icons16. Desktop navigation gap4,14px/20px.
## States & Behaviors
Theme toggles, search opens dialog, menu toggles drawer; buttons hover text/default and soft bg in150ms. Menu at mobile right4px; selected soft background. Desktop header unchanged on scroll. On docs-agent open >=768 width calc(100% -440px),300ms transition.
## Per-State Content
Desktop Home/API/ChatGPT/Resources, Resources active. Mobile logo/search/menu only. >=1536 search pill Start searching replaces icon.
## Assets
Shared icons SearchIcon,SunIcon,MoonIcon,MenuIcon,ExternalLinkIcon; logo /sites/developers-openai-com-56163ca2/shared/icons/logo.svg.
## Text Content (verbatim)
OpenAI Developers; API Dashboard; Toggle light and dark theme; Search developer resources; Toggle menu; Home; API; ChatGPT; Resources; Start searching.
## Responsive Behavior
Desktop navigation/dashboard/theme >=1024; mobile menu <1024; row padding breakpoint768; search pill1536.
