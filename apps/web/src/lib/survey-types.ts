import type { GraphSnapshot } from "./graph-types";
export type SurveyTarget = { key: string; name: string };
export type SurveyQuestion = { id: string; prompt: string; kind: "short_text" | "single_choice"; options: string[] };
export type Survey = {
  id: string; creator_id: string; title: string; description: string; created_at: string;
  questions: SurveyQuestion[]; communities: { id: string; slug: string; name: string }[];
  audience_count: number; response_count: number; can_respond: boolean; answers: string[] | null; results: { counts: Record<string, number>; texts: string[] }[] | null;
};
/** Canonical community URLs deduplicate alternate disease identifiers in the graph. */
export function graphSurveyTargets(graph: GraphSnapshot): SurveyTarget[] {
  const targets = new Map<string, SurveyTarget>();
  for (const node of graph.nodes) {
    if (node.kind !== "disease" || !node.community_url?.startsWith("/community/")) continue;
    const key = node.community_url.slice("/community/".length);
    if (!/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(key)) continue;
    if (!targets.has(key)) targets.set(key, { key, name: node.label });
  }
  return [...targets.values()].sort((a, b) => a.name.localeCompare(b.name));
}
