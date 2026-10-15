# CLAUDE.md

给在这个仓库里工作的 Claude Code 看的约定。项目怎么跑、目录怎么分，先看
[CONTRIBUTING.md](CONTRIBUTING.md) 和 [docs/README.md](docs/README.md)；通用的 AI 开发建议在
[AI 辅助开发指南](docs/core/contributing/ai-development.md)。这里只写这些文档里没有、但每次都要守的规矩。

## 沟通

- 回复一律用简体中文；代码标识符、命令、路径照原样写。
- 提交信息用中文，前缀照 `类型(范围): 说明`，例如 `fix(translate): …`、`refactor(api): …`。
  合并提交写 `Merge branch 'xxx'：一句话说清合进来什么`。

## 分工

- 前端（`frontend/`）和后端分开做。做后端时**不改 `frontend/` 下的任何文件**，读前端代码确认调用方可以。
- 后端改动对前端只增不改：旧字段、旧地址保留，新的加在旁边。前端需要跟着改的地方，写进对应的
  后端文档或整理记录（例如 `docs/ops/planning/api-reorg/`），写清文件、旧写法、新写法。

## 分支、合并、推送

- 不直接在 `main` 上开发。每件事开一个分支，用 worktree 做，做完留着等确认。
- 用户说 **"合"** 才在本地合进 `main`；说 **"推送"** 才 push。每次确认只管那一次，不顺延到下一次。
- 推送后跑 `gh run list --limit 8`，有红的要报出来，不能当作做完。
- 改了接口行为、事件、数据库结构、产物结构，文档在**同一个分支**里一起改（`docs/api/`、`docs/core/api/`、
  `backend/api/docs/api-spec/` 等）。

## 合并前必须跑的检查

CI 的 Tests、Rust API Architecture、Publish Current Web Image 都卡在这些上，本地先跑过再合：

```bash
python3 backend/api/scripts/check_architecture.py
python3 -m unittest backend/api/scripts/test_check_architecture.py
python3 backend/pipeline/devtools/check_pipeline_architecture.py
backend/.venv/bin/python backend/pipeline/devtools/extract_pipeline_requirements.py --services-root backend --check
python3 .github/scripts/sync_backend_python_requirements.py --repo-root . --services-root backend --check
cargo test -p rust_api -p retain-jobs -p retain-db -p retain-data -p retain-core -p retain-config
```

- 改了 `backend/pipeline/retainpdf_pipeline/translate/`：再跑
  `backend/.venv/bin/python backend/pipeline/devtools/run_translation_tests.py`，以及加 `--reverse` 再跑一次。
  它会连带跑 `tests/performance/pipeline/tests`；提示词快照有 sha256 守卫，改提示词只重录受影响的用例。
- 改了 `backend/ai/`：跑 AI 服务的 pytest。
- 跑不了的检查，在回复和合并说明里写明没跑、为什么。

## 本地环境的坑

- worktree 里没有 `backend/.venv`，跑 Python 和 rust_api 测试前指向主检出的环境：
  `UV_PROJECT_ENVIRONMENT=<主检出>/backend/.venv`，`PATH=<主检出>/backend/.venv/bin:$PATH`
  （rust_api 的部分测试要用 PATH 上的 `retainpdf-pipeline`）。
- `model_executor` 的测试并行跑会偶发失败，单独用 `--test-threads=1` 跑。
- 不要对整个已有文件跑 rustfmt（仓库没有整体格式化过）；从 stdin 喂给 `rustfmt --check` 永远返回 0，不能当检查用。
- 本地多用户开发栈开着时，pipeline 测试收尾会报"测试污染了仓库的 data/ 目录"，是开发栈写的，CI 上不会出现。

## 接口快照与架构检查

- 三份快照钉住对外行为：接口清单 `backend/api/docs/routes.txt`、文件出口返回 `backend/api/docs/file-responses.txt`、
  建任务各入口的结果 `backend/api/docs/submissions.txt`。有意改了才重新生成：
  `UPDATE_ROUTES=1 cargo test -p rust_api --lib route_manifest`、
  `UPDATE_FILE_RESPONSES=1 cargo test -p rust_api --lib file_responses`、
  `UPDATE_SUBMISSIONS=1 cargo test -p rust_api --lib submissions`。重新生成后看 diff，只该有预期的改动。
- 加路由、加 service 前先读 `check_architecture.py` 的规则：新的路由文件要进白名单（按完整 import 字符串匹配）；
  新的路径参数要在 `services/accounts/access.rs` 的 `classify_param` 里归类，否则多用户模式下 403 "not cleared"。
- `/jobs` 的路由和处理函数按 9 类分文件（`app/router/jobs/`、`routes/jobs/`，同名）。新接口放进对应的类，
  不要再加 `/jobs/:job_id/<名字>` 这种顶层地址；旧地址紧跟在新地址下面、行尾标 `// 旧地址`。
- 建新任务只走 `services/jobs/creation/submit.rs` 的 `submit_job`，原地重跑已有任务走 `relaunch_in_place`；
  别处直接落库或发射任务，`check_job_entry_ownership` 会报错。
- 产物文件路径只许出口模块拼（`services/jobs/downloads`、`services/library/media.rs`、`services/artifacts`、
  `services/derived_artifacts`）。`KNOWN_ARTIFACT_PATH_READERS` 是冻结的现状名单，只许减不许加。

## 记录

- **开发记录**：`docs/ops/devlog/YYYY-MM.csv`，一行一件事，每个分支合并前加一行，推送后改状态。
  列和取值见 [docs/ops/devlog/README.md](docs/ops/devlog/README.md)，`python3 ops/development/check_devlog.py` 检查格式。
- **方案和决定**：按主题放在 `docs/ops/planning/<主题>/`，文件名 `YYYYMMDD-主题.md`。
  新决定写新文件，不改写旧文件；目录的 README 列出所有文件。
- 已有的 CSV 台账（`docs/ops/planning/*.csv`）只追加，不覆盖旧行。

## 安全

- 不提交、不打印密钥和 token；用户贴进来的密钥不要写进文件或平台设置，提醒用户作废重发。
- 不提交真实用户文件、`data/` 下的数据库和任务目录、`tmp/`、大体积实验输出。
- 用户的邮箱等个人信息不发给外部服务。
