# 包名统一为 retainpdf

> 日期：2026-10-17
> 分支：`refactor/rename-retainpdf`（接在 `main` 14dd5547 后面）
> 改的是：[包改名实现记录](./20261016-包改名实现记录.md)定的 `retain-pdf-*`

## 决定

用户 2026-10-17 定：
- Rust 包统一叫 `retainpdf-*`，和 Python 的 `retainpdf-math`、`retainpdf-pipeline`、`retainpdf-ai-service` 一个写法。
  原来的 `retain-pdf-*` 和 math 包放在一起容易看混。
- `backend/packages/`、`database/` 下的目录也一起改，目录名就是包名。
- AI 包也叫 `retainpdf-ai`。代码里写作 `retainpdf_ai`，和 Python AI 服务的模块同名，看报错、搜代码时要看清是哪边的。
- 另一个容易看混的：翻译拆包（`refactor/translation-pkg-prep`，另一个会话在做）新建的 Python 底座包叫 `retainpdf-base`（`backend/base`），
  Rust 这边的底座包叫 `retainpdf-api-base`。两个不撞名，读的时候注意。

不改的：
- 应用目录 `backend/api`、`backend/cli`、`backend/jobs`：它们按用途命名，本来就不带 `retain`；
- 可执行文件名 `rust_api`、`retain-jobsd`、`retainpdf`、`retainpdf-agent`：写在 Docker、进程监督和部署脚本里，改名收益小、风险大；
- 历史开发记录和旧的规划文件。

## 对照

| 原目录 | 新目录 | 原包名 | 新包名 |
|---|---|---|---|
| `backend/packages/retain-core` | `backend/packages/retainpdf-core` | `retain-pdf-core` | `retainpdf-core` |
| `backend/packages/retain-data` | `backend/packages/retainpdf-data` | `retain-pdf-data` | `retainpdf-data` |
| `backend/packages/retain-jobs` | `backend/packages/retainpdf-runner` | `retain-pdf-runner` | `retainpdf-runner` |
| `backend/packages/retain-proc` | `backend/packages/retainpdf-proc` | `retain-pdf-proc` | `retainpdf-proc` |
| `backend/packages/retain-config` | `backend/packages/retainpdf-config` | `retain-pdf-config` | `retainpdf-config` |
| `database/retain-db` | `database/retainpdf-db` | `retain-pdf-db` | `retainpdf-db` |
| `backend/packages/retain-pdf-<领域>` | `backend/packages/retainpdf-<领域>` | `retain-pdf-<领域>` | `retainpdf-<领域>` |
| `backend/api` | 不变 | `retain-pdf-api` | `retainpdf-api` |
| `backend/jobs` | 不变 | `retain-pdf-runnerd` | `retainpdf-runnerd` |
| `backend/cli` | 不变 | `retain-pdf-cli` | `retainpdf-cli` |

领域包是这 10 个：`accounts`、`ai`、`api-base`、`files`、`glossaries`、`ingest`、`jobs`、`library`、`platform`、`providers`。

最容易混的一处顺带没了：原来 `retain-jobs` 目录里是运行器，`retain-pdf-jobs` 才是任务包。现在两个分别叫 `retainpdf-runner`、`retainpdf-jobs`。

## 实现

- 目录用 `git mv` 挪，历史跟得上。
- 名字按三条规则替换，都限定了边界：
  - 包名 `retain-pdf-<名字>`，后面不能紧跟字母、数字或 `-`，所以测试临时目录名 `retain-pdf-glossaries-…` 不动；
  - crate 名 `retain_pdf_` → `retainpdf_`；
  - 目录名 `retain-core` 等。

  替换范围是仓库里所有的文本文件，不包括：
  - `frontend/`：前端会话改；
  - 历史开发记录和旧的规划文件；
  - `Cargo.lock`：由 cargo 重新生成。
- 架构检查单测里有用 f-string 拼出来的包名（`f"retain-pdf-{domain}"`），规则匹配不到，单测报出来以后手改。
- **`.gitignore` 的 `data/` 规则**会挡掉任务包的源码目录 `src/data/`。原来能提交，是因为这几个文件早就在 git 里，`git mv` 挪的时候还算跟踪中。
  改名过程中重做提交时，`git add -A` 没把它们加回来。本机目录里文件都在，所以编译、测试都过了，CI 上会缺文件。
  - 比对改名前后 git 里的文件列表，发现了这 3 个文件，补回提交；
  - `.gitignore` 加了 `!backend/packages/*/src/**/data/` 例外，运行时的 `data/` 目录照旧忽略；
  - 用干净检出的分支重新编译、跑了任务包和书库包的测试。
- **顺带修了日志**：rust_api 的默认日志过滤只写了 `retain_pdf_api`、`retain_pdf_api_base`。日志的目标是写日志那段代码所在的 crate 名，
  所以拆出去的 9 个领域包、运行器、`retainpdf-data` 里的 `info` 日志一直看不到。例如任务包里有 14 处，运行器里有 15 处。
  - 现在默认过滤列全了 rust_api 依赖的所有 `retainpdf_*`；
  - 加了一个测试：`Cargo.toml` 里有、过滤里没有的 `retainpdf-*` 包，测试会红。以后拆新包不会再漏。

## 前端

只有一处会让前端 CI 红：`frontend/web/tests/contracts/job-failure-brief-contract.test.mjs` 直接读
`backend/packages/retain-core/src/models/view/job_types.rs`。其余都是注释。

已经把新旧路径发给前端会话，由它改。两边要一起合、一起推，否则前端那条测试找不到文件。

## 怎么验证的

- `cargo check --locked --workspace --all-targets` 没有警告；
- 全部 `cargo test` 和 `model_executor` 串行都过了，测试数和改名前一样；
- 这次也跑了 AI 服务的 pytest（上次拆任务就是漏了它）；
- 两个架构检查和 124 个单测、`.github/scripts/tests`、`ops/development/tests`、`check_test_filters`、依赖报告、requirements 同步检查都过了。
- 在整个仓库里搜过旧名字和旧目录：
  - 没改的只剩测试临时目录名（`retain-config-…`、`retain-jobs-test-…`）、`User-Agent: retain-pdf-github-actions`、`retain-jobsd`；
  - 还有历史记录和 `frontend/`。
