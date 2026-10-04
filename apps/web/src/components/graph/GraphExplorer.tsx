"use client";

import dynamic from "next/dynamic";
import { useEffect, useState } from "react";
import { Network, RefreshCw, Search } from "lucide-react";
import { communityApi, errorMessage } from "@/lib/community-api";
import { sourceTypes, type GraphSelection, type GraphSnapshot } from "@/lib/graph-types";
import { GraphDetails } from "./GraphDetails";

const SigmaCanvas = dynamic(() => import("./SigmaCanvas").then((module) => module.SigmaCanvas), { ssr: false, loading: () => <p role="status" className="p-6 text-sm text-[var(--site-secondary)]">Starting graph…</p> });
const emptySnapshot: GraphSnapshot = { nodes: [], edges: [], query: "", result_uid: "", truncated: false, generated_at: "" };

/** A tool-result snapshot can be supplied by the future chat/stream integration. */
export function GraphExplorer({ snapshot: suppliedSnapshot }: { snapshot?: GraphSnapshot }) {
  const [snapshot, setSnapshot] = useState<GraphSnapshot | null>(null);
  const [draft, setDraft] = useState("");
  const [query, setQuery] = useState("");
  const [version, setVersion] = useState(0);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [inspection, setInspection] = useState<{ selection: GraphSelection; pinned: boolean } | null>(null);
  const [hiddenKinds, setHiddenKinds] = useState<string[]>([]);
  const current = suppliedSnapshot ?? (query ? snapshot : emptySnapshot);
  const selection = inspection?.selection ?? null;

  useEffect(() => {
    if (suppliedSnapshot || !query) return;
    let cancelled = false;
    const params = new URLSearchParams({ q: query });
    communityApi<GraphSnapshot>(`/graph?${params}`).then((data) => {
      if (!cancelled) { setSnapshot(data); setError(""); }
    }).catch((error) => { if (!cancelled) setError(errorMessage(error)); }).finally(() => { if (!cancelled) setLoading(false); });
    return () => { cancelled = true; };
  }, [query, version, suppliedSnapshot]);

  function refresh(nextQuery = query) {
    setLoading(Boolean(nextQuery)); setError(""); setInspection(null); setSnapshot(null); setHiddenKinds([]);
    setQuery(nextQuery); setVersion((value) => value + 1);
  }
  const types = [...new Set(current?.nodes.map((node) => node.kind) ?? [])];
  const visibleNodes = current?.nodes.filter((node) => !hiddenKinds.includes(node.kind)) ?? [];
  const visibleIds = new Set(visibleNodes.map((node) => node.id));
  const visibleEdges = current?.edges.filter((edge) => visibleIds.has(edge.source) && visibleIds.has(edge.target)) ?? [];
  function isVisible(value: GraphSelection) {
    return value.kind === "node" ? visibleIds.has(value.id) : visibleEdges.some((edge) => edge.id === value.id);
  }
  const visibleSelection = selection && isVisible(selection) ? selection : null;
  function preview(next: GraphSelection) {
    setInspection((previous) => previous?.pinned && isVisible(previous.selection) ? previous : { selection: next, pinned: false });
  }
  function select(next: GraphSelection | null) {
    setInspection(next ? { selection: next, pinned: true } : null);
  }

  return (
    <section aria-labelledby="graph-heading" className="flex h-full min-h-[640px] min-w-0 flex-col lg:min-h-0">
      <header className="flex min-h-16 shrink-0 flex-wrap items-center justify-between gap-3 border-b border-[var(--site-border)] px-5 py-4">
        <div><h1 id="graph-heading" className="text-sm font-semibold">Source graph</h1><p className="mt-1 text-[11px] text-[var(--site-secondary)]">{current?.nodes.length ? `${current.nodes.length} ${current.nodes.length === 1 ? "source" : "sources"} · ${current.edges.length} ${current.edges.length === 1 ? "connection" : "connections"}` : "Search for a disease to start"}</p></div>
        {!suppliedSnapshot && <button type="button" onClick={() => refresh()} disabled={loading || !query} className="flex items-center gap-2 rounded-full border border-[var(--site-border)] px-3 py-2 text-xs hover:bg-[var(--site-hover)] disabled:opacity-40"><RefreshCw size={13} />Refresh graph</button>}
      </header>
      {!suppliedSnapshot && <form onSubmit={(event) => { event.preventDefault(); refresh(draft.trim()); }} className="flex shrink-0 items-center gap-2 border-b border-[var(--site-border)] px-5 py-3">
        <Search size={15} className="shrink-0 text-[var(--site-secondary)]" />
        <input aria-label="Find a disease" value={draft} onChange={(event) => setDraft(event.target.value)} maxLength={200} placeholder="Search for a disease…" className="min-w-0 flex-1 bg-transparent text-sm outline-none" />
        <button type="submit" disabled={loading} className="rounded-full bg-[var(--site-solid)] px-3 py-1.5 text-xs text-[var(--site-solid-text)] disabled:opacity-40">Search</button>
      </form>}
      {types.length > 0 && <div aria-label="Source type filters" className="flex shrink-0 flex-wrap gap-1 border-b border-[var(--site-border)] px-4 py-2">
        {types.map((kind) => {
          const style = sourceTypes[kind] ?? sourceTypes.other;
          return <button key={kind} type="button" aria-pressed={!hiddenKinds.includes(kind)} onClick={() => { setInspection(null); setHiddenKinds((current) => current.includes(kind) ? current.filter((value) => value !== kind) : [...current, kind]); }} className={`flex items-center gap-1.5 rounded-full px-2 py-1 text-[11px] hover:bg-[var(--site-hover)] ${hiddenKinds.includes(kind) ? "opacity-40" : ""}`}><span className={`h-2 w-2 rounded-full ${style.dotClass}`} />{style.label}</button>;
        })}
      </div>}
      {current?.truncated && <p className="shrink-0 bg-[var(--site-soft)] px-5 py-2 text-[11px] text-[var(--site-secondary)]">Showing a bounded overview. Narrow the topic to see more relevant sources. Literature is paginated separately.</p>}
      {error && <div role="alert" className="shrink-0 border-b border-[var(--site-border)] px-5 py-3 text-xs leading-5"><p>{error}</p><button onClick={() => refresh()} className="mt-1 underline">Try again</button></div>}
      <div className="relative min-h-[380px] flex-1 lg:min-h-0">
        {current && current.nodes.length > 0 && <SigmaCanvas snapshot={current} hiddenKinds={hiddenKinds} selection={visibleSelection} onHover={preview} onSelect={select} />}
        {!suppliedSnapshot && loading && <p role="status" className="absolute bottom-4 right-4 rounded-full bg-[var(--site-panel)] px-3 py-2 text-xs text-[var(--site-secondary)]">Loading graph…</p>}
        {!loading && !error && current?.nodes.length === 0 && <div className="absolute inset-0 flex flex-col items-center justify-center px-8 text-center"><Network size={30} strokeWidth={1.3} className="mb-5 text-[var(--site-secondary)]" /><h2 className="text-lg font-medium">{query ? "No matching disease" : "Find a disease"}</h2><p className="mt-3 max-w-xs text-sm leading-6 text-[var(--site-secondary)]">{query ? "Try another disease name or identifier." : "Search for a disease to start your graph."}</p></div>}
        {visibleSelection && current && <GraphDetails selection={visibleSelection} snapshot={current} pinned={inspection?.pinned ?? false} onTogglePin={() => setInspection((current) => current ? { ...current, pinned: !current.pinned } : null)} onClose={() => select(null)} />}
      </div>
      {current && current.nodes.length > 0 && <div className="shrink-0 border-t border-[var(--site-border)] px-5 py-3">
        <p className="mb-2 text-[11px] text-[var(--site-secondary)]">Hover to inspect · Click to pin details · Scroll to zoom</p>
        <details className="text-xs"><summary className="cursor-pointer">Browse sources and relationships</summary><div className="mt-3 max-h-48 overflow-y-auto">
          <div className="flex flex-wrap gap-2">{visibleNodes.map((node) => <button key={node.id} type="button" onClick={() => select({ kind: "node", id: node.id })} className="rounded-full border border-[var(--site-border)] px-3 py-1.5 text-left hover:bg-[var(--site-hover)]">{node.label}</button>)}</div>
          <ul className="mt-3 space-y-2">{visibleEdges.map((edge) => <li key={edge.id}><button type="button" onClick={() => select({ kind: "edge", id: edge.id })} className="text-left text-[var(--site-secondary)] hover:underline">{current.nodes.find((n) => n.id === edge.source)?.label} → {edge.label} → {current.nodes.find((n) => n.id === edge.target)?.label}</button></li>)}</ul>
        </div></details>
      </div>}
    </section>
  );
}
