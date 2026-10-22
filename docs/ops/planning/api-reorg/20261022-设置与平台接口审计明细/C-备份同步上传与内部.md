# 备份、同步、上传、健康检查、模型执行器内部接口

审计范围：`/api/v1/backups` 4 个、`/api/v1/sync` 4 个、`POST /api/v1/uploads`（只看上传本身）、`GET /health`、`GET /ready`、`/api/v1/internal/model/jobs/...` 4 个（只看接口层）。
只读代码（main，`35043e3f`）得出，没跑测试、没调用运行中的服务。标「未实跑」的是读代码推断的结论。上传和书的关系书库审计 C11 已记过，这里不再记。

## 接口一览

| 方法 地址 | 处理函数 | 返回类型 | 契约 | 文档 | 调用方（前端文件:行） | 备注 |
|---|---|---|---|---|---|---|
| GET `/api/v1/backups`（admin） | `routes/backups.rs::list_backups_route` → `retainpdf-platform` `backup/api.rs::backup_status_view` → `BackupService::status` | `BackupStatusView { dir, auto_interval_hours, last_auto_at, running, restore_blockers, items }` | 无；前端手写 `frontend/packages/api/src/backups.ts:10-36` | `docs/api/10-备份/01-数据库备份.md:15-34` | `BackupPanel.tsx:36`（经 `platform/api/domains/backups.ts`）；CLI `backend/cli/src/cmd/backup.rs:45` | 不分页（保留规则封顶约 27 份），约定里没写 |
| POST `/api/v1/backups`（admin） | `backups::create_backup_route` → `create_backup_view` | `BackupItemView` | 无 | 同上 `:36-38` | `BackupPanel.tsx:50`；CLI `backup.rs:96` | 在阻塞线程里做 |
| DELETE `/api/v1/backups/:backup_id`（admin） | `backups::delete_backup_route` → `delete_backup_view` | `BackupStatusView` | 无 | 同上 `:65-67` | `BackupPanel.tsx:81`；CLI `backup.rs:152` | |
| POST `/api/v1/backups/:backup_id/restore`（admin） | `backups::restore_backup_route` → `BackupService::restore` → `retainpdf-db` `backup.rs::restore_backup` | `BackupRestoreView { restored, safety_backup, status }` | 无 | 同上 `:40-63` | `BackupPanel.tsx:64`；CLI `backup.rs:126` | 见第 1、2、3、7、8 条 |
| GET `/api/v1/sync`（admin） | `routes/sync.rs::get_sync_route` → `sync/api.rs::sync_status_view` → `SyncControl::status_view` | `SyncStatusView`（`retainpdf-core/src/models/sync.rs:84-106`） | 无；前端手写 `frontend/packages/api/src/sync.ts:38-56` | `docs/api/09-同步/01-多设备同步.md:25-76` | `SyncSettingsPanel.tsx:62`（面板开着时轮询）；CLI `backend/cli/src/cmd/sync.rs:50` | 密码只回 `webdav_has_password` |
| PUT `/api/v1/sync`（admin） | `sync::update_sync_route` → `update_sync_settings_view` → `SyncControl::update` | `SyncStatusView` | 无 | 同上 `:78-100` | `SyncSettingsPanel.tsx:82`；CLI `sync.rs:57` | 见第 9 条 |
| POST `/api/v1/sync/run`（admin） | `sync::run_sync_route` → `run_sync_now_view` → `SyncService::run_once` | `SyncStatusView` | 无 | 同上 `:111-113` | `SyncSettingsPanel.tsx:138`；CLI `sync.rs:114` | 请求一直开着等这一轮跑完 |
| POST `/api/v1/sync/test`（admin） | `sync::test_sync_route` → `test_sync_target_view` → `SyncControl::test` | `SyncTestView { ok, location, latency_ms, error }` | 无 | 同上 `:102-109` | `SyncSettingsPanel.tsx:114` | 见第 5 条 |
| POST `/api/v1/uploads`（signed，`unlimited_body`） | `routes/uploads.rs::upload_pdf` → `retainpdf-ingest` `uploads/service.rs::store_for` | `UploadView { upload_id, document_id, filename, bytes, page_count, uploaded_at }` | 无（`create-job.v1` 只有 `upload_id`） | `backend/api/docs/api-spec/jobs-submission.md:5-62`、`storage-and-errors.md:117-129`；`docs/api/` 下没有单独一页 | `frontend/packages/api/src/jobs-submit.ts:41`；`frontend/web/src/features/ingest/domain/upload/config-port.ts:7`、`execution.ts:128`；`frontend/web/scripts/frontend-status-smoke.mjs:296` | 见第 10 条 |
| GET `/health`（public） | `routes/health.rs::health` → `health_api.rs::build_health_view` | `HealthView { status, db, queue_depth, running_jobs, provider_backends, ai_service, jobsd, time }` | 无 | 只有地址（`docs/core/api/index.md:12`、`overview.md:8`），没写返回字段 | 桌面 `frontend/desktop/src/main/backend-http.js:8`、`port-occupant.js:179`（认 `data.status === "up"`）；CLI `backend/cli/src/context.rs:144,152`、`cmd/status.rs:117-118`（读 `queue_depth`、`running_jobs`）；Docker `ops/deployment/docker/delivery/docker-compose.yml:36` | 主端口和 42000 端口都有 |
| GET `/ready`（public） | `health::ready` → `build_current_readiness_view` | `ReadinessView { status, components, reasons, time }`，不就绪时 503 | 无 | 同上，没写返回字段 | CLI `context.rs:157`；Docker `docker-compose.yml:19` | 503 时信封仍是 `code: 0` |
| POST `/api/v1/internal/model/jobs/:job_id/capability`（closed，认证之内） | `routes/model_requests.rs::issue` → `retainpdf-providers` `model_requests_api.rs::issue` | 裸 JSON `{token_type, capability, expires_in}`（不带信封） | 无 | 只有接口目录一行 | **没有生产调用方**：任务运行时在进程内直接建会话（`retainpdf-runner/src/job_runner/worker_process.rs:107-123`）；只有 `api_tests/model_requests.rs:344`、`model_executor.rs:276` | 见第 11 条 |
| POST `/api/v1/internal/model/jobs/:job_id/requests`（closed，认证之外，凭 Bearer） | `model_requests::submit` | 裸 JSON `ModelOperation` | 无 | 同上 | `backend/translation/retainpdf_translation/llm/shared/rust_executor.py:107` | |
| GET `/api/v1/internal/model/jobs/:job_id/requests/:operation_id` | `model_requests::status` | 裸 JSON `ModelOperation` | 无 | 同上 | `rust_executor.py:127` | |
| POST `/api/v1/internal/model/jobs/:job_id/requests/:operation_id/cancel` | `model_requests::cancel` | 裸 JSON `{changed}` | 无 | 同上 | `rust_executor.py:130` | |

