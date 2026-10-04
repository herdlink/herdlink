CREATE CONSTRAINT graph_node_uid IF NOT EXISTS
FOR (n:GraphNode) REQUIRE n.uid IS UNIQUE;

CREATE INDEX query_cache_expiry IF NOT EXISTS
FOR (n:QueryCache) ON (n.expires_at);
