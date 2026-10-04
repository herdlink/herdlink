# MobileDrawer Specification
## Overview
Target MobileDrawer.tsx in page namespace; screenshots original-mobile-menu.png, original-mobile-API.png, original-mobile-ChatGPT.png, original-mobile-Resources.png. Click-driven tabs/subsections.
## DOM Structure
Fixed full-screen drawer behindheader,tab strip/theme button,scrollable search/suggestions/nav content,fixed footerdashboard. Mainthree tabs; within tab contextbutton selects lower nav panel.
## Computed Styles
Drawerinset0,z40,backgroundwhite dark#212121,flexcolumn,translateX100% closed→0open,300ms. Tabstripmargin-top64,padding24px24px0;height76includingpadding. Tabspillsheight44,letter-spacing normal,top88px,border1px rgba(40,40,40,.05),radiusfull,gap4,12px/18px400,px6py6mobile;>=640font14px/20px,px14. Theme44square at right. Scrollbodyflex1 minheight0 overflowauto px24py16,gap24. Searchinputbackground#ececec,radius18,padding16px 56px 16px 24px,font18/22.5,width100%,height56.5; suggestionscaption14/21muted,margin-bottom10px,pills14/21 border subtle px12py5.6,gap8. Contextlinks14/20,px12py8,radius10,weight600exceptShowcase400. Selectedbackground#e2e2e2. LowerAllposts14/20,px12py6,rounded10,bg#e2e2e2. HeadingsRecent/Topics12/18secondary,letter-spacing normal; groupsgap24px,heading-to-links12px,linkgap4px. Lowerlinks14/20 px12py6; no desktop2lineclamp. Footerminheight101,padding24,bordertop;dashboardfullwidthheight32pill14/14weight500,lightblack/white,darkwhite/black,arrow12. Dark selected contexts/linkswhite16%,searchwhite12%,selectedtab#303030.
## States & Behaviors
API/ChatGPT/Resources are click-to-switch tabs; Resources initial; defaultBlogcontext. Contextoption switches lowersection links (actual source datasets copied into ./mobile-navigation.ts). Sourceinternal navigation links remain original except/blog→/. Searchinput and suggestions invoke onSearch(query) and close drawer. Themebutton invokes toggle. Escape/onClose handled parent, tab state persists while drawerclosed; scroll locked by parent. No scroll-driven switching.
## Per-State Content
mobileNavigationPanels contains ALL extracted real lowerpanel entries and every section's contextoptions. API Overview/Models/Agents/Tools/Audio & voice/Production/API reference; ChatGPT Overview link,Sign in with ChatGPT/Plugins/Workspace Agents/Commerce/Ads plus external userdocs/usecases links; Resources Showcase link,Blog/Cookbook/Learn/Community. InitialBlogcontainsAllposts,Recent5,Topics6.
## Assets
SharedSunIcon,MoonIcon,ExternalLinkIcon,SearchIcon; no images/video.
## Text Content (verbatim)
Start searching; Suggested; responses create; reasoning_effort; realtime; prompt caching; API Dashboard; three tablabels and all exact entrytexts in mobileNavigationPanels.
## Responsive Behavior
Visibleonly<1024. Mobile390available380px,content332pxatx24. Tablet768samefullviewportdrawer. Tabfontbreakpoint640;hidden1024.
