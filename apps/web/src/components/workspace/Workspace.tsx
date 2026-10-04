"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { useEffect, useRef, useState, useSyncExternalStore } from "react";
import { ArrowUp, MessageCircle, Moon, Plus, SquarePen, Sun } from "lucide-react";

const navigation = [
  { label: "Surveys", href: "/surveys" },
  { label: "Institutions", href: "/institutions" },
  { label: "Inbox", href: "/inbox" },
  { label: "Home", href: "/" },
  { label: "Communities", href: "/communities" },
];

type Chat = { id: string; title: string; messages: string[] };

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

export function Workspace({ children }: { children: React.ReactNode }) {
  const pathname = usePathname();
  const isCommunity = pathname.startsWith("/community/");
  const isCommunityDirectory = pathname === "/communities";
  const withoutChat = isCommunity || isCommunityDirectory;
  const theme = useSyncExternalStore(subscribeTheme, readTheme, () => "light");
  const [chats, setChats] = useState<Chat[]>([]);
  const [activeId, setActiveId] = useState<string | null>(null);
  const [drafts, setDrafts] = useState<Record<string, string>>({});
  const activeChat = chats.find((chat) => chat.id === activeId);
  const draftKey = activeId ?? "new";
  const query = drafts[draftKey] ?? "";
  const inputRef = useRef<HTMLTextAreaElement>(null);
  const messagesRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    document.documentElement.dataset.siteTheme = theme;
    return () => { delete document.documentElement.dataset.siteTheme; };
  }, [theme]);

  useEffect(() => {
    messagesRef.current?.scrollTo({ top: messagesRef.current.scrollHeight });
  }, [activeChat]);

  function toggleTheme() {
    try { localStorage.setItem("openai-blog-theme", theme === "dark" ? "light" : "dark"); }
    catch { return; }
    window.dispatchEvent(new Event("openai-theme-change"));
  }

  function newChat() {
    setActiveId(null);
    inputRef.current?.focus();
  }

  function sendMessage() {
    const message = query.trim();
    if (!message) return;
    if (activeId) {
      setChats((current) => current.map((chat) => chat.id === activeId
        ? { ...chat, messages: [...chat.messages, message] } : chat));
    } else {
      const id = crypto.randomUUID();
      setChats((current) => [{ id, title: message, messages: [message] }, ...current]);
      setActiveId(id);
    }
    setDrafts((current) => ({ ...current, [draftKey]: "" }));
  }

  return (
    <div className={`openai-site flex flex-col ${withoutChat ? "h-dvh overflow-hidden" : "min-h-dvh lg:h-dvh lg:overflow-hidden"}`} data-theme={theme}>
      <a href="#workspace-content" className="sr-only z-50 rounded-full bg-[var(--site-solid)] px-4 py-2 text-[var(--site-solid-text)] focus:not-sr-only focus:absolute focus:left-4 focus:top-4">Skip to content</a>
      <header className="flex shrink-0 flex-wrap items-center justify-between gap-x-6 border-b border-[var(--site-border)] px-4 md:px-8 lg:h-16 lg:flex-nowrap">
        <Link href="/" className="flex h-16 items-center text-xl font-semibold tracking-tight">Herdlink</Link>
        <nav aria-label="Primary navigation" className="order-3 flex w-full items-center gap-1 overflow-x-auto pb-3 lg:order-none lg:w-auto lg:pb-0">
          {navigation.map(({ label, href }) => (
            <Link key={href} href={href} aria-current={pathname === href || (href === "/communities" && isCommunity) ? "page" : undefined} className={`shrink-0 rounded-full px-4 py-2 text-[14px] transition-colors hover:bg-[var(--site-hover)] ${pathname === href || (href === "/communities" && isCommunity) ? "bg-[var(--site-hover)] font-medium" : "text-[var(--site-secondary)]"}`}>{label}</Link>
          ))}
        </nav>
        <div className="flex shrink-0 items-center gap-3">
          <button aria-label="Toggle light and dark theme" onClick={toggleTheme} className="flex h-10 w-10 items-center justify-center rounded-full text-[var(--site-secondary)] hover:bg-[var(--site-hover)]">
            {theme === "dark" ? <Moon size={17} /> : <Sun size={17} />}
          </button>
          <div aria-label="Signed in as Demo user, User role" className="flex items-center gap-2.5">
            <span aria-hidden="true" className="flex h-8 w-8 items-center justify-center rounded-full bg-[var(--site-hover)] text-xs font-semibold">D</span>
            <div className="text-left leading-4">
              <p className="text-[13px] font-medium">Demo user</p>
              <p className="mt-0.5 text-[11px] text-[var(--site-secondary)]">User</p>
            </div>
          </div>
        </div>
      </header>

      {withoutChat ? (
        <main id="workspace-content" tabIndex={-1} className={`min-h-0 flex-1 focus:outline-none ${isCommunityDirectory ? "overflow-y-auto" : ""}`}>{children}</main>
      ) : (
      <div className="grid min-h-0 flex-1 grid-cols-1 md:grid-cols-[minmax(0,1fr)_320px] lg:grid-cols-[220px_minmax(0,1fr)_minmax(300px,20%)]">
        <aside aria-labelledby="history-heading" className="flex min-h-0 flex-col border-b border-[var(--site-border)] md:col-span-2 lg:col-span-1 lg:border-r lg:border-b-0">
          <header className="flex h-16 shrink-0 items-center justify-between px-5">
            <h2 id="history-heading" className="text-[14px] font-semibold">Chat history</h2>
            <button onClick={newChat} aria-label="New chat" className="flex h-8 w-8 items-center justify-center rounded-lg text-[var(--site-secondary)] hover:bg-[var(--site-hover)]"><SquarePen size={16} /></button>
          </header>
          <div className="px-3 pb-3">
            <button onClick={newChat} className="flex w-full items-center gap-2 rounded-full border border-[var(--site-border)] px-3 py-2 text-[14px] hover:bg-[var(--site-hover)]"><Plus size={16} />New chat</button>
          </div>
          <div className="max-h-48 overflow-y-auto px-3 pb-5 lg:max-h-none lg:flex-1">
            {chats.length ? (
              <ul className="space-y-1">
                {chats.map((chat) => (
                  <li key={chat.id}>
                    <button onClick={() => setActiveId(chat.id)} aria-pressed={chat.id === activeId} title={chat.title} className={`flex w-full items-center gap-2 rounded-lg px-3 py-2 text-left text-[14px] hover:bg-[var(--site-hover)] ${chat.id === activeId ? "bg-[var(--site-hover)]" : ""}`}>
                      <MessageCircle size={15} className="shrink-0 text-[var(--site-secondary)]" /><span className="truncate">{chat.title}</span>
                    </button>
                  </li>
                ))}
              </ul>
            ) : <p className="px-3 py-3 text-[13px] leading-5 text-[var(--site-secondary)]">Your conversations will appear here.</p>}
          </div>
        </aside>

        <main id="workspace-content" tabIndex={-1} className="min-h-0 min-w-0 overflow-y-auto bg-[var(--site-surface)] focus:outline-none">{children}</main>

        <aside aria-labelledby="chat-heading" className="flex h-[560px] min-h-0 flex-col border-t border-[var(--site-border)] bg-[var(--site-panel)] md:h-auto md:border-t-0 md:border-l">
          <header className="flex h-16 shrink-0 items-center justify-between border-b border-[var(--site-border)] px-5">
            <h2 id="chat-heading" className="text-[14px] font-semibold">Herdlink assistant</h2>
            <button aria-label="Start a new assistant chat" onClick={newChat} className="flex h-8 w-8 items-center justify-center rounded-lg text-[var(--site-secondary)] hover:bg-[var(--site-hover)]"><SquarePen size={16} /></button>
          </header>
          <div ref={messagesRef} className="min-h-0 flex-1 overflow-y-auto px-5 py-8">
            {activeChat ? (
              <div role="log" aria-label="Chat messages" aria-live="polite" className="space-y-4">
                {activeChat.messages.map((message, index) => <p key={index} className="ml-5 whitespace-pre-wrap break-words rounded-2xl bg-[var(--site-hover)] px-4 py-3 text-[14px] leading-6"><span className="sr-only">You: </span>{message}</p>)}
                <p role="status" className="text-[13px] leading-5 text-[var(--site-secondary)]">The assistant isn’t connected yet. Your messages are kept in this workspace until you refresh.</p>
              </div>
            ) : (
              <div className="pt-6">
                <MessageCircle size={24} strokeWidth={1.5} className="mb-5 text-[var(--site-secondary)]" />
                <h3 className="text-[24px] leading-8 font-medium tracking-tight">What can I help<br />you explore?</h3>
                <p className="mt-3 text-[14px] leading-6 text-[var(--site-secondary)]">Start a conversation alongside your graph.</p>
              </div>
            )}
          </div>
          <form onSubmit={(event) => { event.preventDefault(); sendMessage(); }} className="m-4 mt-0 rounded-2xl border border-[var(--site-border)] bg-[var(--site-soft)] p-3 focus-within:ring-1 focus-within:ring-[var(--site-secondary)]">
            <textarea ref={inputRef} aria-label="Message Herdlink assistant" placeholder="Ask a question…" rows={2} value={query} onChange={(event) => setDrafts((current) => ({ ...current, [draftKey]: event.target.value }))} onKeyDown={(event) => {
              if (event.key === "Enter" && !event.shiftKey && !event.nativeEvent.isComposing) { event.preventDefault(); sendMessage(); }
            }} className="block max-h-40 min-h-12 w-full resize-none bg-transparent text-[14px] leading-6 outline-none" />
            <div className="mt-2 flex items-center justify-between gap-2">
              <span className="text-[11px] text-[var(--site-secondary)]">Assistant preview</span>
              <button type="submit" aria-label="Send message" disabled={!query.trim()} className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-[var(--site-solid)] text-[var(--site-solid-text)] disabled:cursor-not-allowed disabled:opacity-30"><ArrowUp size={17} /></button>
            </div>
          </form>
        </aside>
      </div>
      )}
    </div>
  );
}
