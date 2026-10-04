import { MultiDirectedGraph, UndirectedGraph } from "graphology";
import forceAtlas2 from "graphology-layout-forceatlas2";
import { sourceTypes, relationshipTypes, type GraphSnapshot } from "@/lib/graph-types";

export type NodeAttributes = { x: number; y: number; label: string; size: number; color: string; kind: string };
export type EdgeAttributes = { label: string; kind: string; size: number; color: string; type: string; curvature?: number };
export type RenderGraph = MultiDirectedGraph<NodeAttributes, EdgeAttributes>;

function position(id: string) {
  const seed = Array.from(id).reduce((value, character) => Math.imul(value ^ character.charCodeAt(0), 16777619) >>> 0, 2166136261);
  const angle = (seed % 6283) / 1000;
  const radius = 2 + (seed % 100) / 10;
  return { x: Math.cos(angle) * radius, y: Math.sin(angle) * radius };
}

/** Layout a simple graph so repeated paper evidence cannot pull endpoints together. */
export function arrangeGraph(graph: RenderGraph) {
  const layout = new UndirectedGraph<NodeAttributes>();
  const ids = graph.nodes().sort();
  ids.forEach((id, index) => {
    const angle = 2 * Math.PI * index / ids.length;
    const radius = Math.max(40, Math.sqrt(ids.length) * 30);
    layout.addNode(id, { ...graph.getNodeAttributes(id), x: Math.cos(angle) * radius, y: Math.sin(angle) * radius, size: 18 });
  });
  graph.forEachEdge((_id, _attributes, source, target) => {
    if (source !== target && !layout.hasEdge(source, target)) layout.addEdge(source, target);
  });
  if (layout.order > 1 && layout.size) {
    forceAtlas2.assign(layout, { iterations: 350, settings: { ...forceAtlas2.inferSettings(layout), adjustSizes: true, barnesHutOptimize: false, scalingRatio: 30, gravity: 0.15 } });
  }
  // Enforce breathing room even for disconnected nodes and dense shared-feature hubs.
  for (let pass = 0; pass < 100; pass++) {
    let moved = false;
    for (let i = 0; i < ids.length; i++) for (let j = i + 1; j < ids.length; j++) {
      const a = layout.getNodeAttributes(ids[i]), b = layout.getNodeAttributes(ids[j]);
      const dx = b.x - a.x, dy = b.y - a.y;
      const distance = Math.hypot(dx, dy);
      if (distance >= 65) continue;
      const angle = (i + j) * 2.399963;
      const ux = distance > 0.001 ? dx / distance : Math.cos(angle);
      const uy = distance > 0.001 ? dy / distance : Math.sin(angle);
      const shift = (65 - distance) / 2 + 0.01;
      layout.mergeNodeAttributes(ids[i], { x: a.x - ux * shift, y: a.y - uy * shift });
      layout.mergeNodeAttributes(ids[j], { x: b.x + ux * shift, y: b.y + uy * shift });
      moved = true;
    }
    if (!moved) break;
  }
  layout.forEachNode((id, { x, y }) => graph.mergeNodeAttributes(id, { x, y }));
}

/** Reconcile in place; metadata updates preserve positions and topology changes reflow. */
export function updateGraph(graph: RenderGraph, snapshot: GraphSnapshot) {
  const topology = (nodes: string[], edges: string[]) => JSON.stringify([nodes.sort(), edges.sort()]);
  const before = topology(graph.nodes(), graph.edges().map((id) => JSON.stringify([id, graph.source(id), graph.target(id)])));
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
    graph.addDirectedEdgeWithKey(edge.id, edge.source, edge.target, { label: edge.label, kind: edge.kind, size: edge.kind === "PUBTATOR_RELATION" || edge.kind === "PHENOTYPE_SIMILARITY" ? 1.7 : 1, color: relationshipTypes[edge.kind]?.color ?? "#bac3ce", type: "arrow" });
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
  const after = topology(graph.nodes(), graph.edges().map((id) => JSON.stringify([id, graph.source(id), graph.target(id)])));
  if (before !== after) arrangeGraph(graph);
}