登记表（`app/router/backups.rs`、`sync.rs`、`ingestion.rs`、`public.rs`、`model_requests.rs`）、`backend/api/docs/routes.txt:51-54,106-109,155-161` 和 `docs/api/00-接口目录.md:15-16,72,215-218,226-229,277-280` 三处一致（同源生成），本组没有直接写 `.route(` 的。

## 问题

### 1. 恢复备份、立即同步的请求被中途断开时，锁先放了、后台的活还在跑：恢复期间同步可能照跑，「正在备份」能卡一整天
- 位置: `backend/packages/retainpdf-platform/src/backup/mod.rs:99-109`（`blocking`：先置 `running=true`，`spawn_blocking(..).await` 回来后才置回 false）、`:134-182`（`restore`：`_guard = self.lock`、`_sync_paused = self.sync.pause()` 都挂在请求的 future 上）、`:185-198`（自动备份只在「到期」时才走 `blocking`）；`sync/mod.rs:82-108`（`run_once` 同样）、`:111-113`（`pause` 就是拿 `run_lock`）；对照 `retainpdf-ingest/src/uploads/service.rs:92-94`（上传专门 `tokio::spawn`，注释「HTTP 调用方断开也要做完，不能在解析器还活着时放掉容量」）。前端 `BackupPanel.tsx:112`（`status.running` 为真时「立即备份」按钮禁用）；CLI 的请求超时 600 秒（`backend/cli/src/context.rs:133`）。
- 现状: 处理函数的 future 在客户端断开时会被丢掉（未实跑，依据是 hyper/axum 的默认行为）。丢掉时：
  - `spawn_blocking` 里的活不会停（JoinHandle 丢了任务照跑），但 `lock`、`run_lock` 的守卫随 future 一起释放；
  - `running.store(false)` 不会执行。
- 问题:
  - **恢复的请求断了**：恢复还在阻塞线程里换库，`run_lock` 已经放了，后台同步循环（每 60 秒）可以开始新的一轮，读写到一半新一半旧的库、或在 `sync_after_restore` 之前读到备份里那份旧的设备号和记账，把旧内容当本机改动发出去——这正是恢复流程「停住同步」要防的事（`backup.rs:28-31`）。恢复后的对账钩子（`after_restore`）也不会跑。另一个恢复/备份也能马上拿到锁，和还在跑的那次并发。
  - **`running` 卡在 true**：设置页一直显示「正在备份或恢复」、「立即备份」按钮禁用；自动备份没到期不会再走 `blocking`，最长要等到下一次自动备份（默认 24 小时）才自己好。
  - **立即同步的请求断了**：同样放掉 `run_lock`，下一个 `POST /sync/run`、后台循环或恢复的 `pause()` 能在上一轮还在跑时进来，「同一时间只跑一轮」不成立。首次 WebDAV 同步、慢线路一轮跑十几分钟很常见，CLI 600 秒超时、关掉设置页都会触发。
  - 恢复要先等正在跑的那一轮同步结束（`pause().await`）、再 `VACUUM INTO` 一份恢复前备份，整个请求可以很长，断开的机会不小。
