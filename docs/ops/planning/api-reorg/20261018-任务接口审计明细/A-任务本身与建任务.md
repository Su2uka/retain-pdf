# 任务本身与建任务（lifecycle + entry）

审计范围：`/jobs`（列表、新建、详情）、`POST /uploads`、`POST /translate/bundle`、`GET /documents/:document_id/jobs`。
只读代码和现有快照得出，没有跑 cargo、没有调用运行中的服务。标「未实跑」的是读代码推断的结论。

## 接口一览

| 方法 地址 | 处理函数 | 返回类型 | 契约 | 文档 | 调用方 | 今日调用 |
|---|---|---|---|---|---|---|
| GET `/api/v1/jobs` | `routes/jobs/lifecycle.rs::list_jobs` | `JobListView { items, invocation_summary }` | `job-status.v1`（`JobListView`，另有一份在 `library-books.v1`）；`job_view_contract.rs` 钉键集合 | `docs/api/01-任务/03-任务列表.md`（陈旧）、`docs/core/rust_api/07-任务列表接口.md`（陈旧）、`api-spec/jobs-query.md` | 前端 `frontend/packages/api/src/jobs.ts:77 fetchJobList`，只有任务中心（`features/task-center/domain/task-center-api.ts:51`）和书库回退路径（`recent-jobs/pagination.ts:80`）用 | 21 |
| POST `/api/v1/jobs` | `lifecycle::create_job` | `JobSubmissionView` | 请求：`create-job.v1`（Rust 测试双向钉住）；响应：无契约 | `docs/api/01-任务/01-创建任务.md`（响应示例过简）、`api-spec/jobs-submission.md` | 前端 `frontend/packages/api/src/jobs-submit.ts:51 submitJobRequest`（返回 `Promise<any>`） | 8 |
| GET `/api/v1/jobs/:job_id` | `lifecycle::get_job` | `JobDetailView` | `job-status.v1`（`JobDetailView`），`job_view_contract.rs` 钉键集合 | `docs/api/01-任务/02-查询任务详情.md`（陈旧）、`api-spec/jobs-query.md` | 前端 `jobs.ts:46 fetchJobPayload`、任务中心 | 29 |
| POST `/api/v1/uploads` | `routes/uploads.rs::upload_pdf` → `retainpdf-ingest` `store_upload_view` | `UploadView` | 无 | `api-spec/jobs-submission.md`（少 `document_id`）、`03-OCR/02-OCR-only任务.md` | 前端 `config-port.ts:7`、`jobs-submit.ts:41` | 0 |
| POST `/api/v1/translate/bundle` | `lifecycle::translate_bundle`（只挂在 42000 端口的 simple 路由） | `JobSubmissionView` | 无（flat multipart 字段没有契约） | `docs/core/api/index.md:901`、`api-spec/jobs-submission.md:254`、`delivery/README.md:485` | 仓库内没有任何调用方（前端、`backend/cli`、`backend/ai`、脚本都没有），只有测试和文档 | 0 |
| GET `/api/v1/documents/:document_id/jobs` | `routes/library_data.rs::list_document_jobs_route` | `DocumentJobListView { items, invocation_summary, total, limit, offset, has_more }` | 无（`DocumentJobListView` 不在任何 schema 里） | `api-spec/jobs-document-history.md:84` | 前端 `frontend/packages/api/src/documents.ts:402 fetchDocumentJobs`，类型手写 | 5 |

## 问题

