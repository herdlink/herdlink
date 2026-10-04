# DocsAgent Specification
## Overview
Target DocsAgent.tsx in page namespace; screenshot original-docs-agent.png. Click-driven panel and demo conversation.
## DOM Structure
Ask AI launcher > fixed dialog containing64px header,Docs agent heading,new-chat/close buttons,body.
## Computed Styles
Launcherfixed right20 bottom20,z50,height44,px16,14px/20px500,radiusfull,bg#181818,textwhite,shadow0 16px48px -18px rgba(15,23,42,.45). Panelz80,width440desktop,fullheight,right0,borderleft subtle. Header64px px16,borderbottom; heading14px/20px600; buttons32-square,icons16,gap6. Loading status top16 inset16,padding12,radius8,border subtle,bg#f9f9f9,14px/20pxsecondary.
## States & Behaviors
PaneltranslateX(100%)→0 desktop; mobiletranslateY(100%)→0;300ms cubic-bezier(0,0,.2,1). Launcherhidden when open. Close/escape dismiss,new-chat resets demo. Original AI backend out of scope; show local demo using extracted article suggestions, no API credentials/calls.
## Per-State Content
Source initial: Loading docs agent...; source configured greeting: What can I help you with?; prompts: Ask a question; Find a page; Build a custom guide.
## Assets
NewChatIcon,CloseIcon from shared/icons; no images/videos.
## Text Content (verbatim)
Ask AI; Docs agent; Start a new docs agent chat; Close docs agent; Loading docs agent...; What can I help you with?
## Responsive Behavior
>=768right440panel with document width shrinking by440; mobilebottomsheetfullwidth,heightmin(78dvh,640px),radius16topcorners.