- 建议: 照上传的做法，把「拿锁 → 阻塞活 → 放锁、置回 running、跑钩子」整个放进 `tokio::spawn` 的任务里，请求只等它的结果；`running` 用一个 drop 守卫置回。同步的 `run_once` 同样处理。顺带：`restore` 的结果可以记下来（成功/失败、安全备份编号），`GET /backups` 回一个 `last_restore`，请求断了前端刷新也能知道结果（只增）。对前端的影响: 无接口变化；加 `last_restore` 的话前端可选读。
- 等级: 高

### 2. 「有没有任务在跑」只在恢复开头查一次，之后要等同步、做安全备份，期间新建的任务照样能进来，然后被恢复抹掉
- 位置: `backup/mod.rs:139-156`（先 `restore_blockers`，再 `sync.pause().await`，再在阻塞线程里 `create_backup(KIND_BEFORE_RESTORE)`、`restore_backup`）；`database/retainpdf-db/src/db/backup.rs:270-289`（只数库里 queued/running 的任务、AI 改文档、AI 计算）、`:291-293`（注释：「调用方负责……确认没有在跑的任务」）；建任务入口 `services/jobs/lifecycle/creation/submit.rs` 不看任何「维护中」标记。
- 现状: 检查和真正换库之间隔着「等这一轮同步跑完」（WebDAV 上可以是几分钟到几十分钟）和一次 `VACUUM INTO`（库大时几秒到几十秒）。这段时间里建任务、AI 改文档、上传都不受限制。
- 问题: 窗口里新建并开跑的任务，换库后行没了（备份里没有它），worker 进程还在跑、还在写事件和产物，对账钩子只处理备份里带回来的那些 queued/running 行（未实跑）。AI 对话正在写消息、上传正在发布（`publish_upload`）也不在 `restore_blockers` 里，换库后消失或留下孤儿文件。文档写的是「`409`：有任务在跑等……什么都没改」，用户以为恢复前已经确认过没有在跑的。
- 建议: 在阻塞线程里、做完安全备份之后、`Backup::step` 之前再查一次 `restore_blockers`，不空就放弃并返回同样的 409（安全备份留着或删掉都行）；更彻底的是给 `BackupService` 一个「恢复中」标记，`submit_job`、AI 改文档的入口在标记期间返回 409（`reason: "restore_in_progress"`），这要动任务包，单独开分支。对前端的影响: 多一种 409 的可能，前端现在只显示 `message`，无需改。
- 等级: 中

### 3. 换库之后的几步（结构升级、同步记账重置、清 pid）不和换库一起提交，中途失败不回滚，接口回 500 而库已经换了
- 位置: `database/retainpdf-db/src/db/backup.rs:332-367`（`Backup::step(-1)` 一步提交后，依次 `execute_batch(PRAGMA…)`、`Db::apply_schema`、`sync_after_restore`、`clear_restored_job_pids`，每步各自提交）；`database/retainpdf-db/src/db/sync.rs:441-474`（`sync_after_restore`：先 `DELETE FROM sync_state` 等十张表，再逐行插回恢复前的设置，各条自动提交）；`backup/mod.rs:149-157`（阻塞线程返回 `Err` 时直接 `?` 成 500，没有回滚、也不告诉调用方安全备份的编号）。
- 现状: 换库那一步本身是原子的；后面四步任何一步失败（库忙超过 busy_timeout、磁盘满、旧备份的结构升级出错），接口返回 500，库停在「内容已经是备份的，后续没做完」的状态。
- 问题:
  - `apply_schema` 没做完：库停在旧结构版本，其它进程（jobsd、AI 服务、pipeline）按新结构读写会报错；
  - `sync_after_restore` 没做完：要么库里还是备份那份旧的设备号和同步记账——同步一恢复（第 1 条，或者下一轮）就会把旧内容当本机改动发出去，盖掉别的设备上更新的内容；要么 `DELETE FROM sync_state` 之后插回一半，同步设置（含 WebDAV 密码）丢了；
  - 恢复期间 `PUT /sync` 改的设置会被 `previous_sync_state`（换库前读的快照）盖回去，接口当时回 200（未实跑）；
  - 500 的信封里只有 `error_id`，用户不知道库已经换了，也不知道该用哪份「恢复前」备份退回。
- 建议: `sync_after_restore` 和 `clear_restored_job_pids` 包进一个 `BEGIN IMMEDIATE` 事务；更稳的是在解开的临时副本上先做（副本是可写打开的，`backup.rs:314-315`）：在副本上 `apply_schema`、清 pid、写入恢复前的 `sync_state` 并重置记账，全部成功后再一步 `Backup::step` 换进正在用的库，这样换库就是最后一步、失败什么都不改。退而求其次，失败时自动用刚做的安全备份再恢复一次，并在错误 `details` 里带 `safety_backup`。对前端的影响: 无。
- 等级: 中

