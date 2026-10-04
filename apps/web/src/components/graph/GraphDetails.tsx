"use client";

import Link from "next/link";
import { useEffect, useState } from "react";
import { ArrowUpRight, BookOpen, Pin, PinOff, X } from "lucide-react";
import { communityApi, errorMessage } from "@/lib/community-api";
import { sourceTypes, type GraphEvidence, type GraphNode, type GraphSelection, type GraphSnapshot, type SourceLink } from "@/lib/graph-types";

export function SourceLinks({ links }: { links: SourceLink[] }) {
  return <div className="flex flex-col gap-2">{links.filter((link) => /^https?:\/\//i.test(link.url)).map((link) => <a key={`${link.label}:${link.url}`} href={link.url} target="_blank" rel="noopener noreferrer" className="inline-flex items-center gap-1.5 text-xs underline decoration-[var(--site-border)] underline-offset-4 hover:decoration-current">{link.label}<ArrowUpRight size={12} /></a>)}</div>;
}

export function GraphDetails({ selection, snapshot, pinned, onTogglePin, onClose }: { selection: GraphSelection; snapshot: GraphSnapshot; pinned: boolean; onTogglePin: () => void; onClose: () => void }) {
  const node = selection.kind === "node" ? snapshot.nodes.find((n) => n.id === selection.id) : null;
  const edge = selection.kind === "edge" ? snapshot.edges.find((e) => e.id === selection.id) : null;
  if (!node && !edge) return null;
  const source = snapshot.nodes.find((n) => n.id === edge?.source);
  const target = snapshot.nodes.find((n) => n.id === edge?.target);
  return (
    <aside aria-label="Graph details" className="absolute right-4 top-4 z-10 max-h-[calc(100%-5rem)] w-[min(310px,calc(100%-2rem))] overflow-y-auto rounded-2xl border border-[var(--site-border)] bg-[var(--site-panel)] p-5 shadow-lg">
      <button type="button" aria-label={pinned ? "Unpin graph details" : "Pin graph details"} aria-pressed={pinned} title={pinned ? "Details pinned — click to unpin" : "Pin these details"} onClick={onTogglePin} className="absolute right-10 top-3 flex h-7 w-7 items-center justify-center rounded-full hover:bg-[var(--site-hover)]">{pinned ? <PinOff size={14} /> : <Pin size={14} />}</button>
      <button type="button" aria-label="Close graph details" onClick={onClose} className="absolute right-3 top-3 flex h-7 w-7 items-center justify-center rounded-full hover:bg-[var(--site-hover)]"><X size={14} /></button>
      <p className="mb-2 pr-14 text-[11px] uppercase tracking-wider text-[var(--site-secondary)]">{node ? (sourceTypes[node.kind] ?? sourceTypes.other).label : "Relationship evidence"}</p>
      <h2 className="break-words pr-4 text-lg font-medium leading-6">{node?.label ?? edge?.label.replaceAll('_', ' ')}</h2>
      {node && <>
        <p className="mb-4 mt-2 break-all text-[11px] text-[var(--site-secondary)]">{node.id}</p>
        {node.description && <p className="mb-4 line-clamp-5 text-xs leading-5 text-[var(--site-secondary)]">{node.description}</p>}
        {node.community_url?.startsWith("/community/") && <Link href={{ pathname: node.community_url, query: { name: Array.from(node.label).slice(0, 100).join("") } }} className="mb-4 inline-flex rounded-full bg-[var(--site-solid)] px-3 py-2 text-xs font-medium text-[var(--site-solid-text)]">Open disease community</Link>}
        <SourceLinks links={node.links} />
      </>}
      {edge && <>
        <p className="my-3 text-xs leading-5 text-[var(--site-secondary)]">{source?.label} → {target?.label}</p>
        {edge.reported_papers !== null && <p className="mb-3 text-xs">{edge.reported_papers.toLocaleString()} papers reported by the source</p>}
        <p className="mb-3 text-[11px] text-[var(--site-secondary)]">{edge.kind.replaceAll('_', ' ')}</p>
        {Object.keys(edge.context).length > 0 && <details className="mb-3 text-xs"><summary className="cursor-pointer text-[var(--site-secondary)]">Source evidence fields</summary><pre className="mt-2 max-h-36 overflow-auto whitespace-pre-wrap break-all rounded-lg bg-[var(--site-soft)] p-3 text-[10px]">{JSON.stringify(edge.context, null, 2)}</pre></details>}
        {source && target && <SourceLinks links={[{ label: "Search related literature", url: `https://pubmed.ncbi.nlm.nih.gov/?term=${encodeURIComponent(`(${source.label}) AND (${target.label})`)}` }]} />}
        {edge.kind === "PUBTATOR_RELATION" && source?.id.startsWith("pubtator:@") && target?.id.startsWith("pubtator:@") && <div className="mt-3"><SourceLinks links={[{ label: "View relation papers in PubTator", url: `https://www.ncbi.nlm.nih.gov/research/pubtator3/?query=${encodeURIComponent(`relations:${edge.label}|${source.id.slice(9)}|${target.id.slice(9)}`)}` }]} /></div>}
      </>}
      <EvidenceList key={`${selection.kind}:${selection.id}`} id={node?.id ?? edge!.evidence_id} kind={node ? "node" : edge!.evidence_kind} />
    </aside>
  );
}

function EvidenceList({ id, kind }: { id: string; kind: string }) {
  const [data, setData] = useState<GraphEvidence | null>(null);
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const [attempt, setAttempt] = useState(0);
  const path = `/graph/evidence?id=${encodeURIComponent(id)}&kind=${kind}`;
  useEffect(() => {
    let cancelled = false;
    // Brief delay avoids a request for every node passed while moving across the graph.
    const timer = setTimeout(() => {
      communityApi<GraphEvidence>(path).then((value) => { if (!cancelled) setData(value); }).catch((error) => { if (!cancelled) setError(errorMessage(error)); });
    }, 180);
    return () => { cancelled = true; clearTimeout(timer); };
  }, [path, attempt]);
  async function loadMore() {
    if (!data || busy) return;
    setBusy(true); setError("");
    try {
      const next = await communityApi<GraphEvidence>(`${path}&offset=${data.offset + 20}`);
      setData({ ...next, papers: [...data.papers, ...next.papers] });
    } catch (error) { setError(errorMessage(error)); }
    finally { setBusy(false); }
  }
  return <section className="mt-5 border-t border-[var(--site-border)] pt-4">
    <h3 className="mb-3 flex items-center gap-2 text-sm font-medium"><BookOpen size={14} />{kind === "node" ? "Associated literature" : "Supporting papers"}</h3>
    {error ? <div role="alert" className="text-xs leading-5"><p>{error}</p><button onClick={() => { setError(""); setAttempt((value) => value + 1); }} className="mt-2 underline">Retry literature</button></div> : !data ? <p role="status" className="text-xs text-[var(--site-secondary)]">Loading stored literature…</p> : <>
      <p className="mb-3 text-[11px] leading-5 text-[var(--site-secondary)]">{data.note}</p>
      {data.papers.length === 0 && <p className="text-xs text-[var(--site-secondary)]">No individual papers stored for this selection.</p>}
      <div className="space-y-4">{data.papers.map((paper: GraphNode) => <div key={paper.id}><p className="mb-2 text-xs font-medium leading-5">{paper.label}</p>{paper.description && <details className="mb-3 text-xs text-[var(--site-secondary)]"><summary className="cursor-pointer">Read abstract</summary><p className="mt-2 whitespace-pre-line leading-5">{paper.description}</p></details>}<SourceLinks links={paper.links} /></div>)}</div>
      {data.has_more && <button onClick={() => void loadMore()} disabled={busy} className="mt-4 rounded-full border border-[var(--site-border)] px-3 py-2 text-xs disabled:opacity-40">{busy ? "Loading…" : "Load more papers"}</button>}
    </>}
  </section>;
}