### 1. 建任务请求的 `runtime.job_id` 由客户端指定，没有任何校验，且落库是 upsert
- 位置: `backend/packages/retainpdf-core/src/models/input/resolved.rs:46-50`（`ResolvedJobSpec::from_input` 直接用 `input.runtime.job_id.trim()`）；`backend/packages/retainpdf-core/src/storage_paths/job_paths.rs:30-31`（`output_root.join(job_id)`）；`database/retainpdf-db/src/db/job_writes.rs:57-84`（`INSERT ... ON CONFLICT(job_id) DO UPDATE`）；`contracts/create-job.v1.schema.json`（`RuntimeInput.job_id` 是裸 `string`）；`backend/api/src/routes/job_requests/fields.rs:34`（bundle 的 flat 字段 `job_id` 也写进去）。
- 现状: `POST /jobs` 的 JSON 里 `runtime.job_id` 是公开字段，非空就原样当作新任务编号。`validate_runtime_limits`（`lifecycle/validation.rs:346`）只查两个超时，没人查 `job_id` 的格式或是否已存在。全仓搜 `runtime.job_id` 没有任何拒绝逻辑。`persist_and_launch` 里的 `persist_job_with_resources` → `save_job` 是 `ON CONFLICT DO UPDATE`。
- 问题: （读代码推断，未实跑）带上已存在任务的编号会把那一行的请求、状态、命令整行覆盖，等于「重置并重跑别人的任务」，没有 409。编号里带 `..` 或绝对路径时，`output_root.join(job_id)` 会在输出目录外建目录（`create_all`）。单机模式只有自己能伤自己；多用户模式（已冻结）下是越权覆盖。前端从不发这个字段（`payload.ts` 里没有），所以它是纯暴露面。内部用它的地方（runner、重试、续跑）都是后端自己写，不经 HTTP。
- 建议: 在 `CreateJobInput` 的公开入口（`create_job` / `parse_translate_bundle_request` 之后、`submit_job` 之前）拒绝非空 `runtime.job_id`（400，"runtime.job_id is not accepted"），或者至少校验格式（`^[A-Za-z0-9_-]{1,64}$`）并在 `submit_job` 里查重（已存在则 409）。同步把 `contracts/create-job.v1.schema.json` 的 `RuntimeInput.job_id` 去掉或标明只读（Rust 的 `create_job_contract` 测试会要求 Rust 字段集合与 schema 一致，所以要连 `RuntimeInput` 一起改，建议只在入口处拒绝而不动结构体）。`submissions.txt` 补一个「带 runtime.job_id」的用例。对前端的影响: 无（前端不发）。
- 等级: 高

### 2. `GET /jobs` 的分页没有元数据，分页风格和同族接口三种各异
- 位置: `backend/packages/retainpdf-jobs/src/views/job_types.rs:607-620`；`presentation/listing.rs:22-38`；`query.rs:33-47`；`views/common.rs:80-106`；`progress/event_feed/mod.rs:35`。
- 现状: `JobListView` 只有 `items` + `invocation_summary`，没有 `total/limit/offset/has_more`。同一个 `JobListItemView` 的另一个出口 `GET /documents/:id/jobs` 有 `total/limit/offset/has_more`；事件接口走 `cursor/next_cursor/has_more/limit`（不支持 `offset`，且 `ListJobEventsQuery` 有 `deny_unknown_fields`）；译文条目列表用 `limit/offset`。`limit` 被静默夹到 `1..500`（`query.limit.clamp(1, MAX_JOB_LIMIT)`），响应里不回显生效值。排序是 `updated_at DESC, job_id DESC`（`database/retainpdf-db/src/db/jobs.rs:96`），按 offset 翻页，运行中的任务每次更新都会改排序位置。
- 问题: 客户端无法知道有没有下一页。前端实际在猜：`task-center-api.ts:68` 用 `pageItems.length === limit` 当 `hasMore`，并在 `:57-63` 用 `seen` 集合手动去重（页间重复就是 offset + 可变排序的后果）；`recent-jobs/pagination.ts:98` 读 `payload.has_more`，对 `/jobs` 永远是 `undefined`，落到「短页启发式」。
- 建议: 给 `JobListView` 只增不改地加 `limit`（生效值）、`offset`、`has_more`（可用多取一条的方式算，不必 `COUNT`）；`total` 可选（`select_jobs` 是流式过滤的，要算 total 需另做 count，非必须）。契约 `job-status.v1`、`library-books.v1` 的 `JobListView` 同步加字段（`job_view_contract.rs` 用 `assert_exact_object_keys` 钉，必须同一分支改）。长期可考虑给 `/jobs` 加 `cursor`（按 `updated_at,job_id` 键集分页）与事件接口一致，但这是新增，不是必须。对前端的影响: 无（只增字段）；前端可以改成读 `has_more`，去掉 `length === limit` 猜测。
- 等级: 中