### 4. WebDAV 密码明文存在数据库里，每份备份都带着它（默认权限的 gzip 文件），凭据库那边却要求 0600
- 位置: `backend/packages/retainpdf-data/src/sync/control.rs:3-4,31,221-223`（密码原文写进 `sync_state`）；`database/retainpdf-db/src/db/backup.rs:121-147`（`VACUUM INTO` 整库再 gzip，`fs::File::create` 默认权限）；对照 `backend/packages/retainpdf-data/src/credentials.rs:1-25`（凭据库单独放 `secrets/credentials.json`，读之前查 0600、不是符号链接）；文档 `docs/api/09-同步/01-多设备同步.md:95-96`（「密码只存在本机数据库里，不进同步数据，任何接口都不会返回它」）、`docs/api/10-备份/01-数据库备份.md:3-5`（没提备份里有密码）。
- 现状: 接口层没问题——`GET /sync` 只回 `webdav_has_password`，`SyncTarget::location`、`describe` 都去掉账号。但 `data/backups/db/` 下自动 7+4 份、手动 10 份、恢复前/升级前各 3 份，每份都是带明文密码的完整库。多用户模式下库里还有 `password_hash`、会话 `token_hash`（冻结，只记）。
- 问题: 用户把备份目录拷走、发给别人排障、或放进网盘（桌面版有「打开备份文件夹」按钮，`frontend/desktop/main.js:688-696`），NAS 的 WebDAV 密码就跟着出去了。恢复时本来就保留恢复前的 `sync_state`（`sync.rs:441-458`），备份里那份密码恢复时根本用不上。
- 建议: 最小改动：`write_backup` 在 `VACUUM INTO` 出的临时副本上先 `DELETE FROM sync_state WHERE key = 'webdav_password'`（再 `VACUUM`）然后压缩，不影响恢复；备份文件建成 0600。长远：WebDAV 密码改存凭据库，`sync_state` 只存 `credential_ref`。文档两处补一句。对前端的影响: 无。
- 等级: 中

### 5. `sync/test` 没有自己的超时：单个请求最长 30 分钟、失败再重试 3 次，一次测试要发 5 个请求
- 位置: `backend/packages/retainpdf-data/src/sync/webdav.rs:120-124`（`connect_timeout(15s)`、`timeout(1800s)`）、`:155-172`（`send`：没拿到回复就重试 3 次，间隔 0.5/1/1.5 秒，能 `try_clone` 的请求体都重试）、`:283-303`（`write_atomic` = MKCOL 每级目录 + PUT + MOVE）；`store.rs:483-493`（`probe` = 写 + 读 + 删）；`control.rs:264-289`；`sync/mod.rs:67-79`（放在 `spawn_blocking` 里，没有外层超时）；前端 `frontend/packages/api/src/internal/runtime.ts:133-139`（`apiFetch` 不设超时）、`SyncSettingsPanel.tsx:114,296-299`（测试期间按钮显示「正在测试…」）。
- 现状: 地址写错、主机不在线：第一个 MKCOL 要 4×15 秒 + 3 秒 ≈ 1 分钟才失败。连得上但不回（防火墙吞包、TLS 卡住、Tailscale 中转断了）：每个请求最长 1800 秒、重试 4 次，第一个请求就可能挂两个小时（未实跑）。
- 问题: 「测试连接」本该几秒内给结论。用户等不及刷新页面再点，每次都占一个阻塞线程（tokio 阻塞池默认 512，不至于耗尽，但线程和连接一直挂着）。
- 建议: 测试用单独的客户端配置：`WebDavBackend::new` 加一个「探测」参数，探测时单请求总超时 10~15 秒、不重试；`SyncService::test` 外层再套 `tokio::time::timeout`（比如 30 秒），超时返回 `{ok:false, error:"连接超时"}`。对前端的影响: 无，只是更快拿到 `ok:false`。
- 等级: 中

