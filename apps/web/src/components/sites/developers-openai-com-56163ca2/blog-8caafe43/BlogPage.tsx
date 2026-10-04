"use client";

import { useEffect, useState, useSyncExternalStore } from "react";
import { ArticleCard } from "./ArticleCard";
import { BlogIntro } from "./BlogIntro";
import { BlogSidebar } from "./BlogSidebar";
import { DocsAgent } from "./DocsAgent";
import { MobileDrawer } from "./MobileDrawer";
import { SearchDialog } from "./SearchDialog";
import { SiteHeader } from "./SiteHeader";
import { articles } from "./content";

function subscribeTheme(callback: () => void) {
  window.addEventListener("storage", callback);
  window.addEventListener("openai-theme-change", callback);
  return () => {
    window.removeEventListener("storage", callback);
    window.removeEventListener("openai-theme-change", callback);
  };
}

function readTheme() {
  try { return localStorage.getItem("openai-blog-theme") === "dark" ? "dark" : "light"; }
  catch { return "light"; }
}

export function BlogPage() {
  const theme = useSyncExternalStore(subscribeTheme, readTheme, () => "light");
  const [menuOpen, setMenuOpen] = useState(false);
  const [searchOpen, setSearchOpen] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");
  const [agentOpen, setAgentOpen] = useState(false);

  function toggleTheme() {
    try { localStorage.setItem("openai-blog-theme", theme === "dark" ? "light" : "dark"); }
    catch { return; }
    window.dispatchEvent(new Event("openai-theme-change"));
  }

  function openSearch(query = "") {
    setMenuOpen(false);
    setSearchQuery(query);
    setSearchOpen(true);
  }

  useEffect(() => {
    document.documentElement.dataset.siteTheme = theme;
  }, [theme]);

  useEffect(() => {
    document.documentElement.dataset.siteScrollLock = String(menuOpen || searchOpen);
    return () => { delete document.documentElement.dataset.siteScrollLock; };
  }, [menuOpen, searchOpen]);

  useEffect(() => {
    function onKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") {
        if (searchOpen) setSearchOpen(false);
        else if (menuOpen) setMenuOpen(false);
        else setAgentOpen(false);
      }
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
        event.preventDefault();
        setMenuOpen(false);
        setSearchQuery("");
        setSearchOpen(true);
      }
    }
    function onResize() {
      if (window.innerWidth >= 1024) setMenuOpen(false);
    }
    window.addEventListener("keydown", onKeyDown);
    window.addEventListener("resize", onResize);
    return () => {
      window.removeEventListener("keydown", onKeyDown);
      window.removeEventListener("resize", onResize);
    };
  }, [menuOpen, searchOpen]);

  return (
    <div className="openai-site" data-theme={theme} data-agent-open={agentOpen}>
      <div className="site-document min-h-dvh pt-16">
        <SiteHeader theme={theme} menuOpen={menuOpen} onSearch={() => openSearch()} onToggleTheme={toggleTheme} onToggleMenu={() => setMenuOpen((open) => !open)} />
        <BlogSidebar />
        <main className="min-w-0 lg:pl-[240px]">
          <div className="blog-container flex flex-col items-center">
            <BlogIntro />
            <div className="w-full pt-20">
              <div className="-mx-8 flex w-[calc(100%+4rem)] flex-col gap-10 md:mx-0 md:w-full">
                {articles.map((article) => <ArticleCard key={article.href} article={article} />)}
              </div>
            </div>
          </div>
        </main>
      </div>
      <MobileDrawer open={menuOpen} theme={theme} onClose={() => setMenuOpen(false)} onToggleTheme={toggleTheme} onSearch={openSearch} />
      {searchOpen && <SearchDialog initialQuery={searchQuery} onClose={() => setSearchOpen(false)} />}
      <DocsAgent open={agentOpen} onOpenChange={setAgentOpen} />
    </div>
  );
}
