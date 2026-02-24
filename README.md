# Stasher

Stasher is a local-first development history tracker designed to capture the intermediate state of your codebase between Git commits. While Git records what you ship, Stasher records how you built it.

Stasher runs as a background daemon that snapshots every file save, making your entire development history searchable and recoverable without requiring manual commits or cloud synchronization.

---

## Technical Overview

Modern development involves rapid iteration and frequent use of AI-assisted coding tools. During these sessions, multiple implementations are often tried, refined, or overwritten. If a valuable piece of logic is deleted or refactored before a commit is made, it is traditionally lost.

Stasher eliminates this risk by automatically capturing differential snapshots of your workspace. It provides a searchable timeline of your work, allowing you to recover deleted snippets, audit AI-generated refactors, and reconstruct previous states of your logic.

---

## Core Capabilities

### Continuous Snapshotting
Stasher monitors file system events and captures unified diffs on every save. Unlike full-file backups, this approach minimizes storage overhead while maintaining a granular history of every change.

### Session Awareness
Changes are automatically grouped into logical working sessions. This allows you to review the "blast radius" of a specific coding session, particularly useful for auditing changes made by AI agents across multiple files.

### Semantic Search and Retrieval
Integrated natural language processing allows you to query your history semantically.
- **Natural Language Queries:** Search for concepts like "how I handled the JWT implementation this morning."
- **Deleted Code Recovery:** Surface logic that was removed before a commit.
- **Time-range Analysis:** View the evolution of a specific file over the last 48 hours.

### Privacy and Local Infrastructure
Stasher is built with a local-first philosophy.
- **No Cloud Dependency:** All data stays on your local machine.
- **No Telemetry:** The system does not report usage data or code content to external servers.
- **On-Device Embeddings:** Uses local models for code embeddings, ensuring search functionality works offline without leaking sensitive code to third-party APIs.

---

## Architecture

Stasher consists of a lightweight daemon and a structured storage layer.

### The Daemon
- **Language:** Built in Rust for minimal resource footprint (<5MB RAM at idle).
- **Monitoring:** Uses FSEvents/inotify to watch file saves.
- **Shell Integration:** Captures terminal commands, output, and execution context to link file changes with the commands that triggered them.

### Storage Layer
- **Metadata:** SQLite (WAL mode) stores structured records of snapshots, session metadata, and terminal logs.
- **Vector Search:** LanceDB manages code embeddings for hybrid search (vector similarity + full-text + metadata).
- **Embeddings:** Uses `nomic-embed-code` via a local CPU-bound embedding engine (`fastembed`).

---

## Data Schema

### Snapshot Record (SQLite)
| Field | Type | Description |
|---|---|---|
| `snapshot_id` | UUID | Primary key |
| `session_id` | UUID | Links changes to a working session |
| `file_path` | String | Path relative to workspace root |
| `timestamp` | Integer | Unix timestamp (ms) |
| `diff_patch` | String | Unified diff format |
| `lines_added` | Integer | Count of additions |
| `lines_removed` | Integer | Count of removals |

### Vector Record (LanceDB)
| Field | Type | Description |
|---|---|---|
| `vector_id` | UUID | Primary key |
| `snapshot_id` | UUID | Foreign key to SQLite |
| `chunk_content` | String | Embedded code fragment |
| `embedding` | Vector(768) | Semantic representation |

---

## Usage

Stasher is controlled via a command-line interface:

- `stasher ask <query>`: Semantic search across your history.
- `stasher show <file>`: View the timeline for a specific file.
- `stasher diff --session`: Review changes within the current session.
- `stasher restore <file> --pre-session`: Revert a file to its state before the current session began.

---

## Contributing

Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on branch naming, commit messages, and the development workflow.

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.