### 3. `ListJobsQuery` 在 `/jobs` 和 `/library/books` 共用，各自静默忽略不同的参数；未知参数一律静默丢弃
- 位置: `views/common.rs:80-106`（`q`、`job_ids`、`include_live_stage` 字段）；`backend/packages/retainpdf-jobs/src/query.rs:33-47`（`list_jobs_filtered` 只用 status/workflow/provider/limit/offset/owner）；`backend/packages/retainpdf-library/src/book_projection.rs:160-200`（书库用 `q`、`job_ids`、强制 `exclude_ocr`，不看 `workflow`）；`backend/ai/retainpdf_ai/tools.py:218-221`。
- 现状: `GET /jobs?q=...` 和 `?job_ids=...` 会被解析成功然后丢弃。`ListJobsQuery` 没有 `deny_unknown_fields`（事件查询有），所以 `?document_id=x` 之类也静默忽略；AI 服务的代码注释已经记了这个坑：「实测那个过滤被静默忽略，传一个不存在的 document_id 照样返回全部 42 条」。前端 `fetchJobList` 会发 `q`（`jobs.ts:98`、`pagination.ts:80`），在 `/jobs` 上等于没过滤。`docs/core/api/index.md:347,734` 与 `docs/core/rust_api/07-任务列表接口.md:163` 的示例带 `status=&provider=` 空值，`status`/`workflow` 是枚举，空串很可能 400（未实跑，需验证）。`include_live_stage=false` 只在 `/jobs`、`/library/books` 有，`/documents/:id/jobs` 没有。
- 问题: 同名参数在两个相邻接口上语义不一样且无提示；调用方（含 AI 服务）会被静默的「不过滤」坑。
- 建议: 在 `/jobs` 上让 `q`、`job_ids` 生效（`select_jobs` 已经支持 `job_ids` 与 `include_upload_filename`，几行的事），或者拆出 `/jobs` 专用的查询结构并加 `#[serde(deny_unknown_fields)]`。若加严格会让带 `q` 的前端调用 400，所以要分两步：先让 `q` 生效，再考虑严格。`status=` 空串可用 `#[serde(deserialize_with)]` 当作未设置。对前端的影响: 让 `q` 生效对前端无破坏；加 `deny_unknown_fields` 则 `fetchJobList` 里没有任何多余参数（它只发 limit/offset/status/workflow/provider/q/include_live_stage），不会受影响。
- 等级: 中

### 4. 用户文档大面积陈旧：列表、详情、创建、上传、事件分页
- 位置:
  - `docs/api/01-任务/03-任务列表.md:14-31`：列表项「应包含」`display_stage/stage/substage/lane/progress`，这些顶层字段已被 `assert_no_legacy_top_level_stage_fields` 禁止，现在在 `stage_snapshot` 里；查询参数一节只说「以接口返回为准」，没列 `limit/offset/status/workflow/provider/q/include_live_stage`。
  - `docs/api/01-任务/02-查询任务详情.md:12-52`：示例整个是顶层 `display_stage/stage/substage/lane/stage_detail/progress/background_stages`；真实字段是 `stage_snapshot`、`background_snapshots`（`JobDetailView`，`job_types.rs:99-137`）。
  - `docs/core/rust_api/07-任务列表接口.md:49-143`：同样的顶层阶段字段；`workflow` 只列 `book/ocr`（实际还有 `translate/render`）；「列表不返回 `failure.summary`」与现状相反（失败任务带精简 `failure`）；「每个 item 固定包含 `stage`」错。
  - `docs/api/01-任务/01-创建任务.md:50-60`：响应示例只有 `job_id/status`，实际还有 `workflow/ocr_reused/source_artifact_job_id/stages/links/actions`（及混合页码拆任务时的 `sibling_job_ids`）。`source.source_url` 写成通用字段，实际只有 `workflow=ocr` 支持，翻译类要求 `upload_id`（`prepare.rs:171-174`）。
  - `backend/api/docs/api-spec/jobs-submission.md:~50-62` 上传响应示例缺 `document_id`（`UploadView` 有）。同文件 `:254` 把 bundle 叫「synchronous multipart helper」，而 `docs/core/api/index.md:905` 与 delivery README 明确说它不等待。
  - `docs/core/api/index.md:385`（`progress/events?limit=200&offset=0`）：事件接口不认 `offset`（`deny_unknown_fields`，且 `jobs_query_contract.rs:140` 断言响应没有 `offset`），照抄会 400。（属进度组，顺带记录）
  - `backend/api/docs/submissions.txt` 里的用例标题还写旧地址 `/jobs/:id/retry-stage`、`/jobs/:id/rerun`、`/jobs/:id/ocr-ambiguity/resolve`（`submissions.rs` 用例名），不影响行为，只是快照可读性。
