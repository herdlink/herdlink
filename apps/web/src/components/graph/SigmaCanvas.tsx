"use client";

import { useEffect, useRef, useState } from "react";
import { MultiDirectedGraph } from "graphology";
import Sigma from "sigma";
import { createEdgeCurveProgram } from "@sigma/edge-curve";
import { Maximize2, Minus, Plus, Shuffle } from "lucide-react";
import { type GraphComplexity, type GraphSelection, type GraphSnapshot } from "@/lib/graph-types";
import { arrangeGraph, updateGraph, type NodeAttributes, type EdgeAttributes, type RenderGraph } from "./graph-model";

const CurvedArrowProgram = createEdgeCurveProgram<NodeAttributes, EdgeAttributes>({ arrowHead: { extremity: "target", lengthToThicknessRatio: 2.5, widenessToThicknessRatio: 2 } });

export function SigmaCanvas({ snapshot, hiddenKinds, selection, onHover, onSelect, complexity = "focused" }: { snapshot: GraphSnapshot; hiddenKinds: string[]; selection: GraphSelection | null; onHover: (selection: GraphSelection) => void; onSelect: (selection: GraphSelection | null) => void; complexity?: GraphComplexity }) {
  const containerRef = useRef<HTMLDivElement>(null);
  const rendererRef = useRef<Sigma<NodeAttributes, EdgeAttributes> | null>(null);
  const graphRef = useRef<RenderGraph | null>(null);
  const inspectRef = useRef({ onHover, onSelect });
  const [error, setError] = useState("");
  useEffect(() => { inspectRef.current = { onHover, onSelect }; }, [onHover, onSelect]);

  useEffect(() => {
    if (!containerRef.current) return;
    let renderer: Sigma<NodeAttributes, EdgeAttributes>;
    try {
      const graph = new MultiDirectedGraph<NodeAttributes, EdgeAttributes>();
      graphRef.current = graph;
      renderer = new Sigma<NodeAttributes, EdgeAttributes>(graph, containerRef.current, {
        enableEdgeEvents: true, renderEdgeLabels: true, labelSize: 12,
        labelRenderedSizeThreshold: 5, labelDensity: 0.6, labelGridCellSize: 120, edgeLabelSize: 10,
        stagePadding: 75, minCameraRatio: 0.08, maxCameraRatio: 8,
        defaultEdgeType: "arrow", allowInvalidContainer: true,
        edgeProgramClasses: { curved: CurvedArrowProgram },
      });
      rendererRef.current = renderer;
      const selectNode = ({ node }: { node: string }) => inspectRef.current.onSelect({ kind: "node", id: node });
      const selectEdge = ({ edge }: { edge: string }) => inspectRef.current.onSelect({ kind: "edge", id: edge });
      renderer.on("enterNode", ({ node }) => inspectRef.current.onHover({ kind: "node", id: node }));
      renderer.on("clickNode", selectNode);
      renderer.on("doubleClickNode", selectNode);
      renderer.on("enterEdge", ({ edge }) => inspectRef.current.onHover({ kind: "edge", id: edge }));
      renderer.on("clickEdge", selectEdge);
      renderer.on("doubleClickEdge", selectEdge);
      renderer.on("clickStage", () => inspectRef.current.onSelect(null));
    } catch {
      queueMicrotask(() => setError("The graph renderer needs WebGL. You can still browse every source and relationship in the list below."));
      return;
    }
    function updateTheme() {
      const dark = document.documentElement.dataset.siteTheme === "dark";
      renderer.setSettings({ labelColor: { color: dark ? "#e3e3e3" : "#343434" }, edgeLabelColor: { color: dark ? "#b9b9b9" : "#656565" }, defaultEdgeColor: dark ? "#626975" : "#bac3ce" });
    }
    updateTheme();
    const observer = new MutationObserver(updateTheme);
    observer.observe(document.documentElement, { attributes: true, attributeFilter: ["data-site-theme"] });
    const resizeObserver = new ResizeObserver(() => { renderer.resize(); renderer.refresh(); });
    resizeObserver.observe(containerRef.current);
    return () => { observer.disconnect(); resizeObserver.disconnect(); renderer.kill(); rendererRef.current = null; graphRef.current = null; };
  }, []);

  useEffect(() => {
    if (!graphRef.current) return;
    updateGraph(graphRef.current, snapshot);
    rendererRef.current?.refresh();
  }, [snapshot]);

  useEffect(() => { rendererRef.current?.getCamera().animatedReset({ duration: 200 }); }, [complexity]);

  useEffect(() => {
    const renderer = rendererRef.current;
    const graph = graphRef.current;
    if (!renderer || !graph) return;
    const hidden = new Set(hiddenKinds);
    const focus = selection?.kind === "node" && graph.hasNode(selection.id) ? selection.id : null;
    const neighbors = new Set(focus ? graph.neighbors(focus) : []);
    const labeledEdges = new Set(focus ? graph.edges(focus).filter((id) => ["PHENOTYPE_SIMILARITY", "PUBTATOR_RELATION", "EXTRACTED_RELATION"].includes(graph.getEdgeAttribute(id, "kind"))).slice(0, 3) : []);
    renderer.setSetting("nodeReducer", (id, data) => ({ ...data, label: data.label.length > 32 ? `${data.label.slice(0, 31)}…` : data.label, hidden: hidden.has(data.kind), color: focus && id !== focus && !neighbors.has(id) ? "#a4aab3" : data.color, highlighted: id === focus, forceLabel: id === focus }));
    renderer.setSetting("edgeReducer", (id, data) => {
      const source = graph.source(id), target = graph.target(id);
      const selected = selection?.kind === "edge" && selection.id === id;
      const edgeColor = document.documentElement.dataset.siteTheme === "dark" ? "#626975" : "#bac3ce";
      return { ...data, label: selected || labeledEdges.has(id) ? data.label : "", hidden: hidden.has(graph.getNodeAttribute(source, "kind")) || hidden.has(graph.getNodeAttribute(target, "kind")), color: selected ? "#4f7ff0" : data.color === "#bac3ce" ? edgeColor : data.color, size: selected ? 3 : data.size, forceLabel: selected || labeledEdges.has(id) };
    });
    renderer.refresh();
  }, [selection, hiddenKinds, snapshot]);

  return (
    <div className="absolute inset-0">
      <div ref={containerRef} aria-label="Interactive source graph" className="absolute inset-0 overflow-hidden" />
      {error && <p role="alert" className="absolute bottom-14 left-4 right-4 rounded-xl border border-[var(--site-border)] bg-[var(--site-panel)] p-4 text-sm">{error}</p>}
      <div className="absolute bottom-4 left-4 flex rounded-full border border-[var(--site-border)] bg-[var(--site-panel)] p-1 shadow-sm">
        <button type="button" aria-label="Zoom in" onClick={() => rendererRef.current?.getCamera().animatedZoom({ duration: 200 })} className="flex h-8 w-8 items-center justify-center rounded-full hover:bg-[var(--site-hover)]"><Plus size={16} /></button>
        <button type="button" aria-label="Zoom out" onClick={() => rendererRef.current?.getCamera().animatedUnzoom({ duration: 200 })} className="flex h-8 w-8 items-center justify-center rounded-full hover:bg-[var(--site-hover)]"><Minus size={16} /></button>
        <button type="button" aria-label="Rearrange graph" title="Spread nodes out" onClick={() => { if (graphRef.current) arrangeGraph(graphRef.current); rendererRef.current?.refresh(); rendererRef.current?.getCamera().animatedReset({ duration: 200 }); }} className="flex h-8 w-8 items-center justify-center rounded-full hover:bg-[var(--site-hover)]"><Shuffle size={15} /></button>
        <button type="button" aria-label="Fit graph" onClick={() => rendererRef.current?.getCamera().animatedReset({ duration: 200 })} className="flex h-8 w-8 items-center justify-center rounded-full hover:bg-[var(--site-hover)]"><Maximize2 size={15} /></button>
      </div>
    </div>
  );
}
