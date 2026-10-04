"use client";

import { useState } from "react";
import { CloseIcon, NewChatIcon } from "../shared/icons";
import { articles } from "./content";

interface DocsAgentProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function DocsAgent({ open, onOpenChange }: DocsAgentProps) {
  const [query, setQuery] = useState("");
  const [submitted, setSubmitted] = useState("");
  const matches = articles.filter((article) => `${article.title} ${article.description}`.toLowerCase().includes(submitted.toLowerCase())).slice(0, 3);

  return (
    <>
      {!open && <button onClick={() => onOpenChange(true)} aria-haspopup="dialog" aria-expanded={false} className="fixed right-5 bottom-5 z-50 flex h-11 items-center justify-center rounded-full border border-transparent bg-[var(--site-solid)] px-4 text-[14px] leading-5 font-medium text-[var(--site-solid-text)] shadow-[0_16px_48px_-18px_rgba(15,23,42,0.45)] transition-opacity hover:opacity-85">Ask AI</button>}
      <aside role="dialog" aria-label="Docs agent" aria-hidden={!open} inert={!open} className={`fixed inset-x-0 bottom-0 z-[80] flex h-[min(78dvh,640px)] flex-col overflow-hidden rounded-t-2xl border border-[var(--site-border)] bg-[var(--site-panel)] transition-transform duration-300 ease-out md:inset-y-0 md:left-auto md:right-0 md:h-auto md:w-[440px] md:rounded-none md:border-y-0 md:border-r-0 ${open ? "translate-y-0 md:translate-x-0" : "translate-y-full md:translate-x-full md:translate-y-0"}`}>
        <header className="flex h-16 shrink-0 items-center justify-between border-b border-[var(--site-border)] px-4">
          <h2 className="text-[14px] leading-5 font-semibold">Docs agent</h2>
          <div className="flex gap-1.5">
            <button aria-label="Start a new docs agent chat" onClick={() => { setSubmitted(""); setQuery(""); }} className="flex h-8 w-8 items-center justify-center rounded-lg text-[var(--site-secondary)] hover:bg-[var(--site-hover)]"><NewChatIcon className="site-icon" /></button>
            <button aria-label="Close docs agent" onClick={() => onOpenChange(false)} className="flex h-8 w-8 items-center justify-center rounded-lg text-[var(--site-secondary)] hover:bg-[var(--site-hover)]"><CloseIcon className="site-icon" /></button>
          </div>
        </header>
        <div className="flex min-h-0 flex-1 flex-col">
          <div className="flex flex-1 flex-col justify-center overflow-y-auto px-6 py-8">
            {submitted ? (
              <div className="space-y-4">
                <p className="ml-8 rounded-2xl bg-[var(--site-hover)] px-4 py-3 text-[14px]">{submitted}</p>
                <p className="text-[14px] text-[var(--site-secondary)]">Explore these posts from the developer blog:</p>
                {(matches.length ? matches : articles.slice(0, 3)).map((article) => <a key={article.href} href={article.href} className="block rounded-xl border border-[var(--site-border)] p-4 text-[14px] hover:bg-[var(--site-hover)]"><p className="font-medium">{article.title}</p><p className="mt-2 text-[var(--site-secondary)]">{article.description}</p></a>)}
              </div>
            ) : (
              <div>
                <h3 className="mb-6 text-center text-[24px] leading-8 font-medium">What can I help you with?</h3>
                <div className="flex flex-col items-center gap-2">
                  {["Ask a question", "Find a page", "Build a custom guide"].map((prompt) => <button key={prompt} onClick={() => setSubmitted(prompt === "Find a page" ? "Codex" : prompt === "Build a custom guide" ? "API" : "OpenAI")} className="rounded-full border border-[var(--site-border)] px-4 py-2 text-[14px] hover:bg-[var(--site-hover)]">{prompt}</button>)}
                </div>
              </div>
            )}
          </div>
          <form onSubmit={(event) => { event.preventDefault(); if (query.trim()) { setSubmitted(query.trim()); setQuery(""); } }} className="m-4 flex items-center gap-2 rounded-2xl border border-[var(--site-border)] bg-[var(--site-soft)] p-3">
            <input aria-label="Ask about docs" placeholder="Ask about docs or what you want to build" value={query} onChange={(event) => setQuery(event.target.value)} className="min-w-0 flex-1 bg-transparent text-[14px] outline-none" />
            <button type="submit" aria-label="Send message" disabled={!query.trim()} className="h-8 w-8 rounded-full bg-[var(--site-solid)] text-[var(--site-solid-text)] disabled:opacity-30">↑</button>
          </form>
        </div>
      </aside>
    </>
  );
}
