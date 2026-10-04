import { expect, test } from "bun:test";
import { graphSurveyTargets } from "./survey-types";
import type { GraphSnapshot } from "./graph-types";
test("survey audience deduplicates canonical disease communities and excludes unrelated sources", () => {
  const base = { labels: [], description: null, links: [], community_url: null };
  const graph: GraphSnapshot = { query: "Huntington disease", result_uid: "", generated_at: "", truncated: false, edges: [], nodes: [
    { ...base, id: "canonical", label: "Huntington disease", kind: "disease", community_url: "/community/mondo-0007739" },
    { ...base, id: "mesh", label: "Huntington Disease", kind: "disease", community_url: "/community/mondo-0007739" },
    { ...base, id: "gene", label: "HTT", kind: "gene" },
    { ...base, id: "related", label: "Related disease", kind: "disease", community_url: "/community/mondo-related" },
    { ...base, id: "invalid", label: "Invalid", kind: "disease", community_url: "https://example.com/community/other" },
  ] };
  expect(graphSurveyTargets(graph)).toEqual([{ key: "mondo-0007739", name: "Huntington disease" }, { key: "mondo-related", name: "Related disease" }]);
});