### 6. 异步处理函数里直接跑 SQLite、文件 IO 和子进程
- 位置:
  - `GET /backups`、`DELETE /backups/:id`：`backup/mod.rs:83-97`（`read_dir` + 每个文件 `metadata` + 连库数三张表）、`:125-132`（`remove_file`）、`backup/api.rs:41-44`（删完再 `status()`）；
  - 恢复前后：`backup/mod.rs:136,139`（`list_backups`、`restore_blockers`）、`:180`（`status()`）；
  - `GET /sync`：`control.rs:356-385`（约 10 次 `sync_state_get`，每次各开一个连接；`device_name()` 在没设过名字时起 `scutil`/`hostname` 子进程并写库，`:42-57,156-165`）；前端面板开着时轮询它；
  - `PUT /sync`：`control.rs:183-235`（多次写库）、`:237-260`（`check_folder`：`canonicalize`、`create_dir_all`、写一个测试文件再删——目标常是 iCloud/网盘/NAS 挂载目录，可能卡几秒）；
  - `POST /sync/run`：`sync/mod.rs:84`（`enabled()`、`target()` 读库）、`sync/api.rs:39`（跑完再 `status()`）；
  - `/health`、`/ready`：`health_api.rs:76-85,107`（`ping` + 两次 `COUNT`，每次新开连接；桌面、Docker 健康检查定时打）；
  - 模型执行器：`model_requests_api.rs:29-43,50-69,90-107`（`get_job`、`create_model_session`、`authorize_model_session`、`get_model_operation`、`cancel_model_operation` 都在 async 里直接调）。
- 现状: 只有备份/恢复的主体、同步一轮、`sync/test` 放进了 `spawn_blocking`；其余全在 tokio 工作线程上。
- 问题: 违反规矩（`20261019-书库读错和阻塞线程.md`）。库忙时每次最多等 busy_timeout；恢复的 `Backup::step` 拿着写锁时，`PUT /sync` 的写会在工作线程上等。`/health` 本来该最便宜，现在每次三个连接。
- 建议: `BackupService::status`、`delete`，`SyncService::status`、`update` 改成 async、内部 `spawn_blocking`；`run_once` 把 `enabled/target` 判断挪进阻塞闭包（`run_recorded` 里已经判过一次，外面这次可以去掉）；`/health` 一次连接里做完 ping 和计数并放进 `spawn_blocking`；`device_name()` 的默认值在 `SyncService::new` 时算好一次。模型执行器的几处留给执行器那块一起改（只记录）。对前端的影响: 无。
- 等级: 中

### 7. 409 都没有 `reason`；备份坏了/版本更新算 400，按 400/409 分工应是 409
- 位置: `backup/api.rs:20-26`（`Busy` → `AppError::conflict`；`Invalid` → `bad_request`）；`backup/mod.rs:158-163`；`sync/api.rs:36-38`（「同步没有开启」→ `conflict`，开着但没配好目标也走这里，`sync/mod.rs:84-86`）；`model_requests_api.rs:44-46,53-62,114`（「job is not active」、两种连接不一致、执行器的 `Conflict`）；`docs/api/00-约定/02-认证与错误.md:36,40-49,51-68`；`docs/api/10-备份/01-数据库备份.md:59-60`。调用方：前端 `backups.ts:38-50`、`sync.ts:76-88` 只读 `message`；Python `rust_executor.py:88-89` 把所有 409 当「需要人工处理」。
- 现状: 本组 5 处 409 的 `details` 都是 `{}`；恢复时「备份文件坏了」「备份来自更新版本」回 400，可请求本身没写错，是那份文件的状态不允许。
- 问题: 违反规矩第 1 条。客户端分不清「有任务在跑，等一会儿再试」和「这份备份永远恢复不了」。`sync/run` 在「开着但没配好」时也说「同步没有开启」，措辞不对。
- 建议: 只增：恢复 `Busy` → `conflict_because("restore_blocked", …)`；`sync/run` → `conflict_because("sync_disabled" | "sync_not_configured", …)` 并分开措辞；模型执行器 → `job_inactive`、`connection_mismatch`。坏备份/新版本备份改 409（`backup_damaged`、`backup_newer`）是改状态码，先和前端、CLI 确认：前端只显示 `message`，CLI `backup.rs:126` 经 `backend.post` 报错也只显示说明，都不看状态码，影响小。文档两处表格补上。对前端的影响: 只多 `reason`；状态码改动前端无需改。
- 等级: 中

### 8. 恢复失败也会先存一份「恢复前」备份并按 3 份清理：连点几次坏备份，真正的恢复前备份被挤掉
- 位置: `backup/mod.rs:149-155`（阻塞闭包里先 `create_backup(KIND_BEFORE_RESTORE)`，再 `restore_backup`，再 `prune_backups`，不管恢复成没成）；`database/retainpdf-db/src/db/backup.rs:60,304-330`（坏文件、版本更新的判断都在 `restore_unpacked` 里，安全备份之后）；文档 `01-数据库备份.md:59-60`。
- 现状: 每次恢复失败（`Damaged`/`Newer`）都多一份 `before-restore`，`KEEP_BEFORE_RESTORE = 3`。
- 问题: 先恢复了一次（留下能退回的那份），再对一份坏备份点三次「恢复」，能退回第一次恢复之前的那份就被清掉了（未实跑）。文档对 400 也没说「但多了一份恢复前备份」。
- 建议: 把解压、`integrity_check`、版本检查挪到安全备份之前（拆出 `Db::check_backup(id)`），检查不过直接拒绝，不留安全备份。对前端的影响: 无。
- 等级: 低