- 问题: 文档按旧结构写，照着做的调用方会读空字段。
- 建议: 重写 `03-任务列表.md`、`02-查询任务详情.md` 的字段段落，直接指向 `contracts/job-status.v1.schema.json` 的 `JobListItemView`/`JobDetailView`；`07-任务列表接口.md` 要么重写要么在开头标「已被 docs/api/01-任务/03-任务列表.md 取代」并删旧内容；补创建响应和上传响应示例。文档与代码在同一分支改。对前端的影响: 无。
- 等级: 中

### 5. 响应侧契约缺口：创建、上传、文档任务列表都没有 schema，前端手写类型
- 位置: `contracts/create-job.v1.schema.json:12-18`（只有请求；`rust_view` 还写着已不存在的 `routes/jobs.rs :: create_job`，现在是 `routes/jobs/lifecycle.rs`）；`contracts/*.schema.json` 中搜不到 `JobSubmissionView`、`UploadView`、`DocumentJobListView`；`frontend/packages/api/src/jobs-submit.ts:51` 返回 `Promise<any>`；`frontend/packages/api/src/documents.ts:97-109` 手写 `DocumentJobSubmissionView`、`DocumentJobsView`。
- 现状: 请求体契约做得很好（`request.rs` 的 `contract_tests` 双向钉字段集合、未知字段探针、`backend/contracts` 镜像对等）。但 `POST /jobs` 的响应、`POST /uploads` 的响应、`GET /documents/:id/jobs` 的响应（含 `total/limit/offset/has_more`）都不在任何契约里，也没有 Rust 测试钉键集合。`JobListItemView` 在 `job-status.v1` 和 `library-books.v1` 里各有一份（`job_view_contract.rs:316,369` 分别钉），两份要手工保持一致。
- 问题: 前端对这三个响应没有类型安全，改字段不会有红灯。
- 建议: 新增 `job-submission.v1`（`JobSubmissionView`，建任务三个入口共用：`/jobs`、`/translate/bundle`、`/documents/:id/translate|ocr`）和上传、文档任务列表的定义，在 `job_view_contract.rs` 旁加键集合断言。`create-job.v1` 的 `rust_view` 改成现路径。对前端的影响: 无（只加契约；前端可以逐步换掉手写类型）。
- 等级: 中

### 6. 建任务的错误形状不统一：顶层 `code` 有时是数字有时是字符串；同一种「源任务不存在」两种报法
- 位置: `backend/packages/retainpdf-api-base/src/error.rs:312-420`（`OcrArtifactReuse` 等变体的顶层 `code` 是字符串，通用错误是数字 40000/40400）；`lifecycle/creation/prepare.rs:96-101`（render 找不到源任务：通用 `NotFound` "artifact job not found"）对比 `lifecycle/ocr_artifact_reuse.rs:63-70`（translate/book 找不到：`OCR_JOB_NOT_FOUND`，带 `reason`、`can_fallback_to_ocr`）。
- 现状: 同一个 `POST /jobs`，`source.artifact_job_id` 指向不存在的任务，`workflow=render` 回 `{code:40400, error:{code:"NOT_FOUND"}}`；`workflow=book|translate` 回 `{code:"OCR_JOB_NOT_FOUND", reason:"job_not_found", can_fallback_to_ocr:true, error:{...}}`。`api-spec/jobs-submission.md:230-240` 已把「顶层字段保留、新代码看 `error.code`」写明，是有意的兼容。
- 问题: 客户端若把顶层 `code` 当数字（信封里其余地方都是数字），在 OCR 复用错误上会类型错。render 路径也没有稳定的字符串码，只能靠 message 文本区分。
- 建议: render 路径找不到源任务也走稳定码（例如复用 `OCR_JOB_NOT_FOUND` 的同族码或新增 `ARTIFACT_JOB_NOT_FOUND`），让「源任务不存在」在三种 workflow 上同码。顶层 `code` 的数字/字符串混用是全局设计，不在本组改，只在文档里把「看 `error.code`」作为唯一规范。对前端的影响: 前端目前用顶层 `code`/`message` 展示错误；render 的 404 码字符串变化只在 `error.code`，旧字段不动，无破坏。
- 等级: 中

