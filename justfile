# Frontend requires Bun and Node.js 24+; backend requires Rust and Cargo.

# List available targets.
default:
    @just --list

# Install frontend dependencies from the lockfile.
frontend-install:
    bun install --cwd apps/web --frozen-lockfile

# Run the frontend on port 3001 (override with FRONTEND_PORT).
frontend:
    bun run --cwd apps/web dev --port "${FRONTEND_PORT:-3001}"

# Build the frontend for production.
frontend-build:
    bun run --cwd apps/web build

# Serve the production frontend after frontend-build.
frontend-start:
    bun run --cwd apps/web start --port "${FRONTEND_PORT:-3001}"

# Run frontend lint, type checking, graph and chat tests, and production build.
frontend-check:
    bun run --cwd apps/web check

# Run the backend on port 3000 (override with BIND_ADDR).
backend:
    cargo run -p backend

# Build the backend for production.
backend-build:
    cargo build -p backend --release

# Run backend tests.
backend-test:
    cargo test -p backend

# Check Rust formatting, lint the backend, and run its tests.
backend-check:
    cargo fmt --all -- --check
    cargo clippy -p backend --all-targets -- -D warnings
    cargo test -p backend

# Build both apps for production.
build: frontend-build backend-build

# Run all frontend and backend checks.
check: frontend-check backend-check

# Download a full official HPO/Mondo snapshot into a new empty directory.
graph-data:
    cargo run -p pubtator3-hpo --example download_data -- "${HPO_DATA_DIR:-phenotype-data}"

# Seed three bounded real PubTator use cases, plus HPO comparisons when configured.
graph-seed:
    cargo run -p biomedical_graph --example seed_use_cases

# Seed 50 additional diseases, or supply your own disease names file.
graph-seed-more diseases_file="apps/biomedical_graph/examples/more-diseases.txt":
    cargo run -p biomedical_graph --example seed_use_cases -- --diseases-file {{quote(diseases_file)}}
