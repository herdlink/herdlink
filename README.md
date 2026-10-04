# Herdlink

Herdlink connects biomedical research with disease communities. Explore
relationships between diseases, genes and phenotypes, inspect the supporting
literature, and discuss findings with other community members.

## Features

- Interactive Sigma.js graph backed by Neo4j, with source colors, relationship
  details and links to papers and disease communities. A complexity slider reveals
  more connections while keeping the default view focused.
- Streaming AI chat that uses PubTator3 and HPO tools to expand the current graph.
- Disease communities with membership, announcements, posts and replies.
- Surveys across the disease communities shown in your graph, with one response
  per member and results for the creator.
- User registration, login and a demo account with prefilled credentials.

## Run locally

Requires Rust, Bun, Node.js 24+, Just and Docker for the local Neo4j instance.
From the repository root:

```sh
just frontend-install
docker compose -f apps/biomedical_graph/compose.yaml up -d
just backend
# In another terminal:
just frontend
```

Open <http://localhost:3001>. The backend runs on port 3000 and creates its SQLite
database automatically. Configure the model and export backend environment
variables as described in the [backend setup](apps/backend/README.md#streaming-graph-chat).

For example graph data, run `just graph-seed` or `just graph-seed-more` for 50
additional disease use cases. `just graph-data` downloads a full HPO/Mondo
snapshot into an empty `phenotype-data` directory for phenotype comparisons.

Run `just check` to validate both apps. See the [frontend notes](apps/web/README.md)
and [graph setup](apps/biomedical_graph/README.md) for more detail.