### 7. 数据库读错误被一律说成 404
- 位置: `backend/packages/retainpdf-api-base/src/job_loading/mod.rs:24-27`（`load_job_or_404`）；`lifecycle/creation/prepare.rs:15-17,96-101`（`load_upload_or_404`、render 源任务）；`lifecycle/queries.rs:19-21`（`get_document` 任何错误→"document not found"）。对照 `database/retainpdf-db/src/db/jobs.rs:164-178`（`get_job` 特意区分「没有这条记录」和「有记录但读不出来（进程版本比写入方旧）」，注释写明「报成 not found 会让人去查错地方」）。
- 现状: `db.get_job(...).map_err(|_| AppError::not_found(...))` 把区分丢了。`GET /jobs/:id` 对「行存在但解码失败」「库被锁」都返回 404 `job not found`。同一个任务在列表里会被 `select_jobs` 跳过（`eprintln!("[db] skipping malformed job row")`），在详情里是 404，在 `GET /documents/:id/jobs` 里是整个请求失败（`jobs.rs:240 for row in rows { jobs.push(row?) }` → 500，`list_jobs_for_document` 没有跳过坏行的逻辑）。
- 问题: 排障信息被吞；三个读取点对同一条坏行三种表现。
- 建议: `load_job_or_404` 里区分 `QueryReturnedNoRows`（404）与其它（5xx，带 `error_id` 的 Internal）；`list_jobs_for_document` 和 `select_jobs` 统一「跳过坏行并日志」或统一「失败」。对前端的影响: 极少数坏数据场景下由 404 变 500；前端对非 OK 都走「读取失败」文案，无功能影响。
- 等级: 中

### 8. `POST /translate/bundle`：无调用方、只在 42000 端口、文档说走同源代理（实际 404）、flat 字段映射与 JSON 契约漂移
- 位置: `backend/api/src/app/router/simple.rs:9`；`backend/api/src/app/router.rs:101-145`（`build_simple_app` 只挂 simple 路由，主应用不含）；`ops/deployment/docker/nginx.conf.template:87-88`（`/api/` → `app:41000`）；`ops/deployment/docker/delivery/README.md:490-498`；`backend/api/src/routes/job_requests/fields.rs`；`multipart.rs`；`lifecycle/submission.rs:36-58`。
- 现状:
  - 全仓无调用方：`frontend/`、`backend/cli`、`backend/ai`、`scripts`、`ops` 里都没有请求它；今日 0 次；只有测试（`api_tests/submissions.rs`、`jobs_create.rs`、`uploads.rs`）和文档。
  - 路由只在 simple 端口（默认 42000）。delivery README 说「这个接口是由前端同源代理转发的…默认路径 `/api/v1/translate/bundle`」，但 nginx 把 `/api/` 都转到 41000，41000 没有这个路由，`$HOST/api/v1/translate/bundle`（40001）会 404（未实跑，依据是路由挂载）。
  - 它在多用户模式下 `closed`；`routes.txt` 里看不出它在另一个端口。
  - flat 字段分发（`fields.rs`，约 95 个字段名）手写，和 `CreateJobInput` 漂移：没有 `translation.page_ranges`（flat 的 `page_ranges` 写到 `ocr.page_ranges`，语义不同——OCR 的字符串范围 vs 翻译的整数列表）、`translation.execution_connection`、`accepted_ambiguous_request_risk`、`glossary_name`、`runtime.render_after_translation`。`workflow` 的未知值被静默忽略（`fields.rs:20-33`，注释说「由 job_builders 再强制为 Ocr」已过时），而别的未知字段名是 400；JSON 路径对未知 workflow 值是 400。
  - `workflow=ocr` 的拒绝发生在已经把文件存成 upload **之后**（`submission.rs:46-55`），拒绝也留下一个孤儿 upload。
  - `submit.rs:3` 注释还写「`POST /jobs`、`/translate/bundle`、书库…」这类带文件入口的说明。
- 问题: 一个没有调用方、文档指引错误、字段与主入口漂移的二号入口；`POST /uploads` + `POST /jobs` 已经覆盖同样流程。
- 建议: 二选一。(a) 删除：移除 `router/simple.rs` 的端点、`translate_bundle` 处理函数、`routes/job_requests/`、`create_translation_bundle_submission`、`submissions.txt` 与 `routes.txt` 中的相应行，simple 端口只留 `/health`，文档（`docs/core/api/index.md` §13、`api-spec/jobs*.md`、delivery README、`CONTRIBUTING.md:60`、`backend/api/OCR_Service_API.md`）同改；(b) 保留给脚本用：把 delivery README 的 curl 改成 `:42000`，在接口目录标明端口，并把 `workflow` 未知值改 400、先判 workflow 再存文件。对前端的影响: 无（前端不调）。该决定属于产品取舍，建议由人定。
- 等级: 中

