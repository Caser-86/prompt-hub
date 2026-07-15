# Search baseline

This file records development baselines, not hardware-independent performance guarantees.

## 已有 foundation baseline

- Machine: AMD Ryzen 7 8845HS, 8 cores / 16 logical processors.
- Build: Rust debug test profile, bundled SQLite, in-memory database.
- Dataset: 1,000 Chinese prompts, all matching the query `性能检索`.
- Query: first page of 20 results using the FTS5 trigram path.
- Observed query duration: 6,484 microseconds.
- Command:

```powershell
cargo test -p prompt-store --test search_baseline -- --ignored --nocapture
```

## Session C 可重复测量

下列命令会在临时文件数据库上依次生成 1,000、10,000 和 50,000 条脱敏中文提示词，并报告冷/热检索、组合筛选、索引重建和备份耗时：

```powershell
cargo test --release -p prompt-store --test search_baseline -- --ignored --nocapture
```

结果必须连同运行日期、机器、Rust 版本与实际输出记录到本文件后，才能定义任何发布性能目标；该测试本身不提供跨硬件保证。

## 2026-07-15 Session C 测量结果

- 机器：AMD Ryzen 7 8845HS，8 核 / 16 逻辑处理器。
- 构建：Rust `release` 测试配置，文件 SQLite 数据库。
- 数据：脱敏中文提示词；每个规模单独新建临时数据库；数据构造采用单一 SQLite 事务，测量从正式 repository 搜索开始。
- 单位：微秒。

| 记录数 | 冷检索 | 热检索 | 分类筛选 | 索引重建 | 备份 |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1,000 | 4,849 | 5,467 | 6,959 | 18,507 | 85,754 |
| 10,000 | 47,202 | 45,083 | 53,230 | 191,148 | 430,442 |
| 50,000 | 223,912 | 221,046 | 261,544 | 816,578 | 1,806,763 |

这些是单机开发环境测量值，不是面向所有硬件的服务等级承诺。
