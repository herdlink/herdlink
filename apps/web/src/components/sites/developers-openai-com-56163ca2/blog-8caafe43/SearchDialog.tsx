"use client";

import { useEffect, useMemo, useRef, useState } from "react";
import { CloseIcon } from "../shared/icons";
import { articles } from "./content";

interface SearchDialogProps {
  initialQuery?: string;
  onClose: () => void;
}

export function SearchDialog({ initialQuery = "", onClose }: SearchDialogProps) {
  const [query, setQuery] = useState(initialQuery);
  const [selected, setSelected] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const dialogRef = useRef<HTMLDivElement>(null);
  const results = useMemo(() => {
    const terms = query.toLowerCase().trim().split(/\s+/).filter(Boolean);
    return articles.filter((article) => terms.every((term) => `${article.title} ${article.description} ${article.topic}`.toLowerCase().includes(term)));
  }, [query]);

  useEffect(() => {
    const previousFocus = document.activeElement;
    inputRef.current?.focus();
    return () => {
      if (previousFocus instanceof HTMLElement) previousFocus.focus();
    };
  }, []);

  function updateQuery(value: string) {
    setQuery(value);
    setSelected(0);
  }

  return (
    <div className="fixed inset-0 z-[60] flex items-start justify-center px-4 pt-20 pb-10 md:px-6 md:pt-24">
      <div onClick={onClose} className="absolute inset-0 bg-black/35 backdrop-blur-[4px]" aria-hidden="true" />
      <div
        ref={dialogRef}
        role="dialog"
        aria-modal="true"
        aria-label="Search developer resources"
        className="relative z-10 flex w-full max-w-4xl flex-col overflow-hidden rounded-[28px] bg-[var(--site-panel)] shadow-[0_36px_120px_-48px_rgba(15,23,42,0.55)] ring-1 ring-black/10"
        onKeyDown={(event) => {
          if (event.key === "Tab") {
            const focusable = dialogRef.current?.querySelectorAll<HTMLElement>("input,button,a[href]");
            if (!focusable?.length) return;
            const first = focusable[0];
            const last = focusable[focusable.length - 1];
            if (event.shiftKey && document.activeElement === first) { event.preventDefault(); last.focus(); }
            if (!event.shiftKey && document.activeElement === last) { event.preventDefault(); first.focus(); }
          }
        }}
      >
        <button aria-label="Close search" onClick={onClose} className="absolute top-7 right-5 z-20 flex h-8 w-8 items-center justify-center rounded-lg text-[#8f8f8f] hover:text-[var(--site-text)] md:right-7">
          <CloseIcon className="site-icon" />
        </button>
        <input
          ref={inputRef}
          aria-label="Search developer resources"
          placeholder="Start searching"
          value={query}
          onChange={(event) => updateQuery(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === "ArrowDown") { event.preventDefault(); setSelected((value) => Math.min(value + 1, results.length - 1)); }
            if (event.key === "ArrowUp") { event.preventDefault(); setSelected((value) => Math.max(value - 1, 0)); }
            if (event.key === "Enter" && query.trim() && results[selected]) window.location.assign(results[selected].href);
          }}
          className="min-h-16 w-full border-0 border-b border-solid border-[var(--site-border)] bg-transparent px-6 py-5 pr-20 text-[18px] leading-[24px] text-[var(--site-text)] outline-none placeholder:text-[#8f8f8f]"
        />
        {query.trim() ? (
          <div className="max-h-[60vh] overflow-y-auto px-3 py-3" aria-live="polite">
            {results.length ? results.map((article, index) => (
              <a key={article.href} href={article.href} className={`block rounded-lg px-3 py-3 hover:bg-[var(--site-hover)] ${index === selected ? "bg-[var(--site-soft)]" : ""}`}>
                <div className="text-[14px] leading-5 text-[var(--site-secondary)]">Blog · {article.topic}</div>
                <div className="mt-1 text-[16px] leading-6 font-medium">{article.title}</div>
                <p className="mt-1 line-clamp-2 text-[14px] leading-5 text-[var(--site-secondary)]">{article.description}</p>
              </a>
            )) : <p className="px-3 py-5 text-[14px] text-[var(--site-secondary)]">No results found.</p>}
          </div>
        ) : (
          <div className="px-6 py-5">
            <p className="mb-2.5 text-[14px] leading-[21px] text-[#8f8f8f]">Suggested</p>
            <div className="flex flex-wrap gap-2">
              {["responses create", "reasoning_effort", "realtime", "prompt caching"].map((suggestion) => (
                <button key={suggestion} onClick={() => { updateQuery(suggestion); inputRef.current?.focus(); }} className="rounded-full border border-[var(--site-border)] px-3 py-[5.6px] text-[14px] leading-[21px] transition-colors hover:bg-[var(--site-hover)]">
                  {suggestion}
                </button>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
