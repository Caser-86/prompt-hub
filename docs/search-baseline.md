# Search baseline

This file records development baselines, not hardware-independent performance guarantees.

## 2026-07-15 foundation baseline

- Machine: AMD Ryzen 7 8845HS, 8 cores / 16 logical processors.
- Build: Rust debug test profile, bundled SQLite, in-memory database.
- Dataset: 1,000 Chinese prompts, all matching the query `性能检索`.
- Query: first page of 20 results using the FTS5 trigram path.
- Observed query duration: 6,484 microseconds.
- Command:

```powershell
cargo test -p prompt-store --test search_baseline -- --ignored --nocapture
```

Session C must repeat this measurement against 1,000, 10,000 and 50,000-record datasets, release builds and a file-backed database before defining production performance targets.