### 9. `PUT /sync` 校验失败时已经写进去一部分；请求体不认识的字段静默忽略；立即同步排队时请求一直挂着
- 位置: `control.rs:183-235`（先校验 transport/folder/url/设备名，**写完** transport、folder、url、用户名、设备名、密码之后，才检查「开启时必须已配好」，`:224-231`）；`sync/mod.rs:60-64`（不管校验结果都 `wake.notify_one()`）；`retainpdf-core/src/models/sync.rs:6-20`（`SyncSettingsInput` 没有 `deny_unknown_fields`）；`sync/mod.rs:81-83`（`run_once` 排在锁后面）、`docs/api/09-同步/01-多设备同步.md:113`。
- 现状:
  - `{"enabled": true, "transport": "webdav", "webdav_url": ""}`：transport 改成 webdav、地址清空，然后回 400「请先填写 WebDAV 地址」；原来开着的同步此时 `enabled` 仍是 1、目标没了，后台静默停掉（未实跑）。
  - `{"webdav_pasword": "…"}` 这类拼错的字段回 200、什么都没改。
  - 已有一轮在跑时 `POST /sync/run` 先等它跑完再跑一轮，HTTP 请求一直开着（文档写了；配合第 1 条的取消问题）。前端运行中禁用按钮（`SyncSettingsPanel.tsx:346`），CLI 不禁。
- 问题: 校验失败应该什么都不改；未知字段违反规矩（`deny_unknown_fields`）。
- 建议: `update` 先把所有校验（含「开启时目标齐不齐」，按「给的值叠加已存的值」算，`target(Some(input))` 现成）做完再写，写放进一个事务（`sync_state_set` 加一个批量版本）；`SyncSettingsInput` 加 `#[serde(deny_unknown_fields)]`（前端 `sync.ts:58-67`、CLI `cmd/sync.rs:134-160` 只发已知字段，不受影响）。`sync/run` 在已有一轮在跑时可以考虑直接回当前状态（`running: true`）而不是排队，这是行为变化，先和前端确认。对前端的影响: 无（前端发的都是合法组合）。
- 等级: 低

### 10. 上传：整份读进内存之后才看容量、还多拷一份；类型只看扩展名；几个边界报错不准
- 位置: `backend/api/src/routes/uploads.rs:27-41`（`read_multipart_field_limited` 读满后 `data.to_vec()` 再拷一份）；`routes/common/extractors.rs:168-188`；`retainpdf-ingest/src/uploads/service.rs:73-88`（扩展名检查、`admitted`、`reserve_buffer` 都在整份读完之后）；`staging.rs:22-39,48-61`；`pdf.rs:13-32`；`retainpdf-ingest/src/lib.rs:24-33`；文档 `backend/api/docs/api-spec/jobs-submission.md:14,52-62`。
- 现状:
  - 单个字段限 512 MiB（`RUST_API_UPLOAD_MAX_BYTES`），边收边数，超了 413，这点没问题；但收的时候没有并发上限，N 个同时上传先各占满 512 MiB，读完才在 `admitted`/`buffers` 上被拒成 503。文档已承认「multipart buffering happens before admission」。`Bytes::from(Vec)` 之后 `to_vec()` 又拷一份，峰值翻倍。
  - 类型只看文件名以 `.pdf` 结尾；内容不是 PDF 时 `lopdf` 解析失败 → 起 Python 修复（最长 60 秒，并发 2），失败才回 400。0 字节文件也走这条路（`"empty upload"` 那条报错实际走不到，`uploads.rs:47`）。
  - 文件名过长（>255 字节）`fs::write` 报 `ENAMETOOLONG` → `UploadError::Io` → 500，应为 400。
  - 503（忙、排队超时、修复超时）没有 `Retry-After`，也没有 `reason`。
  - multipart 里不认识的字段静默跳过（流式丢弃，不占内存）。
  - 失败清理：`PendingUpload` 的 drop 删掉本次新建的目录，修复中间文件在同一目录里一并删掉；发布入库失败也删。这部分是对的。
  - 文档的返回示例没有 `document_id`（`jobs-submission.md:52-62`）；`docs/api/` 下没有上传这一页，只在 OCR、建任务里顺带提到。
- 问题: 单机下风险小（本机用户自己传）；多用户已冻结。主要是峰值内存和几个报错码不准。
- 建议: 在读 multipart 之前先拿 `admitted` 许可（读之前就能拒成 503）；`to_vec()` 改成 `Vec` 直接传（`read_multipart_field_limited` 回 `Vec<u8>` 或让 `UploadedPdfInput.bytes` 收 `Bytes`）；写文件前查前 5 个字节是不是 `%PDF-`（真坏的 PDF 头一般还在，修复照走），不是就 400；0 字节直接 400 `empty upload`；文件名按字节截到 200 以内再落盘（`filename` 字段仍回原名）；503 加 `Retry-After: 2`。文档补 `document_id`，`docs/api/` 加一页上传。对前端的影响: 无；非 PDF 文件改成更早得到 400。
- 等级: 低

