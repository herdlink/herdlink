import { describe, expect, test } from "bun:test";
import { MultiDirectedGraph } from "graphology";
import { type GraphSnapshot, sourceTypes } from "@/lib/graph-types";
import { updateGraph, type NodeAttributes, type EdgeAttributes } from "./graph-model";

const node = (id: string, kind = "gene") => ({ id, label: id, kind, labels: [], description: null, links: [], community_url: null });
const edge = (id: string, source: string, target: string) => ({ id, source, target, label: "associate", kind: "PUBTATOR_RELATION", evidence_id: id, evidence_kind: "summary", reported_papers: 3, context: {} });
const snapshot = (nodes: GraphSnapshot["nodes"], edges: GraphSnapshot["edges"]): GraphSnapshot => ({ nodes, edges, query: "", result_uid: "", truncated: false, generated_at: "2026-10-04T00:00:00Z" });

describe("replaceable graph snapshots", () => {
  test("metadata updates preserve positions while snapshots reconcile sources and evidence", () => {
    const graph = new MultiDirectedGraph<NodeAttributes, EdgeAttributes>();
    updateGraph(graph, snapshot([node("disease", "disease"), node("gene")], [edge("association", "disease", "gene")]));
    const original = { x: graph.getNodeAttribute("disease", "x"), y: graph.getNodeAttribute("disease", "y") };
    updateGraph(graph, snapshot([{ ...node("disease", "disease"), label: "Updated disease name" }, node("gene")], [edge("association", "disease", "gene")]));
    expect({ x: graph.getNodeAttribute("disease", "x"), y: graph.getNodeAttribute("disease", "y") }).toEqual(original);
    updateGraph(graph, snapshot([{ ...node("disease", "disease"), label: "Updated disease name" }, node("paper", "publication")], [edge("new-evidence", "disease", "paper")]));
    expect(graph.hasNode("gene")).toBe(false);
    expect(graph.hasEdge("association")).toBe(false);
    expect(graph.hasEdge("new-evidence")).toBe(true);
    expect(graph.getNodeAttribute("disease", "label")).toBe("Updated disease name");
    expect(graph.getNodeAttribute("paper", "color")).not.toBe(graph.getNodeAttribute("disease", "color"));
    updateGraph(graph, snapshot([], []));
    expect(graph.order).toBe(0); expect(graph.size).toBe(0);
  });
  test("streaming expansion spreads shared-feature hubs and disconnected sources", () => {
    const graph = new MultiDirectedGraph<NodeAttributes, EdgeAttributes>();
    updateGraph(graph, snapshot([node("disease", "disease")], []));
    const features = Array.from({ length: 6 }, (_, i) => node(`feature-${i}`, "phenotype"));
    const matches = [node("match-a", "disease"), node("match-b", "disease")];
    const nodes = [node("disease", "disease"), ...matches, ...features, node("disconnected")];
    const edges = [edge("match-a", "disease", "match-a"), edge("match-b", "disease", "match-b"), ...features.flatMap((n) => ["disease", "match-a", "match-b"].map((id) => edge(`${id}:${n.id}`, id, n.id)))];
    updateGraph(graph, snapshot(nodes, edges));
    for (let i = 0; i < nodes.length; i++) for (let j = i + 1; j < nodes.length; j++) {
      const a = graph.getNodeAttributes(nodes[i].id), b = graph.getNodeAttributes(nodes[j].id);
      expect(Number.isFinite(a.x) && Number.isFinite(a.y)).toBe(true);
      expect(Math.hypot(a.x - b.x, a.y - b.y)).toBeGreaterThan(64);
    }
    const before = graph.nodes().map((id) => graph.getNodeAttributes(id));
    // Multiple citations must not strengthen the layout's attraction for this pair.
    updateGraph(graph, snapshot(nodes, [...edges, edge("extra-citation", "disease", "match-a")]));
    expect(graph.nodes().map((id) => graph.getNodeAttributes(id))).toEqual(before);
  });
  test("parallel evidence stays separate and unknown source types use a safe fallback", () => {
    const graph = new MultiDirectedGraph<NodeAttributes, EdgeAttributes>();
    updateGraph(graph, snapshot([node("source", "future-source"), node("target")], [edge("paper-a", "source", "target"), edge("paper-b", "source", "target"), edge("dangling", "source", "missing")]));
    expect(graph.size).toBe(2);
    expect(graph.getEdgeAttribute("paper-a", "curvature")).not.toBe(graph.getEdgeAttribute("paper-b", "curvature"));
    expect(graph.getEdgeAttribute("paper-a", "type")).toBe("curved");
    expect(graph.getNodeAttribute("source", "color")).toBe(sourceTypes.other.color);
  });
});
