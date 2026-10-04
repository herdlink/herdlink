"use client";

import dynamic from "next/dynamic";
import { useState } from "react";
import { Network, RefreshCw, Search } from "lucide-react";
import { useGraphChat } from "@/components/chat/GraphChatProvider";
import { graphComplexities, sourceTypes, type GraphSelection, type GraphSnapshot } from "@/lib/graph-types";
import { CreateSurveyButton } from "@/components/surveys/CreateSurvey";
import { graphSurveyTargets } from "@/lib/survey-types";
import { GraphDetails } from "./GraphDetails";

const SigmaCanvas = dynamic(() => import("./SigmaCanvas").then((module) => module.SigmaCanvas), { ssr: false, loading: () => <p role="status" className="p-6 text-sm text-[var(--site-secondary)]">Starting graph…</p> });

/** Live conversation snapshots share the same stable graph model as disease search. */
export function GraphExplorer({ snapshot: suppliedSnapshot }: { snapshot?: GraphSnapshot }) {
  const chat = useGraphChat();
  const [draft, setDraft] = useState("");
  const current = suppliedSnapshot ?? chat.graph;
  const query = current.query;
  const loading = chat.searching;
  const error = chat.graphError;
  const [inspection, setInspection] = useState<{ selection: GraphSelection; pinned: boolean } | null>(null);
  const [hiddenKinds, setHiddenKinds] = useState<string[]>([]);
  const selection = inspection?.selection ?? null;

  function refresh(nextQuery = query) {
    setInspection(null); setHiddenKinds([]);
    void chat.searchDisease(nextQuery);
  }
  const types = [...new Set(current?.nodes.map((node) => node.kind) ?? [])];
  const visibleNodes = current?.nodes.filter((node) => !hiddenKinds.includes(node.kind)) ?? [];
  const visibleIds = new Set(visibleNodes.map((node) => node.id));
  const visibleEdges = current?.edges.filter((edge) => visibleIds.has(edge.source) && visibleIds.has(edge.target)) ?? [];
  function isVisible(value: GraphSelection) {
    return value.kind === "node" ? visibleIds.has(value.id) : visibleEdges.some((edge) => edge.id === value.id);
  }
  const visibleSelection = selection && isVisible(selection) ? selection : null;
  const complexityIndex = graphComplexities.findIndex((level) => level.value === chat.complexity);
  const complexityLevel = graphComplexities[complexityIndex];
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
        <div className="flex flex-wrap items-center gap-2">
        {!suppliedSnapshot && <div className="flex items-center gap-2 rounded-full border border-[var(--site-border)] px-3 py-2">
          <label htmlFor="graph-complexity" className="text-xs text-[var(--site-secondary)]">Complexity</label>
          <input id="graph-complexity" type="range" min={0} max={2} step={1} value={complexityIndex} aria-label="Graph complexity" aria-valuetext={`${complexityLevel.label}, up to ${complexityLevel.sources} sources`} title="Reveal more stored connections. New searches start focused." disabled={loading || chat.busy || !current.nodes.length} onChange={(event) => { void chat.changeComplexity(graphComplexities[Number(event.target.value)].value); }} className="h-5 w-24 cursor-pointer accent-[var(--site-solid)] disabled:cursor-wait disabled:opacity-40" />
          <span aria-live="polite" className="min-w-14 text-[11px]">{complexityLevel.label}</span>
        </div>}
        <CreateSurveyButton targets={graphSurveyTargets(current)} />
        {!suppliedSnapshot && !chat.activeId && <button type="button" onClick={() => refresh()} disabled={loading || !query} className="flex items-center gap-2 rounded-full border border-[var(--site-border)] px-3 py-2 text-xs hover:bg-[var(--site-hover)] disabled:opacity-40"><RefreshCw size={13} />Refresh graph</button>}</div>
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
      {chat.update && <p role="status" className="shrink-0 border-b border-[var(--site-border)] bg-[var(--site-soft)] px-5 py-2 text-[11px] text-[var(--site-secondary)]">{chat.update}</p>}
      {current?.truncated && <p className="shrink-0 bg-[var(--site-soft)] px-5 py-2 text-[11px] text-[var(--site-secondary)]">{complexityLevel.label} overview: up to {complexityLevel.sources} sources, {complexityLevel.matches} phenotype matches and {complexityLevel.phenotypes} shared features. Full comparisons and literature are available in the details panel.</p>}
      {error && <div role="alert" className="shrink-0 border-b border-[var(--site-border)] px-5 py-3 text-xs leading-5"><p>{error}</p><button onClick={() => { if (chat.activeId) void chat.openChat(chat.activeId); else refresh(); }} className="mt-1 underline">Try again</button></div>}
      <div className="relative min-h-[380px] flex-1 lg:min-h-0">
        {current && current.nodes.length > 0 && <SigmaCanvas snapshot={current} hiddenKinds={hiddenKinds} selection={visibleSelection} onHover={preview} onSelect={select} complexity={chat.complexity} />}
        {!suppliedSnapshot && loading && <p role="status" className="absolute bottom-4 right-4 rounded-full bg-[var(--site-panel)] px-3 py-2 text-xs text-[var(--site-secondary)]">Loading graph…</p>}
        {!loading && !error && current?.nodes.length === 0 && <div className="absolute inset-0 flex flex-col items-center justify-center px-8 text-center"><Network size={30} strokeWidth={1.3} className="mb-5 text-[var(--site-secondary)]" /><h2 className="text-lg font-medium">{query ? "No matching disease" : "Find a disease"}</h2><p className="mt-3 max-w-xs text-sm leading-6 text-[var(--site-secondary)]">{query ? "Try another disease name or identifier." : "Search for a disease to start your graph."}</p></div>}
        {visibleSelection && current && <GraphDetails selection={visibleSelection} snapshot={current} pinned={inspection?.pinned ?? false} onTogglePin={() => setInspection((current) => current ? { ...current, pinned: !current.pinned } : null)} onClose={() => select(null)} />}
      </div>
      {current && current.nodes.length > 0 && <div className="shrink-0 border-t border-[var(--site-border)] px-5 py-3">
        {current.edges.length > 0 && <div aria-label="Relationship legend" className="mb-2 flex flex-wrap gap-x-4 gap-y-1 text-[10px] text-[var(--site-secondary)]"><span className="flex items-center gap-1.5"><span className="h-0.5 w-4 bg-[#d49b29]" />Phenotypes / similarity</span><span className="flex items-center gap-1.5"><span className="h-0.5 w-4 bg-[#169d93]" />Reported associations</span><span className="flex items-center gap-1.5"><span className="h-0.5 w-4 bg-[#368c75]" />Extracted relationships</span><span className="flex items-center gap-1.5"><span className="h-0.5 w-4 bg-[#9571d9]" />Literature</span></div>}
        <p className="mb-2 text-[11px] text-[var(--site-secondary)]">Hover to inspect · Click to pin details · Scroll to zoom</p>
        <details className="text-xs"><summary className="cursor-pointer">Browse sources and relationships</summary><div className="mt-3 max-h-48 overflow-y-auto">
          <div className="flex flex-wrap gap-2">{visibleNodes.map((node) => <button key={node.id} type="button" onClick={() => select({ kind: "node", id: node.id })} className="rounded-full border border-[var(--site-border)] px-3 py-1.5 text-left hover:bg-[var(--site-hover)]">{node.label}</button>)}</div>
          <ul className="mt-3 space-y-2">{visibleEdges.map((edge) => <li key={edge.id}><button type="button" onClick={() => select({ kind: "edge", id: edge.id })} className="text-left text-[var(--site-secondary)] hover:underline">{current.nodes.find((n) => n.id === edge.source)?.label} → {edge.label} → {current.nodes.find((n) => n.id === edge.target)?.label}</button></li>)}</ul>
        </div></details>
      </div>}
    </section>
  );
}