### 11. 模型执行器内部接口：用了 axum 原生 `Json`/`Path`，报错不走统一信封、未鉴权先解析请求体；签发令牌的接口没人调
- 位置: `backend/api/src/routes/model_requests.rs:10-13,25-29,34-38`（`axum::extract::{Path, Json}`，不是 `ApiPath`/`ApiJson`）、`:17-24`（Bearer 长度必须 64）；`model_requests_api.rs:79-83`（注释「先鉴权，再查任务、校验」）；`retainpdf-core/src/model_connection.rs:21-39`、`retainpdf-providers/src/model_executor/policy.rs:17-30`（两边都 `deny_unknown_fields`，不含密钥，只有 `credential_ref`）；`retainpdf-runner/src/job_runner/worker_process.rs:107-123`（任务运行时自己生成令牌、直接 `create_model_session`）；Python `rust_executor.py:79-98`。
- 现状:
  - 请求体写错时回的是 axum 的纯文本 400/415/422，不是 `{code, message, error}` 信封；serde 的报错原文可能带上请求里的值（`invalid type: string "…"`），`ApiJson` 专门避免了这一点（`extractors.rs:14,47`）。回给的是调用方自己发的内容，不算泄露，但和全站约定不一致。
  - worker 那组挂在认证之外：提取器顺序是先解析 JSON 再进处理函数验令牌，没有令牌的请求也能拿到「哪个字段不认识」的回答，和注释「先鉴权」不符（只泄露结构，不泄露数据）。
  - Python 客户端把非 2xx、非 401/403/409 一律当「执行器暂时不可用，用同一个操作号重试」，请求体真写错时会一直重试到期限。
  - 返回不带信封（裸 JSON），和其它接口不一样；作为内部接口可以接受，但接口目录、文档里没写。
  - `POST …/capability` 没有生产调用方（只有两处测试），能力令牌实际由运行时进程内直接发。
- 问题: 接口层的小不一致；签发接口多一个能铸令牌的入口（要应用 API 密钥，多用户下 `closed`）。
- 建议: 四个处理函数换成 `ApiPath`/`ApiJson`；worker 三个把 `token(&headers)?` 和 `authorize` 挪到解析请求体之前（`submit` 先收 `Bytes`，验完令牌再 `serde_json::from_slice`）；`capability` 接口确认没有外部部署在用后删掉（连同 `ModelRequestsApi::issue`、两处测试）或在登记表说明里写明「只给测试用」；接口目录给这一组注一句「返回不带信封」。Python 客户端对 400 改成直接报错（`backend/translation`，改了要跑翻译测试）。对前端的影响: 无（前端不调）。
- 等级: 低

### 12. `/health`、`/ready`：返回字段没文档；`/ready` 503 时信封仍是 `code: 0`；读库失败计数当 0；「只有 /health 免密钥」的说法不全
- 位置: `routes/health.rs:12-27`（503 也用 `ApiResponse::ok`）；`health_api.rs:76-96`（`count_jobs_with_status(..).unwrap_or(0)`）；`docs/api/00-约定/02-认证与错误.md:5`、`docs/core/api/index.md:15`、`backend/api/docs/api-spec/runtime-and-events.md:153`（「除 `GET /health` 外都要 `X-API-Key`」）；`app/router.rs:62-66`（`/ready` 和模型执行器 worker 三个也在认证之外）。
- 现状: 返回里没有路径、版本号、密钥，`provider_backends` 是代码里写死的 OCR 服务商名单，`ai_service`/`jobsd` 是固定的状态词，`reasons` 是固定代码——没有泄露内部信息。`queue_depth`、`running_jobs` 是业务量，单机下无所谓；多用户下 `/health` 在公网可读（冻结，只记）。
- 问题: 库读不出来时 `queue_depth: 0` 看着像「空闲」（`status` 会是 `degraded`，CLI `cmd/status.rs:117-118` 只看计数）；`/ready` 503 + `code: 0` + `message: "ok"` 让只看信封的客户端误判（Docker 用 `curl -f` 看状态码，不受影响）。文档没写返回字段，桌面和 CLI 依赖的 `data.status === "up"`、`queue_depth` 没被钉住（`api_tests/health.rs` 只测了 ready 的组件逻辑和状态码）。
- 建议: 计数失败时回 `null`（`Option<i64>`，只改值不改键；CLI 已经 `unwrap_or(0)`）；`/ready` 不就绪时信封可以保持（改信封会影响 CLI `context.rs:157` 的 `data` 读取），但文档写明「看 HTTP 状态码」；`docs/core/api/index.md` 补两个接口的返回字段；三处认证说明改成「除 `/health`、`/ready` 和 `/api/v1/internal/model/jobs/*/requests*`（凭任务令牌）外」。对前端的影响: 无。
- 等级: 低

