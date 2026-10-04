import { MultiDirectedGraph } from "graphology";
import forceAtlas2 from "graphology-layout-forceatlas2";
import { sourceTypes, type GraphSnapshot } from "@/lib/graph-types";

export type NodeAttributes = { x: number; y: number; label: string; size: number; color: string; kind: string };
export type EdgeAttributes = { label: string; kind: string; size: number; color: string; type: string; curvature?: number };
export type RenderGraph = MultiDirectedGraph<NodeAttributes, EdgeAttributes>;

function position(id: string) {
  const seed = Array.from(id).reduce((value, character) => Math.imul(value ^ character.charCodeAt(0), 16777619) >>> 0, 2166136261);
  const angle = (seed % 6283) / 1000;
  const radius = 2 + (seed % 100) / 10;
  return { x: Math.cos(angle) * radius, y: Math.sin(angle) * radius };
}

/** Reconcile in place so Sigma, the camera, and existing node coordinates survive updates. */
export function updateGraph(graph: RenderGraph, snapshot: GraphSnapshot) {
  const wasEmpty = graph.order === 0;
  const ids = new Set(snapshot.nodes.map((node) => node.id));
  graph.forEachNode((id) => { if (!ids.has(id)) graph.dropNode(id); });
  graph.clearEdges();
  for (const node of snapshot.nodes) {
    const attributes = { label: node.label, kind: node.kind, size: node.kind === "disease" ? 10 : node.kind === "publication" ? 5 : 7, color: (sourceTypes[node.kind] ?? sourceTypes.other).color };
    if (graph.hasNode(node.id)) graph.mergeNodeAttributes(node.id, attributes);
    else graph.addNode(node.id, { ...position(node.id), ...attributes });
  }
  for (const edge of snapshot.edges) {
    if (!graph.hasNode(edge.source) || !graph.hasNode(edge.target) || graph.hasEdge(edge.id)) continue;
    graph.addDirectedEdgeWithKey(edge.id, edge.source, edge.target, { label: edge.label, kind: edge.kind, size: edge.kind === "PUBTATOR_RELATION" ? 1.7 : 1, color: "#bac3ce", type: "arrow" });
  }
  // Separate evidence between the same endpoints, including opposite directions.
  const parallel = new Map<string, string[]>();
  graph.forEachEdge((id, _attributes, source, target) => {
    const key = JSON.stringify([source, target].sort());
    parallel.set(key, [...(parallel.get(key) ?? []), id]);
  });
  for (const edges of parallel.values()) {
    if (edges.length < 2) continue;
    edges.sort();
    edges.forEach((id, index) => {
      const offset = (index - (edges.length - 1) / 2) * Math.min(0.32, 1.5 / edges.length);
      if (offset === 0) return;
      const direction = graph.source(id) < graph.target(id) ? 1 : -1;
      graph.mergeEdgeAttributes(id, { type: "curved", curvature: offset * direction });
    });
  }
  if (wasEmpty && graph.order > 1 && graph.size > 0) {
    forceAtlas2.assign(graph, { iterations: 100, settings: { ...forceAtlas2.inferSettings(graph), barnesHutOptimize: true, scalingRatio: 12, gravity: 0.8 } });
  }
}