### 9. `JobListItemView` 与 `JobDetailView` 同一概念的字段位置/命名/空值不一致
- 位置: `views/job_types.rs:99-137`（详情）、`:486-532`（列表）。
- 现状（对照）:
  - 书名、页数、原文件名、封面：列表平铺 `display_name/page_count/source_file_name/cover_url/thumbnail_url`；详情放 `book_summary.{title,page_count,source_file_name,cover_url,thumbnail_url,document_id}`。`display_name` ↔ `title` 命名不同。列表项没有 `document_id`，详情有。
  - 时间：列表平铺 `created_at/updated_at`；详情在 `timestamps.{created_at,updated_at,started_at,finished_at,duration_seconds}`；列表没有 `started_at/finished_at`。
  - 重试：列表平铺 `attempt/retry_count/last_retry_at`；详情在 `runtime`（`Option<JobRuntimeInfo>`）。
  - 就绪标志：列表平铺 `output_pdf_ready/markdown_ready/bundle_ready`；详情在 `artifacts`/`actions` 里。
  - 链接：列表 `detail_path/detail_url`；详情 `links.{self_path,self_url,...}`。
  - 失败：列表 `failure` 是精简版且**失败时才出现、否则键不存在**；详情 `failure` 是完整版且**否则为 `null`**，另有 `error`、`failure_diagnostic`（构造函数叫 `job_failure_to_legacy_view`，内容是 `failure` 的子集）三处表达同一件事。
  - `source_artifact_job_id`/`trace_id`/`invocation`/`stage_snapshot`/`background_snapshots`/`stages`/`completion_note`：两边同名同形，这部分一致。
  - `workflow`/`status`/`job_id`：一致。
- 问题: 前端要为同一概念写两套取值路径；空值约定（缺键 vs `null`）在 `failure` 上相反。这些是历史形成的，有契约钉着。
- 建议: 不要改已有字段（前端依赖）。新增方向：(1) 列表项加 `document_id`（`book_document_id` 已经算出来了，`listing.rs:78` 就在手边），书库/详情页不用再靠 `cover_url` 反推；(2) 文档里写清两种形状的映射表（放进 `docs/api/01-任务/03-任务列表.md`）；(3) 详情的 `failure_diagnostic` 标注为弃用，等前端确认没人读再删（`grep failure_diagnostic frontend` 可确认）。对前端的影响: 全部是只增。
- 等级: 中

### 10. 单元测试测的是 `#[cfg(test)]` 里的复制品，而不是线上的 `presentation/` 投影
- 位置: `backend/packages/retainpdf-jobs/src/views/job_builders.rs`（348 行，`job_to_detail`/`job_to_list_item` 都是 `#[cfg(test)]`）；`views/tests.rs`（581 行，`:57,84,112,148,370,470,515` 调 `job_to_detail`）；线上实现在 `presentation/detail.rs`、`detail_projection.rs`、`listing.rs`。
- 现状: 两套并行实现。已经出现分歧：测试版 `ocr_reused = !request_payload.source.artifact_job_id.trim().is_empty()`，线上版 `job.source_job_id().is_some()`；测试版 `background_snapshots: Vec::new()`、自己拼 `test_stage_snapshot`，线上版走 `build_job_stage_view`。
- 问题: `views/tests.rs` 通过时不代表线上字段对；线上改了测试仍绿。真正兜线上的是 `api_tests/job_view_contract.rs` 和 `jobs_live_stage.rs` 等 API 级测试。
- 建议: 把 `views/tests.rs` 里值得留的断言改成调 `build_job_detail_view`/`build_job_list_view`（需要 `Db` 夹具，`EventFeedFixture` 已有），然后删 `job_builders.rs` 的测试复制品。对前端的影响: 无。
- 等级: 中