### 13. 契约和约定：本组都没有 schema，前端类型手写；不分页的列表没在约定里写明；几处说明不准
- 位置: `contracts/`（没有 backups、sync、uploads、health 的 schema）；`frontend/packages/api/src/backups.ts:10-36`、`sync.ts:7-74`（手写类型）；`docs/api/00-约定/03-分页.md:51`（只写了 `/collections` 不分页）；`docs/api/00-接口目录.md:72`（上传「不限请求体大小」，实际文件字段限 512 MiB，只是不受框架的默认请求体上限）、`:16`（`/ready`「AI 服务、jobsd 都就绪」，实际只在受监督时才要求）。
- 现状: 手写类型和后端视图目前一致（逐字段对过：`BackupStatusView`、`BackupItemView`、`BackupRestoreView`、`SyncStatusView`、`SyncRunView`、`SyncMaintenanceView`、`SyncTestView`）。`BackupStatusView.dir` 回的是服务器上的绝对路径，前端用来显示「备份放在 …」（`BackupPanel.tsx:201`），单机下是用户自己的目录，保留合理。
- 建议: 量小、改动少，暂不加契约；在 `03-分页.md` 补「`/backups` 的 `items`（受保留规则限制，最多约 27 份）不分页」；接口目录两句说明在登记表里改（`app/router/ingestion.rs:9` 改成「不受默认请求体上限，文件限 `RUST_API_UPLOAD_MAX_BYTES`」；`public.rs:10` 改成「能接请求了（受监督的 AI 服务、jobsd 就绪）」），`UPDATE_ROUTES=1` 重新生成。对前端的影响: 无。
- 等级: 低

## 查过、没问题的

- `backup_id` 路径参数不能穿越：`backup_path`（`backup.rs:260-267`）拒绝含 `/`、`\` 的编号，还要求整个文件名能按 `<种类>-<时刻>-v<版本>.db.gz` 解析回来（种类在白名单里、时刻能解析），`..`、`%2F` 解码后都过不了；恢复前还要在 `list_backups` 的结果里精确命中（`backup/mod.rs:136`）；有测试 `restore_backup("../../etc/passwd")` → `NotFound`（`backup.rs:436`）。删除走同一个 `backup_path`。
- 备份写临时文件再改名，半份文件不会被当成备份；中断留下的 `.tmp-*` 超过一小时由 `prune_backups` 清掉。备份和恢复在同一把锁下串行，自动备份也走这把锁。
- 恢复的坏文件、版本更新的判断在动正在用的库之前，被拒时库不变（有测试 `backup.rs:442-467`）；换库是一步 `Backup::step(-1)`，别的连接不会读到一半新一半旧；恢复出来的 queued/running 任务清 pid、进程内模式立即对账（有测试 `api_tests/backups.rs:86`）。
- 同步设置的密码不回给客户端：`GET /sync` 只回 `webdav_has_password`（有测试 `api_tests/sync.rs:106`）；WebDAV 地址里写账号密码在保存和测试时都被拒（`control.rs:93-105`），所以 `webdav_url`、`sync_root`、`location`、错误信息里的地址都不带账号；同步数据不含 `sync_state`；WebDAV 跨主机重定向时 reqwest 会去掉 `Authorization`。小瑕疵：`sync/test` 校验失败时 `location` 原样回填调用方自己发的地址（`control.rs:267`），如果地址里写了密码会回显给发送者本人，不涉及第三方。
- `sync/test` 放在阻塞线程里；`sync/test` 和同步都只访问 `http`/`https`。
- 同步文件夹不能在数据目录里或包含数据目录（`control.rs:247-250`），必须是绝对路径、真实存在、可写。
- 上传：文件字段边收边数，超限 413 且不落盘；文件名只取最后一段，拒绝 `..`、`/`、`\`、NUL，落在每次新建的独立目录里（`create_dir` 不复用已有目录）；修复子进程 60 秒超时、超时杀掉并回收，stdout/stderr 不回给客户端；内部错误只回 `error_id`；HTTP 断开后已进入处理的上传照样做完、容量不提前释放。
- 模型执行器 worker 接口：令牌 64 位十六进制、库里只存哈希、绑定单个任务和过期时间，`status`/`cancel`/`submit` 都先验令牌；连接快照不含密钥（`credential_ref`），请求体两种类型都 `deny_unknown_fields`；`job_id` 不存在和读错分开（`model_requests_api.rs:30-43`）。
- 代理（AI）能力令牌访问不了备份和同步：`requested_action` 是白名单（`retainpdf-ai/src/agent_capabilities.rs:278-317`）。
- 多用户相关（`admin` 策略、`/health` 在公网可读）只看了有没有，按约定不细查。
