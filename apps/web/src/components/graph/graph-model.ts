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

/** Arrange connected groups independently so isolated sources cannot stretch the view. */
export function arrangeGraph(graph: RenderGraph) {
  if (!graph.order) return;
  const ids = graph.nodes().sort();
  const remaining = new Set(ids);
  const groups: string[][] = [];
  for (const root of ids) {
    if (!remaining.delete(root)) continue;
    const group = [root];
    for (let i = 0; i < group.length; i++) {
      for (const neighbor of graph.neighbors(group[i]).sort()) {
        if (remaining.delete(neighbor)) group.push(neighbor);
      }
    }
    groups.push(group.sort());
  }
  const boxes = groups.map((group) => {
    const layout = new UndirectedGraph<NodeAttributes>();
    const hub = group.length > 2 ? [...group].sort((a, b) => graph.neighbors(b).length - graph.neighbors(a).length || a.localeCompare(b))[0] : null;
    let index = 0;
    // Start the busiest source in the center to balance radial neighborhoods.
    group.forEach((id) => {
      const angle = 2 * Math.PI * index / (group.length - (hub ? 1 : 0));
      const radius = id === hub ? 0 : Math.max(40, Math.sqrt(group.length) * 30);
      layout.addNode(id, { ...graph.getNodeAttributes(id), x: Math.cos(angle) * radius, y: Math.sin(angle) * radius, size: 18 });
      if (id !== hub) index++;
    });
    // Repeated paper evidence must not strengthen attraction between endpoints.
    graph.forEachEdge((_id, _attributes, source, target) => {
      if (source !== target && layout.hasNode(source) && layout.hasNode(target) && !layout.hasEdge(source, target)) layout.addEdge(source, target);
    });
    if (layout.order > 1 && layout.size) {
      forceAtlas2.assign(layout, { iterations: 350, settings: { ...forceAtlas2.inferSettings(layout), adjustSizes: true, barnesHutOptimize: false, scalingRatio: 20, gravity: 0.5, strongGravityMode: true } });
    }
    const points = group.map((id) => ({ id, ...layout.getNodeAttributes(id) }));
    const minX = Math.min(...points.map((p) => p.x)), maxX = Math.max(...points.map((p) => p.x));
    const minY = Math.min(...points.map((p) => p.y)), maxY = Math.max(...points.map((p) => p.y));
    const extent = group.length > 1 ? Math.sqrt(group.length) * 100 : 0;
    const width = maxX - minX, height = maxY - minY;
    const scale = extent / Math.max(width, height, 1);
    // Give long, thin connected groups enough room in both axes.
    const scaleX = group.length > 2 ? Math.max(scale, extent * 0.6 / Math.max(width, 1)) : scale;
    const scaleY = group.length > 2 ? Math.max(scale, extent * 0.6 / Math.max(height, 1)) : scale;
    for (const point of points) {
      point.x = (point.x - (minX + maxX) / 2) * scaleX;
      point.y = (point.y - (minY + maxY) / 2) * scaleY;
    }
    return { points, width: Math.max(90, width * scaleX + 100), height: Math.max(90, height * scaleY + 100) };
  }).sort((a, b) => b.points.length - a.points.length || a.points[0].id.localeCompare(b.points[0].id));
  const rowWidth = Math.max(...boxes.map((b) => b.width), Math.sqrt(boxes.reduce((sum, b) => sum + b.width * b.height, 0)) * 1.3);
  let cursorX = 0, cursorY = 0, rowHeight = 0;
  for (const box of boxes) {
    if (cursorX > 0 && cursorX + box.width > rowWidth) { cursorX = 0; cursorY += rowHeight + 80; rowHeight = 0; }
    for (const point of box.points) graph.mergeNodeAttributes(point.id, { x: point.x + cursorX + box.width / 2, y: point.y + cursorY + box.height / 2 });
    cursorX += box.width + 80;
    rowHeight = Math.max(rowHeight, box.height);
  }
  // Enforce breathing room even for dense shared-feature hubs.
  for (let pass = 0; pass < 150; pass++) {
    let moved = false;
    for (let i = 0; i < ids.length; i++) for (let j = i + 1; j < ids.length; j++) {
      const a = graph.getNodeAttributes(ids[i]), b = graph.getNodeAttributes(ids[j]);
      const dx = b.x - a.x, dy = b.y - a.y;
      const distance = Math.hypot(dx, dy);
      if (distance >= 85) continue;
      const angle = (i + j) * 2.399963;
      const ux = distance > 0.001 ? dx / distance : Math.cos(angle);
      const uy = distance > 0.001 ? dy / distance : Math.sin(angle);
      const shift = (85 - distance) / 2 + 0.01;
      graph.mergeNodeAttributes(ids[i], { x: a.x - ux * shift, y: a.y - uy * shift });
      graph.mergeNodeAttributes(ids[j], { x: b.x + ux * shift, y: b.y + uy * shift });
      moved = true;
    }
    if (!moved) break;
  }
  const coordinates = ids.map((id) => graph.getNodeAttributes(id));
  const centerX = (Math.min(...coordinates.map((p) => p.x)) + Math.max(...coordinates.map((p) => p.x))) / 2;
  const centerY = (Math.min(...coordinates.map((p) => p.y)) + Math.max(...coordinates.map((p) => p.y))) / 2;
  graph.forEachNode((id, { x, y }) => graph.mergeNodeAttributes(id, { x: x - centerX, y: y - centerY }));
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