### 11. 死代码与调试残留（建任务路径、上传路径、限额配置）
- 位置与证据:
  - `lifecycle/creation/prepare.rs:150-158`：`eprintln!("[prepare_ocr_input] no file/upload_id/source_url: ...")` 调试输出 + `return Err(...)`，上面已经是函数末尾，`return ...;` 多余。
  - `lifecycle/creation/job_builders.rs:21-25`：`build_translation_job_snapshot` 里 `WorkflowKind::Ocr => bad_request("workflow=ocr: upload the PDF ...")` 不可达（`submit_job` 在 `submit.rs:93` 已把 Ocr 分流到 `build_ocr_job_snapshot`）；bundle 自己另有一份同文案的检查（`submission.rs:52`）。
  - `backend/api/src/routes/uploads.rs:55-62`：`pub async fn store_upload` 全仓无引用（`routes::uploads::store_upload` 零命中）。
  - `routes/uploads.rs:46` 和 `routes/job_requests/multipart.rs:56` 的 `"empty upload"` 分支不可达（`file_name`、`file_bytes` 同时赋值，先命中的永远是 "missing multipart field: file"）。
  - `backend/packages/retainpdf-core/src/config/limits.rs:56-75`：`max_job_limit()`、`max_job_event_limit()`、`max_document_limit()` 等「读环境变量」的函数无任何调用方，真正生效的是常量 `MAX_JOB_LIMIT`（`query.rs:40`、`listing.rs:50`），所以 `RUST_API_MAX_JOB_LIMIT` / `RUST_API_MAX_LIST_LIMIT` 环境变量是死配置；事件分页还硬编码 `clamp(1, 500)`（`event_feed/mod.rs:35`），不走 `MAX_JOB_EVENT_LIMIT`。
  - `database/retainpdf-db/src/db/jobs.rs:~190 list_jobs`：只有测试在用，生产列表走 `select_jobs`。
  - `core/models/common.rs:52 job_api_prefix` 恒返回 `/api/v1/jobs`，`build_job_links`/`build_job_links_with_workflow` 的 `workflow` 参数是历史遗留（`views/job_builders.rs:46-51` 里 `job_path_prefix` 同）。
  - `presentation/detail_projection.rs:~150`：`build_book_summary(...)` 已经设置了 `cover_url`，后面又 `.with_cover_url(cover_url)` 用同一个函数的结果覆盖，多余。
  - 测试里 `for router in [Router::new()...]` 单元素循环（`multipart.rs` 的测试），是删掉一个入口后的残留。
- 问题: 干扰阅读；环境变量的死配置会误导运维。
- 建议: 逐项清理；环境限额要么接上（`query.rs`/`listing.rs` 改用 `max_job_limit()`），要么删函数并在文档里不要提。对前端的影响: 无。
- 等级: 低

### 12. 同一种「缺来源」两种报错文案，且文案提到不支持的 `file`
- 位置: `prepare.rs:171`（book/translate: `"upload_id is required"`）、`prepare.rs:156`（ocr: `"either file, upload_id, or source_url is required"`）；`submissions.txt:46,98`。
- 现状: 两个都是 400 `BAD_REQUEST`，但 OCR 的提示里有 `file`，而 `POST /jobs` 是纯 JSON、没有 `file` 入口；两者都没带字段路径（`source.upload_id`）。
- 问题: 文案误导；调用方只能靠 message 判断缺哪个字段。
- 建议: 统一成 `source.upload_id is required (upload the PDF with POST /api/v1/uploads first)`；OCR 的改成 `one of source.upload_id or source.source_url is required`。文案变更要同步 `submissions.txt`（`UPDATE_SUBMISSIONS=1`）。对前端的影响: 前端按 message 展示，仅文字变化；若有按文案匹配的代码需 grep（`grep -rn "upload_id is required" frontend`）确认。
- 等级: 低

### 13. 上传与 bundle 的 multipart 解析重复，宽严不一
- 位置: `routes/uploads.rs:16-52` 与 `routes/job_requests/multipart.rs:23-62`、`fields.rs:12`。
- 现状: 两处各写一遍「读 file 字段 + 限长」。`/uploads` 对未知字段静默忽略，只认 `file`、`developer_mode`；bundle 对未知字段名 400。`developer_mode` 两处解析不同（uploads: `1/true/True/TRUE`；bundle: `parse_bool_like` 还认 `yes/on`）。
- 问题: 同样的 multipart 概念两种宽严。
- 建议: 抽出共用的「读文件 part」函数；`/uploads` 对未知字段也 400 的话可能破坏旧客户端，先保持，只统一 `developer_mode` 解析。对前端的影响: 无（前端只发 `file`，见 `jobs-submit.ts:11-14`）。
- 等级: 低

