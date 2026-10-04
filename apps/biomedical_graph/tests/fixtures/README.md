# Bounded official examples

Fetched on 2026-10-04:

- `pubtator-entities.json`: `https://www.ncbi.nlm.nih.gov/research/pubtator3-api/entity/autocomplete/?query=Huntington%20disease&concept=disease&limit=2` (two entities).
- `pubtator-relations.json`: `https://www.ncbi.nlm.nih.gov/research/pubtator3-api/relations?e1=%40DISEASE_Huntington_Disease&e2=gene&type=associate&limit=2` (two relations).
- `pubtator-document.json`: `https://www.ncbi.nlm.nih.gov/research/pubtator3-api/publications/export/biocjson?pmids=19894120` (one abstract, 26 annotations).
- `hpo-term.json`: `https://ontology.jax.org/api/hp/terms/HP:0002072` (one term: Chorea).

Extracted from already downloaded official `v2026-09-01` releases:

- `phenotype.hpoa`: four unchanged rows for `OMIM:143100`, including Chorea.
- `hp.obo`: corresponding terms and direct-parent closure (fourteen terms),
  retaining complete OBO term metadata. The header explicitly says it is a subset.
- `mondo.json`: Huntington `MONDO:0007739`, retaining its complete node metadata
  from `mondo.json`, with ontology edges removed. This is not a complete Mondo graph.

Release asset sources:

- `https://github.com/obophenotype/human-phenotype-ontology/releases/download/v2026-09-01/hp.obo`
- `https://github.com/obophenotype/human-phenotype-ontology/releases/download/v2026-09-01/phenotype.hpoa`
- `https://github.com/monarch-initiative/mondo/releases/download/v2026-09-01/mondo.json`

The synthetic fixtures in `pubtator3-hpo/tests/fixtures` used by some tests are
separate and explicitly labeled; their annotations are not clinical evidence.
