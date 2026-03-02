# Stasher Technical Reference

A summary of core concepts and design decisions for the Stasher project.

## 🛠️ Tool-to-Module Mapping
- **Daemon**: Rust + `notify` (using `FSEvents` on macOS).
- **Concurrency**: Tokio (Async runtime).
- **Relational Data**: SQLite + SQLx (WAL mode).
- **Semantic Intelligence**: LanceDB + `fastembed` (`nomic-embed-code`).
- **CLI**: Clap.

## 🧠 Core Concepts
- **FSEvents**: Kernel-level notification for file changes (OS-buffered).
- **Myers Diff**: Algorithm to find the Shortest Edit Script (SES). O(ND) complexity.
- **Vector Embeddings**: 768-dimensional representation of code logic for semantic search.
- **WAL (Write-Ahead Log)**: Allows concurrent readers and single writer in SQLite.
- **Graceful Daemons**: Handling `SIGINT`/`SIGTERM` to prevent DB corruption.

## ✂️ Chunking Strategies
- **Fixed-Size**: Simple but loses context (e.g., cuts functions).
- **Line-Based**: Respects breaks but lacks semantic understanding.
- **Semantic (Tree-sitter)**: **Best Approach.** Groups code by logic units (functions, classes).

## 📊 Myers Algorithm logic
- **Graph Search**: Treats diffing as finding the shortest path in an "Edit Graph" (X=Original, Y=Modified).
- **Diagonal moves**: Matches (cost 0).
- **Horizontal/Vertical**: Deletions/Additions (cost 1).

## 🌳 Merkle Trees vs. CAS
- **Merkle Trees**: Tree of hashes. Used by Git/Cursor for fast verification and sync.
- **CAS (Content-Addressable Storage)**: Simple deduplication (hash-based storage). Sufficient for local-first stasher.

## 🔄 Diffing & Line Shifting
- **Hunk Headers**: `@@ -start,len +start,len @@` uses context lines to apply patches.
- **Replay Logic**: History is a linear stream of patches. To restore, we "invert" the patches (swap + and -).
- **No Conflicts**: Since capture is sequential and local, there are no branches, thus no merge conflicts.
- **Anchor Points**: Occasional full-file snapshots (checkpoints) prevent error accumulation in the diff chain.
