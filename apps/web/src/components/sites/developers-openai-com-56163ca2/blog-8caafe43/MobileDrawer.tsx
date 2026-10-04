"use client";

import { useState, type FormEvent } from "react";
import { ExternalLinkIcon, MoonIcon, SunIcon } from "../shared/icons";
import {
  mobileNavigationPanels,
  type MobileNavigationEntry,
} from "./mobile-navigation";
import styles from "./MobileDrawer.module.css";

interface MobileDrawerProps {
  open: boolean;
  theme: string;
  onClose: () => void;
  onToggleTheme: () => void;
  onSearch: (query?: string) => void;
}

function destination(href: string) {
  if (href === "/" || href === "/blog" || href === "https://developers.openai.com/blog") return "/";
  return href.startsWith("/") ? `https://developers.openai.com${href}` : href;
}

function navigationGroups(entries: MobileNavigationEntry[]) {
  const groups: { heading?: string; entries: MobileNavigationEntry[] }[] = [];
  for (const entry of entries) {
    if (entry.kind === "heading") {
      groups.push({ heading: entry.text, entries: [] });
    } else {
      if (!groups.length) groups.push({ entries: [] });
      groups[groups.length - 1].entries.push(entry);
    }
  }
  return groups;
}

export function MobileDrawer({
  open,
  theme,
  onClose,
  onToggleTheme,
  onSearch,
}: MobileDrawerProps) {
  const initialPanel = mobileNavigationPanels.find((panel) => panel.label === "Resources")!;
  const [panelId, setPanelId] = useState(initialPanel.id);
  const [variantId, setVariantId] = useState(initialPanel.defaultVariant);
  const [query, setQuery] = useState("");
  const panel = mobileNavigationPanels.find((item) => item.id === panelId)!;
  const variant = panel.variants.find((item) => item.id === variantId);
  const groups = navigationGroups(variant?.entries ?? []);

  function search(value: string) {
    onSearch(value);
    onClose();
  }

  function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    search(query);
  }

  return (
    <aside
      id="mobile-navigation"
      className={`${styles.drawer} ${open ? styles.open : ""}`}
      data-theme={theme}
      aria-label="Mobile navigation"
      aria-hidden={!open}
      inert={!open}
    >
      <div className={styles.tabsRow}>
        <div className={styles.tabs} role="tablist" aria-label="Developer resources">
          {mobileNavigationPanels.map((item) => (
            <button
              key={item.id}
              type="button"
              role="tab"
              aria-selected={panelId === item.id}
              className={styles.tab}
              onClick={() => {
                setPanelId(item.id);
                setVariantId(item.defaultVariant);
              }}
            >
              {item.label}
            </button>
          ))}
        </div>
        <button
          type="button"
          className={styles.theme}
          aria-label={theme === "dark" ? "Switch to light theme" : "Switch to dark theme"}
          onClick={onToggleTheme}
        >
          {theme === "dark" ? <MoonIcon /> : <SunIcon />}
        </button>
      </div>
      <div className={styles.body}>
        <div className={styles.searchArea}>
          <form onSubmit={submit}>
            <input
              className={styles.search}
              type="search"
              placeholder="Start searching"
              aria-label="Search documentation"
              value={query}
              onChange={(event) => setQuery(event.target.value)}
            />
          </form>
          <div>
            <div className={styles.suggested}>Suggested</div>
            <div className={styles.suggestions}>
              {["responses create", "reasoning_effort", "realtime", "prompt caching"].map((suggestion) => (
                <button key={suggestion} type="button" onClick={() => search(suggestion)}>
                  {suggestion}
                </button>
              ))}
            </div>
          </div>
        </div>
        <nav aria-label={`${panel.label} navigation`}>
          <div className={styles.contexts}>
            {panel.label === "Resources" && (
              <a className={styles.contextLink} href="https://developers.openai.com/showcase" onClick={onClose}>
                Showcase
              </a>
            )}
            {panel.label === "ChatGPT" && (
              <a className={styles.contextLink} href="https://developers.openai.com/chatgpt" onClick={onClose}>
                Overview
              </a>
            )}
            {panel.options.map((option) => {
              const entries = panel.variants.find((item) => item.id === option.id)?.entries;
              const className = `${styles.contextLink} ${panel.label === "ChatGPT" ? "" : styles.contextButton}`;
              return entries?.length ? (
                <button
                  key={option.id}
                  type="button"
                  className={className}
                  aria-pressed={variantId === option.id}
                  onClick={() => setVariantId(option.id)}
                >
                  {option.label}
                </button>
              ) : (
                <a key={option.id} className={className} href={destination(option.href)} onClick={onClose}>
                  {option.label}
                </a>
              );
            })}
            {panel.label === "ChatGPT" && (
              <div className={styles.externalResources}>
                <a href="https://learn.chatgpt.com/docs" onClick={onClose}>
                  ChatGPT + Codex user docs <ExternalLinkIcon />
                </a>
                <a href="https://learn.chatgpt.com/use-cases" onClick={onClose}>
                  Use cases <ExternalLinkIcon />
                </a>
              </div>
            )}
          </div>
          {groups.length > 0 && (
            <div className={`${styles.navigationGroups} ${panel.label === "API" ? styles.apiGroups : ""}`}>
              {groups.map((group, groupIndex) => (
                <section key={`${variantId}-${groupIndex}`} className={styles.group}>
                  {group.heading && <h2 className={styles.heading}>{group.heading}</h2>}
                  <div className={styles.links}>
                    {group.entries.map((entry, entryIndex) => (
                      <a
                        key={`${entry.text}-${entryIndex}`}
                        href={destination(entry.href ?? "#")}
                        onClick={onClose}
                        className={`${styles.link} ${entry.active ? styles.active : ""}`}
                        aria-current={entry.active ? "page" : undefined}
                      >
                        {entry.text}
                        {entry.classes.includes("justify-between") && <ExternalLinkIcon />}
                      </a>
                    ))}
                  </div>
                </section>
              ))}
            </div>
          )}
        </nav>
      </div>
      <footer className={styles.footer}>
        <a href="https://platform.openai.com" onClick={onClose}>
          API Dashboard <ExternalLinkIcon />
        </a>
      </footer>
    </aside>
  );
}
