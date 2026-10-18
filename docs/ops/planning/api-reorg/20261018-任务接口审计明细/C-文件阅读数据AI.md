# 文件、阅读、数据、AI 读物（files + reader + data + ai）

范围：`router/jobs/{files,reader,data,ai}.rs`、`routes/jobs/{files,reader,data,ai}.rs`、`routes/download_response/`、`retainpdf-jobs` 的 `files/ downloads/`、`reader/`、`data/`、`ai/`，快照 `file-responses.txt`，契约 `reader-data.v1`、`job-data.v1`。只读审计，没跑 cargo，没碰运行中的服务。

## 接口一览

| 方法 地址 | 处理函数 | 返回类型 | 契约 | 文档 | 调用方 | 今日调用 |
|---|---|---|---|---|---|---|
| GET /jobs/:job_id/files | `files::list_job_files` | `JobFilesView`（`{code,message,data}`） | reader-data.v1 `JobFilesView`；`job_view_contract.rs:398` 逐键钉住 | docs/api/07-产物下载/01-下载总览.md；file-responses.txt | frontend `packages/api/src/jobs-artifacts.ts:127`；`frontend-status-smoke.mjs:459` | 63 |
| GET /jobs/:job_id/files/output/*name | `files::job_output_file` | 文件流（`FileDownload`） | 无（二进制） | 同上；file-responses.txt 钉了头 | frontend `reader/domain/dialog/runtime-port.ts:71,78`、`reader/shared/data/resource-resolver.ts:101`、`book-detail/domain/artifact-resources.ts:28`；AI 服务 `backend/ai/retainpdf_ai/tools.py:102`（拼 ocr/images 链接） | 8 |
| GET /jobs/:job_id/files/pages/:page | `files::job_page_file` | JPEG 流 | 无 | 同上；docs/core/api/index.md:625 | frontend `reader/shared/ai/answer-enhance.ts:229` | 1 |
| GET /jobs/:job_id/files/internal/*name（admin） | `files::job_internal_file` | 文件流 | 无 | 同上 | 仅 `frontend-status-smoke.mjs:474,537`；今日 6 次来源未查到前端调用，应为脚本/手动 | 6 |
| GET /jobs/:job_id/reader/markdown | `reader::get_markdown_document` | `MarkdownDocumentView` | reader-data.v1 `MarkdownDocumentView`（字段与 Rust 一致，无专门测试） | 04-任务接口分类.md；api-spec/reader-read-model.md | frontend `packages/api/src/jobs-artifacts.ts:140,149`（两个函数一模一样） | 17 |
| GET /jobs/:job_id/reader/metadata | `reader::get_reader_metadata` | `ReaderMetadataView` | reader-data.v1（一致，无专门测试） | 同上 | frontend `packages/api/src/reader.ts:16` | 10 |
| GET /jobs/:job_id/reader/regions | `reader::get_reader_regions` | `ReaderRegionsView` | reader-data.v1（一致，无专门测试） | 同上 | frontend `packages/api/src/reader.ts:7` | 12 |
| GET /jobs/:job_id/data | `data::job_data_catalog_route` | `JobDataCatalogView` | job-data.v1；`data/tests.rs` 只测登记表自检 | docs/core/api/index.md §7.5；分类表一行 | `backend/api/src/bin/retainpdf-agent.rs:313`（agent CLI 不带 `--dataset`）；frontend 有包装 `packages/api/src/job-data.ts:49` 但没人调用 | 0 |
| GET /jobs/:job_id/data/:dataset | `data::job_data_route` | `JobDataView` | job-data.v1（含 400/404 说明） | 同上 | frontend `quality-list-loader.ts`、`use-editorial-flow.ts:43`、`BookDetailTermsTab.tsx`、`reader/hooks/use-reader-revised-blocks.ts:32`；agent CLI `retainpdf-agent.rs:313` | 352 |
| GET /jobs/:job_id/ai/reading-path | `ai::download_ai_reading_path` | 原样 JSON 文件（不包信封） | 无 | 分类表一行 | 没有。前端已无 `reading-canvas*`；AI 服务不再让 agent 写它 | 0 |
| GET /jobs/:job_id/ai/canvas | `ai::download_ai_canvas` | 原样 JSON 文件（不包信封） | 无 | 分类表一行 | 没有（同上） | 0 |
| GET /jobs/:job_id/ai/board | `ai::list_ai_board` | `AiBoardListing`（包信封） | 无 | 分类表一行 | frontend `reader/ui/terminal-chrome.tsx:295`（4 秒轮询） | 0（今日没开终端） |
| GET /jobs/:job_id/ai/board/:name | `ai::download_ai_board_file` | 文件流 | 无 | 分类表一行 | frontend `reader/ui/board-html.tsx:44` | 0 |

## 问题

### 1. 译文页预览缓存永不失效，还带 `immutable` 一年
- 位置: `backend/packages/retainpdf-files/src/derived_artifacts/preview.rs:76`；`backend/packages/retainpdf-jobs/src/files/downloads/files.rs:38,330-350`；`backend/packages/retainpdf-jobs/src/files/downloads/catalog.rs:73-87`；`docs/core/api/index.md:640`
- 现状: `ensure_page_preview` 只看 `output_path.exists()`，文件名是 `preview-translated-p0003-w1200-d0.jpg`，只由页码、宽度、dpi 决定。响应头 `Cache-Control: public, max-age=31536000, immutable`（file-responses.txt 里 `files/pages/1.jpg`）。同类的对照 PDF、Word 用的是 `cached_output_is_fresh`（`derived_artifacts.rs:63`，比 mtime），只有页预览和封面没有。全仓没找到清理 `preview-*.jpg` 的地方（`grep "preview-"` 只在生成处和测试里出现）。
- 问题: 译文 PDF 是会变的（`relaunch_in_place`、重试渲染阶段、改译文后重渲染都会覆盖 `translated.pdf`），重渲染后同一个地址仍返回旧页图，磁盘缓存和浏览器缓存各一份都是旧的；`immutable` 让浏览器一年内不再问。文档写的"同一个地址内容不变"对译文页不成立（对原文页成立）。另外认证后的内容用 `public` 而封面用 `private`，两处不一致。
- 建议: 译文页预览像 side-by-side 一样比较 `translated.pdf` 的 mtime（过期就重新渲染）；响应头改成 `private, max-age=…` 加 ETag 校验，不要 `immutable`（ETag/304 在 `stream_file` 里已有）。原文页（`/documents/:id/files/pages`）可保持 immutable。对前端的影响: none（地址不变）。
- 等级: 高

### 2. 页码超出总页数返回 500，不是 404
- 位置: `files.rs:325-345`（`JobFile::Page` 分支只检查 `page>=1`）；`preview.rs:140-157`（python 脚本 `page out of range` 抛 RuntimeError）→ `preview.rs:169` 的 `AppError::internal("failed to render page preview")`
- 现状: `catalog.rs` 的 `page_file` 只拒绝 0 和非数字（快照里是 404）。`pages/9999.jpg` 会起一个 python 进程，脚本失败后变成 500 `internal server error (ref …)`。
- 问题: 客户端写错页码是 4xx；500 会进错误日志，还白白起一个子进程。同一个渲染函数也给 `/documents/:id/files/pages/:page` 用，一样的问题。
- 建议: 渲染前用 `/files` 里已经会算的页数（`book_page_count`）判断，越界返回 404 `no such file: pages/N.jpg`，和 `pages/0.jpg` 的报错对齐，并把越界用例加进 `file_responses.rs`。对前端的影响: none（前端只请求 `1..page_count`）。
- 等级: 中

### 3. "还没产出"在本组有五种写法
- 位置: `files.rs:214-230,285-300,355-372`；`reader/markdown.rs:35`；`reader_regions/artifacts.rs:93-96`；`reader_regions/metadata.rs`；`data/mod.rs:560-583`、`data/api.rs:27-36`；`ai/documents.rs:57-59`；`ai/ai_board.rs:168-176`
- 现状（同一个"任务没跑完/产物没出"）:
  - 文件出口: 404，但文案不统一：`pdf not ready`、`translated pdf not ready`、`source pdf not ready`、`markdown not found`、`artifact not ready`；`all.zip` 却是 **409** `job is not finished successfully`（file-responses.txt 两条并排）。
  - `reader/markdown`: 404 `markdown not found`。
  - `reader/regions`: 没有规范化文档就是 **200 + 空 items**（`load_source_region_map` 返回空 map），调用方分不清"还没好"和"确实没有区块"。
  - `reader/metadata`: **200**，`source`/`translated` 为 null；PDF 读不开是 500（`metadata.rs:27-29`）。
  - `data`: 文件不在是 **200 + `available:false`**；任务没有输出目录是 **404** `job has no output directory`（契约只说 404 是"任务或数据集不存在"）。
  - `ai/reading-path`、`canvas`: 404；`ai/board`: **200 空列表**。
  - 清单 `/files` 里这些情况统一是 `ready:false`，没有区分。
- 问题: 前端要为每个接口各写一套"没好"的判断（`fetchJobMarkdown` 把 404 当 null，regions 靠空数组，data 靠 `available`）。
- 建议: 不要强行统一成一种（空列表/`available:false` 对轮询很方便）。但至少: ① 文件出口"没产出"文案统一成 `<name> not ready`，`all.zip` 未成功改成和其它一样的 404，或者全部改 409（二选一，改动后重生成 file-responses.txt）；② `reader/regions` 在任务既没有规范化文档也没有翻译清单时，要么 404 要么加 `ready` 字段（只增不改）；③ `data` 的"没有输出目录"改成 `available:false` 的 200，和契约说明一致；④ 把约定写进 `docs/api/00-约定`。对前端的影响: ① 如果 `all.zip` 改状态码，前端按 409 判断的地方要跟着改（未逐处核对）；其余 none。
- 等级: 中

### 4. `ai/reading-path`、`ai/canvas` 没有任何调用方，建议关掉
- 位置: `router/jobs/ai.rs`；`routes/jobs/ai.rs:20-40`；`retainpdf-jobs/src/ai/documents.rs`；`retainpdf-core/src/storage_paths/constants.rs:12-15`；`ai/ai_carryover.rs`（接力逻辑里还为它们留着）
- 现状: 前端 `frontend/web/src/features/reader/domain/` 里只剩 `board.ts`、`board-html.ts` 等，没有 `reading-canvas*`；`git log` 显示 `bd088531`、`92231982` 把阅读路径、画布并掉了。全仓（frontend、backend/ai、backend/cli）搜 `reading-path`、`ai/canvas` 只剩 `backend/ai/tests/test_fx_workspace.py:252-281`（它用 glob `reading-canvas*.ts` 去找已经不存在的前端文件）和注释。AI 服务生成的 agent 说明（`fx_workspace.py`）只教 agent 往 `./board/` 丢文件。`documents.rs:62-` 的门禁测试还在"守"这两种产物。
- 问题: 两个活着的端点没有读者也没有写者；返回的是 agent 写的原样 JSON，不包 `{code,message,data}`，也不校验，和同类的 `ai/board`（包信封）不一致；`documents.rs` 的"产物只许两种"门禁和接力里的特殊处理都是为它们服务。
- 建议: 在 router 里改成 `closed`（保留 `routes.txt` 一行记录），观察一个版本后删处理函数、`DocumentDownloadKind`、`resolve_ai_canvas`/`resolve_ai_reading_path`、门禁测试、`test_fx_workspace.py` 里那段对前端 glob 的检查；`docs/api/01-任务/04-任务接口分类.md` 的 AI 表删两行。对前端的影响: none（没人调）；需要前端确认 `frontend/web/tests/reader` 里没有别的地方读这两个地址（grep 未发现）。
- 等级: 中

### 5. `ai/board*` 是带副作用的 GET，而且在异步线程里同步做磁盘活
- 位置: `routes/jobs/ai.rs:41-58`；`retainpdf-jobs/src/ai/ai_board.rs:165-170,238-245`；`ai/ai_carryover.rs:65-80,200-269`；对比 `ai/downloads.rs:9-24`（`ai_document` 走 `download_generation.run`）
- 现状: `list_ai_board`、`download_ai_board_file` 在 async 处理函数里直接同步调用，内部 `read_dir`、逐文件 `symlink_metadata`、`canonicalize`，并且每次先跑 `carry_over_ai_workspace`：条件满足时会 `create_dir_all`、写 `.carried-from` 标记、复制最多 500 个文件/64 MB（`ai_carryover.rs:60-61`）。前端 `terminal-chrome.tsx:295` 每 4 秒轮询一次这个列表。
- 问题: ① GET 不是只读的（会建目录、拷文件），任何探测都会触发；② 拷 64 MB 发生在 tokio 工作线程上，会卡住别的请求；③ 同组的 `ai/reading-path` 却走了阻塞线程池，两套做法。代码注释自己也承认"这段代码跑在一个下载请求的线程上"。
- 建议: 两个 board 处理函数同样放进 `download_generation.run`（或 `spawn_blocking`）；接力改到"任务读取路径"之外显式触发（例如阅读页打开时的写接口，或任务完成时），GET 只读。对前端的影响: none。
- 等级: 中

### 6. `/data` 不认合并阅读的虚拟任务 id，也不检查旧目录格式
- 位置: `retainpdf-jobs/src/data/api.rs:27-33`（`deps.db.get_job` + `AppError::not_found("job not found")`）；对比 `retainpdf-api-base/src/job_loading/mod.rs:42-52`（`load_supported_job`）；`api_tests/merged_reading.rs:218-249`
- 现状: 本组其它接口（files、reader/*、ai/*）都走 `load_supported_job`，所以 `merged-<文档>-<指纹>` 这种合并阅读的虚拟 id 能读，旧目录布局的任务返回 409。`/data` 直接查表，虚拟 id 返回 404 `job not found`，旧布局任务则照常去读文件。
- 问题: 阅读器在合并阅读视图下调用 `data/revisions`（`use-reader-revised-blocks.ts:32`）会拿到 404（前端把失败当"没有"吞掉，所以看不出来，改过的块的小标记在合并视图里就是不会出现）。
- 建议: `roots_for` 改用 `load_supported_job`，虚拟任务的根目录由 `load_virtual_job` 给出（需要确认虚拟任务有没有 `translated`/`logs` 目录，没有就都 `available:false`）。对前端的影响: none（只会让合并视图多出数据）。
- 等级: 中

### 7. `/data/:dataset` 的返回形状有几处不一致
- 位置: `retainpdf-jobs/src/data/mod.rs:320-340,575-585,655-693`；`routes/jobs/data.rs:34-39`
- 现状: ① 分页只有 `total/offset/limit`，没有 `has_more`；而 `DocumentJobListView`（`views/job_types.rs:612`）等列表有 `has_more`。② `group_by` 时 `total` 是**筛选后的行数**，`offset/limit` 却是对**分组**分页，没法判断后面还有没有分组。③ 文件不存在且带 `group_by` 时同时返回 `rows:[]` 和 `groups:[]`（`mod.rs:581-582`），存在时只有 `groups`。④ `object` 类数据集返回 `offset:0` 但把请求的 `limit` 原样带回。⑤ 任何未知查询参数（包括前端/代理常见的 `_=时间戳` 缓存参数）都按字段名报 400（`mod.rs:494-503`）；`files` 一组对未知参数却是忽略的。
- 问题: 契约 `JobDataView` 把 `rows/groups/object` 都写成可选，实际组合并不稳定；分页写法和别的列表不同。
- 建议: 加 `has_more`（只增不改）；`group_by` 时 `total` 之外再加 `group_total`（或让 `total` 变成分组数，需前端确认——现在前端只读 `groups`，见 `use-reader-revised-blocks.ts:23`）；不可用时的分支按是否有 `group_by` 只给一个；在契约 `note` 里写清未知参数 400。对前端的影响: 加字段无影响。
- 等级: 低

### 8. 同一份数据有三到四条读取路径，没有写明哪条是标准的
- 位置: `contracts/job-data.v1.schema.json` 的 `datasets`；`router/jobs/{progress,translations,reports,files}.rs`；`retainpdf-files/src/links.rs:11-26`（`JOB_INTERNAL_FILES`）
- 现状（只核对了地址和数据源，没有逐个比对返回形状）:
  - 事件: `progress/events`、`data/events`、`data/pipeline_events`、`files/internal/events.jsonl`。
  - 译文条目: `translations`（`/translations/:item_id`）、`data/translation_items`、`files/internal/translation-manifest.json`，再加 `reader/regions` 里的 `markdown/translated.text`。
  - 修订记录: `translations/:item_id/revisions`、`data/revisions`。
  - 用量、质检: `reports/usage` vs `data/token_usage`；`reports/translation-qa` vs `data/qa_violations`。
  - 原文 Markdown: `files/output/ocr.md`（原始）、`reader/markdown`（JSON 视图，`content` 与 `content_with_absolute_image_urls` 两份全文）、`files/output/ocr.zip`。这三个不算重复，用途不同。
  - 页数: `files.pages.page_count` 与 `reader/metadata.*.page_count`（后者读 PDF，前者读缓存）。
- 问题: 新同学（和 agent）不知道该用哪个；`data` 的描述是"新读需求加登记，不加接口"，但旧接口并没有关，两边改字段容易漂。
- 建议: 在 `docs/api/01-任务/04-任务接口分类.md` 增加一节"同一份数据的标准入口"，明确: 列表/筛选/分组 → `data`；单条读写译文 → `translations`；实时 → `progress/*`；原始文件 → `files/*`（`internal` 仅排查）。对 `reports/usage`、`reports/translation-qa`、`translations` 这类与 `data` 数据集重叠的，由另外两位审计者那边决定是否合并；本组不改。对前端的影响: none。
- 等级: 中（设计层面，没有错行为）

### 9. `dpi`、`width` 的含义随接口变，且没有统一取值范围
- 位置: `views/common.rs:153-170`（`JobFilesQuery`）；`files.rs:67-86`（`translated_docx` 夹在 72–300；`page` 的 dpi 只做 `min(300)`）；`preview.rs:150-153`
- 现状: 同样叫 `dpi`: 对 `translated.docx` 是背景位图清晰度，超出 72–300 会被**静默夹住**；对 `pages/N.jpg` 是渲染倍率，`dpi>0` 时 `width` 被**完全忽略**，`dpi=10` 会渲染出 10 dpi 的小图，不夹下限。`width` 对 `docx`、`output/*` 被静默忽略。三个参数的任意组合都会在 `artifacts/` 下留一个新文件（宽 240–2400 × dpi 0–300 × 页数），没有上限。
- 问题: 查询参数同名不同义；越界静默夹值，调用者无从知道实际用了多少；缓存文件可被无限制地制造。
- 建议: 文档里写清"dpi 优先于 width"（`docs/api/07-产物下载/01-下载总览.md` 已有，但写的是"≤300"没提下限）；页预览 dpi 也夹到 72–300；或者只保留 `width`（`dpi` 现在没有前端调用: 前端只用 `width=240`，docx 在 `book-detail` 里带 `dpi`，需前端确认）。缓存文件数量可以把 `width` 规整到几个档（例如 240/480/960/1200/1800/2400）。对前端的影响: 夹下限无影响。
- 等级: 低

### 10. 契约覆盖不全，也没有测试钉住
- 位置: `contracts/reader-data.v1.schema.json`；`backend/api/src/api_tests/job_view_contract.rs:398`；`backend/contracts/check_parity.py:12-27`；`api_tests/jobs_reader/{regions,metadata}.rs`
- 现状: 手工比对后，`JobFilesView`、`MarkdownDocumentView`、`ReaderRegionsView`、`ReaderMetadataView`、`JobData*` 的字段、必填、可空与 Rust 类型一致，没有发现不匹配。但: ① 只有 `JobFilesView` 有"响应按契约逐键核对"的测试；`reader/markdown`、`regions`、`metadata` 的测试只测行为，没有按契约核对；`data` 的测试只验证登记表自检，没有按 `JobDataView` 定义核对返回。② `ai/board` 的 `AiBoardListing`（`schema`, `items[]`, `skipped`）没有契约，也没有文档（分类表只有一行），前端手写解析（`board.ts`）。③ `check_parity.py` 的 `CONTRACT_NAMES` 缺 `job-data.v1.schema.json` 和 `token-usage.v1.schema.json`（现在两份目录里内容一致，是巧合）。
- 问题: 契约改了、Rust 类型改了，其中一边漂了没有测试会报。
- 建议: 在 `job_view_contract.rs` 里仿照 `JobFilesView` 加三个 reader 视图和 `JobDataView` 的核对；给 `AiBoardListing` 写一个小契约（可并进 reader-data.v1）；`CONTRACT_NAMES` 补上缺的两份。对前端的影响: none。
- 等级: 低

### 11. 文档过期或缺失
- 位置:
  - `backend/api/docs/api-spec/reader-read-model.md:38`: 写着 `POST /jobs/{job_id}/reader/ai/chat` "remains a legacy … endpoint"，`routes.txt` 里没有这个路由（本组 reader 只有 3 个 GET）。
  - `reader-read-model.md:28`: "Job file list" 只列了 `name/url/content_type/ready/on_demand/size_bytes`，缺 `updated_at`、`stage`（`docs/api/07-产物下载/01-下载总览.md` 写全了）。
  - `docs/api/` 下没有 `/data`、`/reader/*`、`/ai/board` 的返回说明页：`data` 只在 `docs/core/api/index.md` §7.5，`reader/*`、`ai/*` 只有 04 号分类表一行。
  - 代码注释里的旧路径: `routes/jobs/ai.rs:3-4,31-32` 写"见 services/jobs/downloads/ai_board.rs"（现在是 `retainpdf-jobs/src/ai/ai_board.rs`）；`reader/markdown.rs` 的注释写"绝对 URL 直接指向 /markdown/images/<rel>"（现在是 `files/output/ocr/images/`）；`ai_board.rs` 头注释说它是"第一个接受调用方给文件名的端点"（`files/output/*name`、`internal/*name` 也收名字）；`catalog.rs` 里 `stage` 的例子 `render`，实际是 `rendering`。
  - frontend 里 `reader/shared/data/reader-regions.ts:379-380`、`answer-enhance.ts:291` 仍兼容旧前缀 `/markdown/images/`，注释说"后端换链接前"，现在后端早已返回新前缀。
- 建议: 修 `reader-read-model.md` 两处，补一页"阅读、数据、AI 读物"的返回说明；顺手改注释。前端那两处兼容代码可以删。对前端的影响: 仅删除死兼容，可选。
- 等级: 低

### 12. 文件出口的几处小硬化/清理
- 位置: `files.rs:361-368`、`catalog.rs:100-110`、`job_helpers.rs:16-21`、`catalog.rs:30,73-87`
- 现状:
  - 所有文件响应都没有 `X-Content-Type-Options: nosniff`（全仓无此头）。`ocr/images/*` 的类型直接由文件扩展名猜（`mime_guess`），可能是 provider 压缩包里的任意文件，没有限定为图片扩展名；`ai/board` 虽然明确不收 svg、并强制 html/pdf 附件，但 `text/plain`、`json` 同样没有 nosniff。
  - `stream_file` 的 404 文案是 `file not found: <磁盘绝对路径>`（`job_helpers.rs:17-20`），NotFound 不像 Internal 那样被屏蔽；正常路径前面已经 `is_file` 过，只有竞态/未预检的路径才会漏。
  - `Registered` 分支的 409 `artifact is a directory…` 实际走不到: `JOB_INTERNAL_FILES` 14 项都是文件，没有目录。
  - `PagePreviewQuery.kind` 与 `preview_kind` 的 400 分支，对任务接口不可达（`page_file` 固定 `translated`），只有书的原文页使用。
  - `ocr.zip`、`all.zip` 默认是 `Content-Disposition: inline`，浏览器实际会下载，行为没问题，但和 `?download=1` 的约定对不上，快照里是 `inline; filename="…markdown.zip"`。
- 建议: 全局加 `nosniff`；`ocr/images` 只放行图片扩展名；404 文案不带路径；删掉不可达的分支。对前端的影响: none。
- 等级: 低

### 13. 路径参数 `:page` 在同类地址里含义不同；`source.url` 用空串表示"没有"
- 位置: `router/jobs/files.rs:14`（`files/pages/:page`，值是 `3.jpg`）vs `router/jobs/progress.rs:15`（`progress/pages/:page`，值是 `3`）；`catalog.rs:200-205`（`source.url` 无书编号时 `unwrap_or_default()` = `""`）；`board` 条目 `size/modified_ms` vs 清单条目 `size_bytes/updated_at`；`ai_board.rs:56` 的 `schema: "retainpdf_ai_board_v1"` vs 前端测试里写的 `ai_board_v1`（`reader-board-html.test.mjs:125`）。
- 现状/问题: 同名参数一个带 `.jpg` 一个不带，`classify_param` 里要分别处理；"没有地址"在 `/files.source.url` 是 `""`，在详情的 `cover_url` 是 `null`；画板条目的大小、时间字段名和格式（毫秒整数）与 `/files` 条目（`size_bytes`、ISO `updated_at`）不一样；`schema` 字符串风格和契约里的 `xxx.v1` 不一致。
- 建议: `source.url` 改 `null`（契约把 `url` 放宽成 `string|null`，前端 `jobFilesToArtifactManifest` 要处理 null，需确认）；画板条目另加 `size_bytes`、`updated_at`（保留旧字段）；`:page` 不动（地址已对外），只在文档里点明。对前端的影响: 加字段无影响；`source.url` 改 null 前端要改 `jobs-artifacts.ts` 里的判断。
- 等级: 低

### 14. 重复实现
- 位置: `reader/markdown.rs:171-210` 与 `reader/reader_regions/artifacts.rs:259-275` 各有一份 `url_path_escape`，前者先解一层百分号再编码，后者不解；`routes/jobs/data.rs:14-20` 自带一个 `blocking` 辅助函数，而同组其它处理函数用 `run_job_query`/`run_job_query_once`；`frontend/packages/api/src/jobs-artifacts.ts:138-155` 的 `fetchJobMarkdown` 与 `fetchJobMarkdownDocument` 逐行相同。
- 问题: 同一个图片文件在 `reader/markdown`（解码一层）和 `reader/regions.asset_urls`（不解码）里，遇到文件名里本来就带 `%` 时会得到不同的 URL。
- 建议: 抽成一个公共函数（放在 `retainpdf-files/src/links.rs` 旁边，统一成"原样编码，不解码"，因为这里的路径来自磁盘上的真实文件名）；`data` 改用 `run_job_query`。对前端的影响: none。
- 等级: 低

## 没问题的

- 路由登记表（`endpoints!`）与 `routes.txt` 一致；这 13 个接口都放对了类别，没有遗留的顶层 `/jobs/:id/<名字>`。
- `files` 这一组的状态码/头由 `file-responses.txt` 钉住: 文件名大小写、`internal/layout.json`、`pages/0.jpg` 都是 404；越界的 Markdown 图片路径是 400；`?download=1` 切换 `attachment`；`Accept-Ranges`、ETag、Last-Modified、Range 都由 `stream_file` 统一处理；Content-Disposition 用 `filename` + `filename*`，转义正确。
- `ai/board` 文件名白名单、拒符号链接、解析后仍在目录内、不收 svg、html/pdf 强制附件，都有测试（`ai_board.rs` 的 `tests`）。
- `JobFilesView` 的 `ready`、`on_demand`、`size_bytes`、`updated_at`、`stage` 与契约 `JobFileEntryView` 完全一致且有逐键测试；`internal` 只有管理员可见，缺省时字段不出现，契约也是可选。
- `reader/regions` 排序、`asset_urls` 的路径检查（`markdown_asset_relative_path` 有测试拒绝 `..` 和绝对路径）正常。
- `data` 的登记表自检、等值筛选、`group_by`、排序、分页、缓存按 mtime 失效，都有测试；错误区分: 未登记数据集 404、未登记字段/不能筛选的字段/类型不对/limit 越界 400，与契约一致；agent CLI 与前端调用的参数名（`fields`、`group_by`、`sort`、`offset`、`limit`）一致。
- Internal 错误的详情被屏蔽成 `internal server error (ref …)`，不会把磁盘路径带给客户端（`error.rs:455`）。
- 本组的查询参数命名: `download`、`dpi`、`width`（文件）与 `fields/group_by/sort/offset/limit`（数据）都是 snake_case，与 `/translations` 的 `limit/offset` 同名同义。
