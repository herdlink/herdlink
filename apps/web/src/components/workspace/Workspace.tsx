"use client";

import Link from "next/link";
import Image from "next/image";
import { usePathname } from "next/navigation";
import { useEffect, useState, useSyncExternalStore } from "react";
import { Moon, PanelLeftClose, PanelLeftOpen, Sun, LogOut } from "lucide-react";

import { useAuth, logout } from "@/components/auth/AuthProvider";
import { GraphChat, ChatHistory } from "@/components/chat/GraphChat";

const navigation = [
  { label: "Surveys", href: "/surveys" },
  { label: "Inbox", href: "/inbox" },
  { label: "Home", href: "/" },
  { label: "Communities", href: "/communities" },
];

function subscribeTheme(callback: () => void) {
  window.addEventListener("storage", callback);
  window.addEventListener("herdlink-theme-change", callback);
  return () => {
    window.removeEventListener("storage", callback);
    window.removeEventListener("herdlink-theme-change", callback);
  };
}

function readTheme() {
  try { return localStorage.getItem("herdlink-theme") === "dark" ? "dark" : "light"; }
  catch { return "light"; }
}

export function Workspace({ children }: { children: React.ReactNode }) {
  const pathname = usePathname();
  const { user } = useAuth();
  const displayName = user.username === "demo_user" ? "Demo user" : user.username;
  const [logoutError, setLogoutError] = useState("");
  const isCommunity = pathname.startsWith("/community/");
  const isCommunityDirectory = pathname === "/communities";
  const withoutChat = isCommunity || isCommunityDirectory || pathname.startsWith("/surveys");
  const theme = useSyncExternalStore(subscribeTheme, readTheme, () => "light");
  const [historyCollapsed, setHistoryCollapsed] = useState(true);
  useEffect(() => {
    document.documentElement.dataset.siteTheme = theme;
    return () => { delete document.documentElement.dataset.siteTheme; };
  }, [theme]);

  function toggleTheme() {
    try { localStorage.setItem("herdlink-theme", theme === "dark" ? "light" : "dark"); }
    catch { return; }
    window.dispatchEvent(new Event("herdlink-theme-change"));
  }

  return (
    <div className={`herdlink-site flex flex-col ${withoutChat ? "h-dvh overflow-hidden" : "min-h-dvh lg:h-dvh lg:overflow-hidden"}`} data-theme={theme}>
      <a href="#workspace-content" className="sr-only z-50 rounded-full bg-[var(--site-solid)] px-4 py-2 text-[var(--site-solid-text)] focus:not-sr-only focus:absolute focus:left-4 focus:top-4">Skip to content</a>
      <header className="flex shrink-0 flex-wrap items-center justify-between gap-x-6 border-b border-[var(--site-border)] px-4 md:px-8 lg:h-16 lg:flex-nowrap">
        <Link href="/" aria-label="Herdlink home" className="flex h-16 shrink-0 items-center">
          <Image src="/logo.png" alt="Herdlink" width={1449} height={1086} sizes="96px" preload className={`h-14 w-24 object-cover ${theme === "dark" ? "invert mix-blend-screen" : "mix-blend-multiply"}`} />
        </Link>
        <nav aria-label="Primary navigation" className="order-3 flex w-full items-center gap-1 overflow-x-auto pb-3 lg:order-none lg:w-auto lg:pb-0">
          {navigation.map(({ label, href }) => (
            <Link key={href} href={href} aria-current={pathname === href || ((href === "/communities" && isCommunity) || (href === "/surveys" && pathname.startsWith("/surveys/"))) ? "page" : undefined} className={`shrink-0 rounded-full px-4 py-2 text-[14px] transition-colors hover:bg-[var(--site-hover)] ${pathname === href || ((href === "/communities" && isCommunity) || (href === "/surveys" && pathname.startsWith("/surveys/"))) ? "bg-[var(--site-hover)] font-medium" : "text-[var(--site-secondary)]"}`}>{label}</Link>
          ))}
        </nav>
        <div className="flex shrink-0 items-center gap-3">
          <button aria-label="Toggle light and dark theme" onClick={toggleTheme} className="flex h-10 w-10 items-center justify-center rounded-full text-[var(--site-secondary)] hover:bg-[var(--site-hover)]">
            {theme === "dark" ? <Moon size={17} /> : <Sun size={17} />}
          </button>
          <div aria-label={`Signed in as ${displayName}`} className="flex items-center gap-2.5">
            <span aria-hidden="true" className="flex h-8 w-8 items-center justify-center rounded-full bg-[var(--site-hover)] text-xs font-semibold">{displayName[0]?.toUpperCase()}</span>
            <div className="text-left leading-4">
              <p className="text-[13px] font-medium">{displayName}</p>
              <p className="mt-0.5 text-[11px] text-[var(--site-secondary)]">{user.role === "user" ? "User" : user.role.replaceAll("_", " ")}</p>
            </div>
          </div>
          <button type="button" aria-label="Log out" title="Log out" onClick={() => { void logout().catch(() => setLogoutError("You have been logged out locally.")); }} className="flex h-8 w-8 items-center justify-center rounded-full text-[var(--site-secondary)] hover:bg-[var(--site-hover)]"><LogOut size={15} /></button>
        </div>
      </header>
      {logoutError && <p role="status" className="px-5 text-xs">{logoutError}</p>}

      {withoutChat ? (
        <main id="workspace-content" tabIndex={-1} className={`min-h-0 flex-1 focus:outline-none ${!isCommunity ? "overflow-y-auto" : ""}`}>{children}</main>
      ) : (
      <div className={`grid min-h-0 flex-1 grid-cols-1 md:grid-cols-[minmax(0,1fr)_320px] ${historyCollapsed ? "lg:grid-cols-[56px_minmax(0,1fr)_minmax(300px,20%)]" : "lg:grid-cols-[220px_minmax(0,1fr)_minmax(300px,20%)]"}`}>
        <aside aria-labelledby="history-heading" className="flex min-h-0 flex-col border-b border-[var(--site-border)] md:col-span-2 lg:col-span-1 lg:border-r lg:border-b-0">
          <header className={`flex h-12 shrink-0 items-center lg:h-16 ${historyCollapsed ? "justify-center px-2" : "justify-between px-5"}`}>
            <h2 id="history-heading" className={historyCollapsed ? "sr-only" : "text-[14px] font-semibold"}>Chat history</h2>
            <button type="button" aria-label={historyCollapsed ? "Expand chat history" : "Collapse chat history"} aria-expanded={!historyCollapsed} aria-controls="chat-history-content" title={historyCollapsed ? "Expand chat history" : "Collapse chat history"} onClick={() => setHistoryCollapsed((value) => !value)} className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full text-[var(--site-secondary)] hover:bg-[var(--site-hover)]">{historyCollapsed ? <PanelLeftOpen size={18} /> : <PanelLeftClose size={18} />}</button>
          </header>
          <div id="chat-history-content" hidden={historyCollapsed} className="min-h-0 flex-1"><ChatHistory /></div>
        </aside>

        <main id="workspace-content" tabIndex={-1} className="min-h-0 min-w-0 overflow-y-auto bg-[var(--site-surface)] focus:outline-none">{children}</main>

        <aside aria-labelledby="chat-heading" className="flex h-[560px] min-h-0 flex-col border-t border-[var(--site-border)] bg-[var(--site-panel)] md:h-auto md:border-t-0 md:border-l">
          <GraphChat />
        </aside>
      </div>
      )}
    </div>
  );
}
