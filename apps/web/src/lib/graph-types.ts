export type SourceLink = { label: string; url: string };
export type GraphNode = {
  id: string; label: string; kind: string; labels: string[];
  description: string | null; links: SourceLink[]; community_url: string | null;
  reasons?: string[];
};
export type GraphEdge = {
  id: string; source: string; target: string; label: string; kind: string;
  evidence_id: string; evidence_kind: string; reported_papers: number | null;
  context: Record<string, unknown>;
};
/** Replace this snapshot when tool results arrive. IDs remain stable across updates. */
export type GraphSnapshot = {
  nodes: GraphNode[]; edges: GraphEdge[]; query: string; result_uid: string;
  truncated: boolean; generated_at: string;
};
export type GraphSelection = { kind: "node"; id: string } | { kind: "edge"; id: string };
export type GraphEvidence = { papers: GraphNode[]; has_more: boolean; offset: number; note: string };
export const relationshipTypes: Record<string, { label: string; color: string }> = {
  PHENOTYPE_SIMILARITY: { label: "Phenotype similarity", color: "#d49b29" },
  HAS_PHENOTYPE: { label: "Curated phenotype", color: "#d49b29" },
  EXCLUDES_PHENOTYPE: { label: "Absent phenotype", color: "#c46565" },
  CONFLICTING_PHENOTYPE: { label: "Conflicting phenotype", color: "#c46565" },
  PUBTATOR_RELATION: { label: "Reported association", color: "#169d93" },
  RELATION_PAPERS: { label: "Relation literature", color: "#9571d9" },
  EXTRACTED_RELATION: { label: "Extracted relationship", color: "#368c75" },
  MENTIONS: { label: "Paper mention", color: "#a4aab3" },
  ANNOTATION_CITATION: { label: "Annotation citation", color: "#9571d9" },
  RELATION_CITATION: { label: "Relation query citation", color: "#9571d9" },
  DISEASE_MAPPING: { label: "Disease mapping", color: "#7f80cd" },
};
export const sourceTypes: Record<string, { label: string; color: string; dotClass: string }> = {
  disease: { label: "Disease", color: "#4f7ff0", dotClass: "bg-[#4f7ff0]" },
  gene: { label: "Gene", color: "#169d93", dotClass: "bg-[#169d93]" },
  phenotype: { label: "Phenotype", color: "#d49b29", dotClass: "bg-[#d49b29]" },
  publication: { label: "Publication", color: "#9571d9", dotClass: "bg-[#9571d9]" },
  chemical: { label: "Chemical", color: "#db6686", dotClass: "bg-[#db6686]" },
  variant: { label: "Variant", color: "#6a9c44", dotClass: "bg-[#6a9c44]" },
  species: { label: "Species", color: "#bc7850", dotClass: "bg-[#bc7850]" },
  cell_line: { label: "Cell line", color: "#4e9eb5", dotClass: "bg-[#4e9eb5]" },
  clinical_trial: { label: "Clinical trial", color: "#368c75", dotClass: "bg-[#368c75]" },
  dataset: { label: "Dataset", color: "#7f80cd", dotClass: "bg-[#7f80cd]" },
  other: { label: "Other source", color: "#929aa4", dotClass: "bg-[#929aa4]" },
};
