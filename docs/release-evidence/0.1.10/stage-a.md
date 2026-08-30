# Prompt Hub 0.1.10 Stage A 数据库与恢复证据

审查日期：2026-08-29
分支：`codex/legacy-v5-schema-recovery`
阶段代码提交：`30cc0fa`、`17c59d4`、`44be732`、`de975c1`、`d074614`

## 已验证修复

1. 未来数据库版本、空迁移账本、缺少规范迁移记录、未知迁移 ID 和错误校验和均 fail closed。
2. `legacy/0.1.2-prompt-usage` 使用固定 SQL 校验和识别，旧 v5 数据库升级后可以再次打开且保留 `last_used_at`。
3. 版本 7 在接受连接前验证全部应用表和关键字段；缺少 `prompt_favorites` 或 `skills.snapshot_path` 不会被静默修复或误判为健康。
4. 迁移前备份使用 SQLite 在线备份 API，能够包含未检查点 WAL 中的已提交提示词。
5. Windows 应用数据目录不可用或为空时，启动状态为 `data_directory_unavailable`；准备服务和重试不会创建相对路径数据库。
6. 完整的最新数据库重开不再执行迁移账本写入；即使另一连接持有写事务也能只读打开，避免与提示词保存形成 `SQLITE_BUSY` 竞态。

数据库架构版本保持为 7；Stage A 未增加数据迁移。

## TDD 红色证据

- 首次迁移血缘测试：11 通过、4 失败。失败分别为未来版本被接受、空账本被回填、缺项账本被回填、旧 v5 升级后的第二次打开把合法 legacy ID 判为未知。
- 首次结构完整性测试：15 通过、2 失败。缺少表或关键字段的版本 7 数据库被错误接受。
- 首次 AppData 测试：编译失败，因为安全的 unavailable 状态和可用性检查尚不存在。
- 首次 WAL 备份测试：备份中预期提示词数量为 1，实际为 0。
- 首次只读重开测试：另一连接持有写事务时，最新数据库重开等待 5 秒后返回 `database is locked`。

以上测试均在生产代码修改前运行并观察到预期失败。

## 绿色证据

| 命令 | 结果 |
| --- | --- |
| `cargo test -p prompt-store --test migrations --test backup --test concurrency` | 迁移 19/19、备份 8/8、并发 1/1 通过 |
| `cargo test -p prompt-hub-desktop --test bootstrap --test commands` | 启动 4/4、命令契约 1/1 通过 |
| `cargo test -p prompt-store --test concurrency` | 并发搜索、写入与备份 1/1 通过 |
| `cargo test -p prompt-store a_failed_migration_rolls_back_the_whole_batch` | 迁移事务回滚 1/1 通过 |
| `pnpm install --frozen-lockfile` | 通过，锁文件未变 |
| `pnpm lint` | 通过 |
| `pnpm typecheck` | contracts 与 desktop 通过 |
| `pnpm test` | contracts 22/22、desktop 60/60 通过 |
| `pnpm test:e2e` | 3/3 通过 |
| `cargo fmt --check` | 通过 |
| `cargo clippy --workspace --all-targets -- -D warnings` | 通过 |
| `cargo test --workspace -q` | 通过；显式性能基准仍按设计忽略 |
| `pnpm --filter @prompt-hub/desktop build` | 通过；713 modules transformed |

## 保留限制

- Playwright E2E 仍模拟 Tauri invoke 边界，不等同于真实安装包与真实 SQLite 的完整桌面 E2E；此项归入后续离线流程和打包阶段。
- `src/App.test.tsx` 仍产生既有 React `act(...)` 测试警告，但 60 项桌面测试全部通过；Stage A 未修改相关前端行为。
- 尚未在安装版上人工触发操作系统 AppData API 失败；该边界通过纯 Rust runtime 和 service guard 自动化验证。
- 本证据不包含数据库内容、提示词正文、凭据、授权头或私人数据库绝对路径。