### 14. 命名小不一致：上传大小字段、`document_id` 空值、vestigial 的 `invocation_summary`
- 位置: `retainpdf-ingest/src/uploads/view.rs:6-27`；`views/job_types.rs:591-594`；`BookSummaryView.document_id`。
- 现状: `UploadView.bytes`，而 `BookSummaryView.file_size_bytes`、`MarkdownImageView.size_bytes`、产物清单 `size_bytes`（全仓 `size_bytes` 7 处，`bytes` 5 处）。`UploadView.document_id` 旧记录是空串（注释写明），而 `BookSummaryView.document_id` 无书时是 `null`。`JobListView.invocation_summary { stage_spec_count, unknown_count }` 是协议迁移期的统计，现在所有任务都是 `stage_spec`；前端只透传（`pagination.ts:17`、`documents.ts:109`）。
- 建议: 不改旧字段；`UploadView` 增 `size_bytes`（与 `bytes` 同值）并标 `bytes` 为旧名；`document_id` 空串→`null` 会破坏前端的 `|| ""` 判断，不建议动，文档写清楚即可；`invocation_summary` 等确认没人读后再删（先在文档标弃用）。对前端的影响: 只增。
- 等级: 低

### 15. `JobLinksView` 里 `artifacts_path` 与 `artifacts_manifest_path` 恒等
- 位置: `views/job_builders.rs:42-45`（`let artifacts_manifest_path = artifacts_path.clone();`）。
- 现状: 两组字段值相同（注释写明「两组旧字段名照旧返回，值相同」），都指向 `/files`。
- 建议: 在 `docs/api` 里标注两个旧名等价于 `files` 地址；确认前端不读其中一个后可只保留一个。对前端的影响: 先 grep 前端 `artifacts_manifest`。
- 等级: 低

## 没问题的

- 新建/列表/详情三个地址分在同一类（`router/jobs/lifecycle.rs`、`routes/jobs/lifecycle.rs`），路径参数统一叫 `:job_id`，查询参数全是 snake_case（`include_live_stage`、`job_ids`），登记表、handler、service 三处同名，没有残留旧别名。
- 建任务入口唯一：`create_job`/bundle/书库都走 `submit_job`（`submit.rs`），凭据入库、多用户预扣、归书、发射各只写一遍；`check_job_entry_ownership` 守着。
- 请求契约 `create-job.v1.schema.json`：每个段 `additionalProperties:false` 与 Rust `deny_unknown_fields` 一一对应；`request.rs` 的 `contract_tests` 双向钉字段集合、未知字段探针、`backend/contracts` 镜像（`check_parity.py`）都有；工作流枚举 `book/ocr/translate/render` 与 `WorkflowKind` 一致。
- 未知请求字段在 JSON 路径上一律 400（`invalid job payload: unknown field ...`），不会静默丢。
- 响应侧：`JobListItemView`、`JobDetailView`、`BookSummaryView` 的键集合由 `job_view_contract.rs` 的 `assert_exact_object_keys` 钉；失败简报 `JobFailureBriefView` 有专门测试（两份 schema 都验）。
- 列表项 `failure` 与详情走同一分类（`build_list_failure_brief`），没有两种说法；时间戳都是字符串，ID 都是字符串，URL 成对给出 `*_path`（相对）+ `*_url`（绝对），同族一致。
- `GET /jobs` 的 owner 由路由按主体填（`ListJobsQuery.owner` 是 `serde(skip)`），客户端不能越权指定。
- `run_job_query_once` 的 key 带随机后缀，实为一次性执行，不会让不同查询串互相共享结果。
- 上传：大小/页数/并发/修复超时的错误映射清楚（413/400/503），Python 的 stderr 不外泄；`UploadError → AppError` 映射集中在 `retainpdf-ingest/src/lib.rs`。
- `GET /documents/:id/jobs`：`total/limit/offset/has_more` 齐全，`limit` 夹取与排序（`updated_at DESC, job_id DESC`）写进文档，文档不存在是 404。
- 建任务的状态码选择：缺上传 400、上传不存在 404、OCR 复用不可用 409、页数额度 402（`submissions.txt` 钉了），语义清楚。
