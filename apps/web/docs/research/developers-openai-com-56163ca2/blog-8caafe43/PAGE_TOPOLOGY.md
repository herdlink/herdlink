# Page topology

1. SiteHeader: fixed top0 height64, z50, site-wide navigation and mobile trigger.
2. BlogSidebar: fixed top64 bottom0 left0 width218, z40, desktop only.
3. BlogIntro: flow content under64px header; top padding80, centered heading and subtitle.
4. Article feed: flow single column, top padding80, gap40, 30 ArticleCard instances, bottom padding96. No footer.
5. MobileDrawer: click-driven overlay z40 behind header; three section tabs and blog navigation.
6. SearchDialog: click-driven modal z60.
7. DocsAgent: click-driven fixed launcher z50/panel z80.

Main padding-left240 at>=1024. Page max1152 at>=768. Container padding16 at>=1280,48 at768–1279,16 and margin16 below576,margin32 at576–767. Mobile article feed expands64 beyond container content to reach viewport edges. Thin scrollbar consumes10px in Chromium.

Global font OpenAI Sans; downloaded regular400,medium500,semibold600. Icons extracted from inline source SVG. All articles/static navigation data namespaced. Articles/topics beyond requested index use original source links.
