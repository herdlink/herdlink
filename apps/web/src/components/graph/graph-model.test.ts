import { describe, expect, test } from "bun:test";
import { MultiDirectedGraph } from "graphology";
import { type GraphSnapshot, sourceTypes } from "@/lib/graph-types";
import { updateGraph, type NodeAttributes, type EdgeAttributes } from "./graph-model";

const node = (id: string, kind = "gene") => ({ id, label: id, kind, labels: [], description: null, links: [], community_url: null });
const edge = (id: string, source: string, target: string) => ({ id, source, target, label: "associate", kind: "PUBTATOR_RELATION", evidence_id: id, evidence_kind: "summary", reported_papers: 3, context: {} });
const snapshot = (nodes: GraphSnapshot["nodes"], edges: GraphSnapshot["edges"]): GraphSnapshot => ({ nodes, edges, query: "", result_uid: "", truncated: false, generated_at: "2026-10-04T00:00:00Z" });

describe("replaceable graph snapshots", () => {
  test("updates preserve positions while adding, changing, and deleting nodes and evidence", () => {
    const graph = new MultiDirectedGraph<NodeAttributes, EdgeAttributes>();
    updateGraph(graph, snapshot([node("disease", "disease"), node("gene")], [edge("association", "disease", "gene")]));
    const original = { x: graph.getNodeAttribute("disease", "x"), y: graph.getNodeAttribute("disease", "y") };
    updateGraph(graph, snapshot([{ ...node("disease", "disease"), label: "Updated disease name" }, node("paper", "publication")], [edge("new-evidence", "disease", "paper")]));
    expect(graph.hasNode("gene")).toBe(false);
    expect(graph.hasEdge("association")).toBe(false);
    expect(graph.hasEdge("new-evidence")).toBe(true);
    expect(graph.getNodeAttribute("disease", "label")).toBe("Updated disease name");
    expect({ x: graph.getNodeAttribute("disease", "x"), y: graph.getNodeAttribute("disease", "y") }).toEqual(original);
    expect(graph.getNodeAttribute("paper", "color")).not.toBe(graph.getNodeAttribute("disease", "color"));
    updateGraph(graph, snapshot([], []));
    expect(graph.order).toBe(0); expect(graph.size).toBe(0);
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
