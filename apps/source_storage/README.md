# source_storage

A synchronous library for SQLite records and local raw-file storage. All three
apps depend on it via `source_storage = { path = "../source_storage" }`; Cargo includes
the library in the workspace through these path dependencies.

```rust
use source_storage::{Client, Category};

let client = Client::open("records.sqlite", "objects")?;
let records = client.find_by_category(Category::Research)?;
for record in records {
    let bytes = client.read_file(record.storage.id)?;
    println!("{}: {} bytes", record.content.title, bytes.len());
}
# Ok::<(), source_storage::Error>(())
```

The four schema sections are columns of one `records` table, represented by
`StorageIdentity`, `Classification`, `Source`, and `Content` inside `Record`.
`Author` models names, optional ORCIDs, and affiliations. JSON serialization of
`Record` flattens the four sections to match the schema's field names.

`Client::open` initializes the schema and indexes automatically. The database's
parent directory must exist; the object directory is created automatically.
SQLite stores UUIDs as text, UTC timestamps as RFC 3339 text, dates as YYYY-MM-DD,
booleans as integers, and JSON as text. Enum, boolean, hash, and JSON constraints
are enforced by SQLite. Category-specific `metadata` must be a JSON object.
Nullable authors support both `None` (SQL NULL) and `Some(vec![])` (empty JSON array).

- `insert_with_file(NewRecord, bytes)` generates an ID, writes
  `objects/raw/{id}.{format}`, computes SHA-256, and inserts the complete record.
- `object_store().put(id, format, bytes)` and `insert(&Record)` provide separate
  file and metadata writes. `insert` also permits metadata for externally stored
  objects, but `read_file` only reads objects managed by this local store.
- `get(id)` returns `Option<Record>`; `list(limit, offset)` provides pagination.
- `find_by_source`, `find_by_doi`, `find_by_content_hash`, and `find_by_category`
  load matching records. DOI and content-hash matches can span multiple sources;
  neither is unique. DOI lookups match the stored text exactly.
- `update(&Record)` updates metadata for an existing ID; it does not write files.
- `read_file(id)` loads the managed file and verifies its checksum.
- `object_store().read(id, format)` loads raw bytes without consulting SQLite.

Files are written through temporary files and published without overwriting an
existing object. If a combined insert fails, the new file is removed. Filesystem
and SQLite writes are not one transaction: a process crash between them can
leave an orphaned object. Concurrent database writes wait up to five seconds.
The client uses blocking I/O; async callers should run operations on a blocking
thread. Changing an existing database schema requires a future migration.

Run a complete insert/load example in a scratch directory:

```sh
cargo run --manifest-path /path/to/apps/source_storage/Cargo.toml --example store_record
```

From the repository, verify the library and its consumers with:

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```
