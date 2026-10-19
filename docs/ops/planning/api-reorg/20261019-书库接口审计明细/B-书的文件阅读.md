# 书的文件、阅读、覆盖和用量

审计范围：`/documents/:document_id/files`（清单、`/:name`、`/pages/:page`）、`/documents/:document_id/reading`（含合并虚拟任务）、
`/documents/:document_id/translation-coverage`、`/documents/:document_id/usage`、`/documents/:document_id/jobs`（只看现在是否合规）、
`/assets`、`/fonts`。
只读代码和现有快照得出，没有跑 cargo、没有调用运行中的服务。标「未实跑」的是读代码推断的结论。

## 接口一览

| 方法 地址 | 处理函数 | 返回类型 | 契约 | 文档 | 调用方（前端文件:行） | 备注 |
|---|---|---|---|---|---|---|
| GET `/api/v1/documents/:document_id/files` | `routes/library_data.rs::list_document_files_route` → `retainpdf-library/src/media.rs::document_files_view` | `DocumentFilesView { document_id, files[], pages{url_template,page_count} }` | 无 schema；`backend/api/docs/file-responses.txt:269` 钉了形状 | `docs/api/07-产物下载/01-下载总览.md:144`、`api-spec/artifacts.md:30` | 仓库内没有调用方（前端都是手拼 `/files/source.pdf` 等地址） | 见问题 11 |
| GET `/api/v1/documents/:document_id/files/:name` | `library_data::document_file_route` → `media.rs::open_document_file` | 文件流（`source.pdf`、`cover.jpg`、`thumbnail.jpg`） | `file-responses.txt:308-332` | 同上，`docs/core/api/index.md:166,578` | `frontend/packages/reader/src/hooks/reader-session/session-helpers.ts:38`、`session-assets.ts:197,371`、`frontend/web/src/features/reader/domain/host/data.ts:220`、`frontend/packages/domain/src/job/actions.ts:192`；封面走后端给的 `cover_url`（`useRecentJobCover.ts:23`） | 见问题 1、4 |
| GET `/api/v1/documents/:document_id/files/pages/:page` | `library_data::document_page_file_route` | 文件流（`3.jpg`） | 无快照 | `下载总览.md:87-88,148` | `frontend/packages/reader/src/shared/ai/answer-enhance.ts:227` | 见问题 1 |
| GET `/api/v1/documents/:document_id/reading` | `library_data::get_document_reading_route` → `retainpdf-library/src/reading.rs::document_reading` → `merge/reading.rs::resolve_reading_target` | `DocumentReadingView { job_id, merged, contributing_job_ids }` | 无 | 只有接口目录一行；`api-spec/jobs-document-history.md:60` 顺带提到 | `frontend/web/src/features/reader/ui/ReaderNavigation.tsx:38`（经 `resolve-reading-target.ts`）、`backend/ai/retainpdf_ai/rust_client.py:79` | GET 里会生成合并目录（跑两个 Python 子命令）。见问题 2、3、10 |
| GET `/api/v1/documents/:document_id/translation-coverage` | `library_data::get_document_translation_coverage_route` → `merge/coverage.rs::translation_coverage` | `TranslationCoverageView { page_count, translated_pages, contributing_jobs, segments[], jobs[] }` | 无 | 只有接口目录一行 | `frontend/web/src/features/book-detail/ui/use-translation-coverage.ts:7`（任务状态一变就重取） | 见问题 3、8、9、15 |
| GET `/api/v1/documents/:document_id/usage` | `routes/token_usage.rs::document_usage_route` → `retainpdf-jobs/src/reports/usage/api.rs::document_usage_view` | `UsageSummaryView` | `token-usage.v1`，`api_tests/token_usage.rs:159` 双向钉键；前端类型从契约生成 | 只有接口目录一行 | `frontend/web/src/features/usage/ui/BookUsageCard.tsx:13` | 登记在 `router/backups.rs`。见问题 7、16 |
| GET `/api/v1/documents/:document_id/jobs` | `library_data::list_document_jobs_route` → `retainpdf-jobs/src/presentation/listing.rs:47` | `DocumentJobListView { items, invocation_summary, total, limit, offset, has_more }` | 无（A 组 5 已报） | `docs/api/00-约定/03-分页.md`、`api-spec/jobs-document-history.md:84` | `frontend/web/src/features/library/domain/documents/job-actions.ts:81`（`frontend/packages/api/src/documents.ts:402`） | 分页已合规。见问题 14 |
| POST `/api/v1/assets`（closed） | `routes/library_extras.rs::upload_asset_route` → `retainpdf-library/src/assets.rs::store_asset` | `AssetRecord` | 无 | `docs/core/frontend/library-api.md` 第 5 节（按已删的收藏流程写的）、`api-spec/storage-and-errors.md:125` | 无（收藏 2026-09 删了以后没人用） | 见问题 11、12 |
| GET `/api/v1/assets/:asset_id`（closed） | `library_extras::download_asset_route` → `assets.rs::load_asset` | 文件流 | 无 | 同上 | 无 | 见问题 12 |
| GET `/api/v1/fonts` | `routes/fonts.rs::list_fonts` → `retainpdf-platform/src/fonts/service.rs::list_fonts` | `Vec<FontInfo>`（`data` 是裸数组） | 无 | `docs/core/api/index.md:743` | `frontend/web/src/features/settings/ui/ThemeAppearancePanel.tsx:46`（`frontend/packages/api/src/fonts.ts:12`） | 见问题 13 |
| POST `/api/v1/fonts/upload`（admin） | `fonts::upload_font` → `service.rs::save_uploaded_font` | `FontInfo` | 无 | `docs/core/api/index.md:744`、`storage-and-errors.md:126` | 只有封装 `frontend/packages/api/src/fonts.ts:22`，界面里没有调用 | 见问题 11、13 |

## 问题

### 1. 书的原文会变（Agent 文档操作提交以后），封面和缩略图却永远不会重新生成，原文页预览图被浏览器按 `immutable` 缓存一年
- 位置: `backend/packages/retainpdf-ai/src/document_operations/facade/source_projection.rs:84-106`（提交后存一条 `version-upload-<版本>` 上传，`content_hash` 写成原 `document_id`，再 `upsert_document_from_upload`）；`database/retainpdf-db/src/db/documents/crud.rs:418-433`（`find_upload_for_document` 优先取当前版本的那条）；`backend/packages/retainpdf-files/src/derived_artifacts/preview.rs:58-69`（`ensure_book_image_at_path`：文件在就直接用）；`backend/packages/retainpdf-library/src/media.rs:24-27,142,163`（`IMAGE_CACHE`、`PAGE_CACHE = "private, max-age=31536000, immutable"`，注释写「书的编号就是源 PDF 的指纹…同一个地址内容不变」）；`docs/api/07-产物下载/01-下载总览.md:87-88`。
- 现状: Agent 文档操作在单机模式下能用（`closed` 只在多用户下生效）。提交以后，`/documents/:id/files/source.pdf` 返回的是新版本，前端读原文时自己加 `?version=<revision>`（`frontend/web/src/features/reader/domain/host/data.ts:220`）。封面、缩略图：`data/documents/<id>/cover.jpg` 已经存在就直接返回，不看源 PDF 改没改，服务端永远是旧图。原文页预览图：服务端按修改时间会重新渲染（`cached_output_is_fresh`），但同一个地址带了 `immutable`、一年有效，浏览器不会来问，看到的还是旧图（`answer-enhance.ts:227` 不带版本参数）。
- 问题: 「书编号 = 原文指纹、地址内容不变」这个前提，在有版本提交以后就不成立了，代码和文档都还按这个前提写。（未实跑）
- 建议: 二选一。(a) 把版本带进地址：`/files/pages/3.jpg?version=<active_version_id>`，或者在 `DocumentFilesView` 和书卡的 `cover_url` 里带上版本参数，`immutable` 照旧；(b) 原文页改成和译文页一样的 `private, max-age=0, must-revalidate` 加 ETag。封面、缩略图的生成改成按源 PDF 的修改时间判断新不新（像页预览那样用 `cached_output_is_fresh`），`generation_key` 里带上源文件的时间戳。文档 `下载总览.md:87` 改掉「同一个地址内容不变」。对前端的影响: 方案 (a) 要前端改用后端给的地址（`cover_url` 已经是后端给的；页预览 `answer-enhance.ts:227` 是手拼的，要换成读 `url_template` 或带上版本）；方案 (b) 前端不用改。
- 等级: 中

### 2. 合并阅读的虚拟任务 id 失效得比注释说的早，失效后是不带 reason 的普通 404
- 位置: `backend/packages/retainpdf-library/src/merge/build.rs:278-310`（`STALE_MERGED_AFTER` 24 小时，`prune_stale` 按目录修改时间算年龄）；`build.rs:270-276`（新目录发布后马上 `prune_stale`）；`backend/packages/retainpdf-api-base/src/job_loading/merged.rs:61-71`（读不到 `reading.json` → `job not found: <id>`）。
- 现状: 注释说「不在新目录一发布就删：可能有人正拿着旧的虚拟 id 开着阅读器」。但年龄算的是合并目录的修改时间，也就是它**生成**的时间，不是最后一次被读的时间。一份两天前生成、今天才打开的合并结果，只要这时有新的范围翻译完成、有人（另一个窗口、AI 服务的 `rust_client.py:79`）再调一次 `/reading` 生成了新目录，旧目录马上被删，正在读的阅读器接下来的页图、Markdown、译文条目请求全部 404 `job not found: merged-…`。
- 问题: 「保留一天」实际上保护的只是「生成后一天内」。失效后的 404 和「任务被删了」完全一样，前端没法知道应该重新问一次 `/reading`。（未实跑）
- 建议: 年龄改按最后使用时间算：`load_virtual_job` 读 `reading.json` 时顺手 touch 一下（或单独记一个 `last_read` 文件），`prune_stale` 看它。虚拟 id 格式合法、目录却不在时，404 带 `error.details.reason = "merged_reading_expired"`，前端按它重新调 `/reading` 换 id。`docs/api/00-约定/02-认证与错误.md` 的 404 一节补这个 reason。对前端的影响: 只增 reason，旧行为不变；前端可以选择按 reason 自动换新 id。
- 等级: 中

### 3. `/reading` 和 `/translation-coverage` 每次请求都把源 PDF 和每个成功任务的输出 PDF 整份载入内存数页数，没有缓存
- 位置: `merge/reading.rs:97-106`（`lopdf::Document::load(source_pdf)` 只为了 `get_pages().len()`）；`merge/sources.rs:93-96`（每个任务的输出 PDF 同样整份 `lopdf::Document::load`）；`merge/coverage.rs:230-232`（经 `document_merge_sources` 走同一段）；`frontend/web/src/features/book-detail/ui/use-translation-coverage.ts`（任务状态一变就重取，任务列表本身在轮询）。
- 现状: 一本书有几个成功的翻译任务，每次打开书、每次书籍详情页里任何任务状态变化，后端就把这几份完整的 PDF 解析一遍。`lopdf` 的 `load` 会解出整个对象表和所有流，几百页、带图的 PDF 是几十到上百 MB 内存、几百毫秒到几秒。
- 问题: 打开书和书籍详情的「进度」分区跟着书变大而变慢，详情页开着时一直有这个开销。（未实跑，没有量过）
- 建议: 页数只需要 `/Count`，可以用 `lopdf::Document::load_metadata`（只读 trailer 和页树）或者复用项目里已有的页数工具；或者按 `(路径, 大小, 修改时间)` 缓存页数（合并指纹里已经算了同样的 `file_stamp`）。源 PDF 的页数直接用 `documents.page_count`（见问题 8，先统一来源）。对前端的影响: 无。
- 等级: 中

### 4. 书库包读文档时把数据库错误一律说成 404
- 位置: `backend/packages/retainpdf-library/src/media.rs:89-91`（`document_source`）、`:227-229`（`document_files_view`）；`backend/packages/retainpdf-library/src/documents.rs:47-49,113-122,230-232`（`require_document_upload`、`get_document` 等，都是 `.map_err(|_| AppError::not_found(...))`）。对照：`database/retainpdf-db/src/db/documents/crud.rs:266-273` 的 `get_document` 已经特意带上 `QueryReturnedNoRows`，注释写「调用方凭它判 404，其余错误是 500」；任务包的 `lifecycle/queries.rs:18-20`（`GET /documents/:id/jobs`）用的就是 `AppError::from_lookup`。
- 现状: `/documents/:id/files`、`/files/:name`、`/files/pages/:page`、`/reading`、`/translation-coverage`（还有本组之外的 `GET /documents/:id`、建任务入口）在库被锁、行解码失败时都回 404 `document not found`。同一本书、同一次故障，`/documents/:id/jobs` 回 500、其余回 404。`document_files_view` 还会把 `document_source` 的任何错误（包括 `find_upload_for_document` 的库错误）吞成 `ready: false`。
- 问题: 违反规矩「数据库读错不能说成 404」（`docs/api/00-约定/02-认证与错误.md`），排障时会去查错地方。
- 建议: 这几处统一改成 `AppError::from_lookup(error, format!("document not found: {document_id}"))`；`document_files_view` 只在「书在、原文不在磁盘」时给 `ready: false`，库错误照常返回 500。`api_tests/job_loading_errors.rs` 旁边补一条书的用例。对前端的影响: 只在坏数据、库锁住时 404 变 500；前端对非 OK 都走「读取失败」，无功能影响。
- 等级: 中

### 5. 阅读入口、翻译覆盖、书的文件清单都没有契约，前端类型手写
- 位置: `contracts/` 下没有 `DocumentReadingView`、`TranslationCoverageView`、`DocumentFilesView`；`frontend/packages/api/src/documents.ts:206-211`（`DocumentReadingView` 手写）、`:224-252`（`TranslationCoverageView` 手写，字段上标着「旧后端没有这个字段」的可选项）；Rust 侧只有 `api_tests/merged_reading.rs` 断言个别字段，`file-responses.txt:269` 钉了 `/files` 清单的形状。
- 现状: `token-usage.v1` 是好例子：契约、Rust 双向钉键（`api_tests/token_usage.rs:159`）、前端从契约生成类型（`frontend/packages/api/src/usage.ts:3`）。阅读入口和覆盖是书籍详情、阅读器的主要数据来源，却没有同等的保护；`TranslationCoverageJob` 前端标成可选的五个字段，后端其实一直返回。
- 问题: 后端改字段不会有红灯，前端类型和实际回包会慢慢走样。
- 建议: 新增 `library-reading.v1`（或并进 `library-books.v1`），收 `DocumentReadingView`、`TranslationCoverageView`（含 `CoverageSegment`、`CoverageJobView`）、`DocumentFilesView`，在 `api_tests` 里照 `token_usage.rs:159` 的写法用 `assert_exact_object_keys` 双向钉。对前端的影响: 无（只加契约）；前端以后可以换成生成的类型，去掉那几个可选标记。
- 等级: 中

### 6. 文档：阅读、覆盖、书的用量没有说明；书库对接文档和总文档里有已删的东西、过时的打开方式
- 位置:
  - `docs/api/` 下只有接口目录里各一行；`/reading`（含虚拟 id 的格式、什么时候合并、写接口拒绝虚拟 id）、`/translation-coverage`（字段含义、和阅读入口同一套规则）、`/documents/:id/usage` 的回包都没有说明。`docs/api/07-产物下载/01-下载总览.md:150-158` 只写了书的文件。
  - `docs/core/frontend/library-api.md:1-3`「更新于 2026-09-02」；`:33`「`active_job_id` 是该文档当前生效的处理 run——**打开阅读器就用它**」，现在该问 `/reading`（前端 `resolve-reading-target.ts` 的注释也说 `active_job_id` 可能只覆盖几页）；`:221` 字段速查同样写「阅读器入口」；第 5 节「资产」整段按已删除的收藏流程写（「POST favorites 时带 `asset_id`」「favorites 记录现在返回 `asset_id`」）。
  - `docs/core/api/index.md:733-744`「图书馆和 Reader 的附加资源入口」把已删除的 `GET|POST /api/v1/favorites`、`PATCH|DELETE /api/v1/favorites/{favorite_id}` 列成在用的接口，也没有列 `/reading`、`/translation-coverage`、`/documents/:id/files`。
  - `docs/api/07-产物下载/01-下载总览.md:87-88`「书的编号是原文的指纹，同一个地址内容不变」，有版本提交以后不成立（见问题 1）。
  - `docs/api/00-约定/03-分页.md` 第 5 条写了译文列表、事件、`GET /jobs` 对未知查询参数怎么办，没写 `GET /documents/:id/jobs`（现在是静默忽略）。
- 问题: 照着文档对接的人会用 `active_job_id` 打开范围翻译过的书（错页），会去调已经不存在的收藏接口。
- 建议: 在 `docs/api/` 下新增「书」一节（或并进 `07-产物下载`），写 `/reading`、`/translation-coverage`、`/documents/:id/usage` 的回包和错误；`library-api.md` 第 1 节、字段速查改成「打开阅读器先问 `/reading`」，第 5 节标「收藏已删，这两个接口现在没有调用方」；`docs/core/api/index.md:733-744` 删掉 favorites 两行、补上书的这几个接口；`03-分页.md` 第 5 条补一句。对前端的影响: 无。
- 等级: 中

### 7. 书不存在时 `/documents/:id/usage` 回 200 空汇总，和规矩、和任务的用量接口都不一致
- 位置: `backend/packages/retainpdf-jobs/src/reports/usage/api.rs:46-58`（不查书在不在，直接 `list_jobs_for_document`）；`backend/api/src/api_tests/token_usage.rs:55-56`（测试钉住了 `/documents/doc-without-jobs/usage` 回 200、`requests = 0`，那本书在库里并不存在）；对照 `api.rs:36-44`（`job_usage_view` 任务不存在 404）。
- 现状: 编号写错、书已经删掉，都回 200 和一份全零的汇总；书删掉以后，助手台账里带这个 `document_id` 的行照样算进来。
- 问题: 违反「任务（或书）本身不存在：普通 404」（`02-认证与错误.md`「还没产出」一节）；同一组用量接口一个 404 一个 200。
- 建议: 先 `db.get_document(document_id)` + `from_lookup`，不存在 404；有书没任务照旧 200 空汇总。测试里的 `doc-without-jobs` 改成先建书。对前端的影响: `BookUsageCard.tsx` 只对已存在的书调用，实际不受影响；书刚被删时卡片从「0 用量」变成「读取失败」。
- 等级: 低

### 8. 翻译覆盖和阅读入口用的页数来源不一样
- 位置: `backend/packages/retainpdf-library/src/reading.rs:62-65`（覆盖用 `documents.page_count`）；`merge/reading.rs:102-105`（阅读入口用 `lopdf` 读源 PDF 的实际页数）；`merge/coverage.rs:6-7` 注释说「覆盖」和阅读入口用同一套规则；`media.rs:147` 的注释说 `page_count` 为 0 表示没记录。
- 现状: 两边算计划的规则一样，计划的长度却来自两处。`page_count` 为 0（没记下页数的老数据）时，覆盖的 `page_count`、`translated_pages`、`segments` 全是 0 或空，`jobs[].pages` 里整本的 OCR 任务也成了空列表（`parse_spec("", 0)`），阅读入口却照常合并整本。两个数对不上时（上传时数错、版本提交后只更新了一处）也会出现「覆盖说第 N 页来自某次翻译，阅读器里不是」。
- 问题: 两个接口说的话可能对不上，和注释的承诺相反。（未实跑）
- 建议: 选一个来源。最省事的是两边都用 `documents.page_count`，为 0 时退回读源 PDF（顺便修问题 3）；或者把阅读入口算出来的页数回写到 `documents`（`crud.rs:652-662` 已经有 `?1 > 0` 才写的更新）。对前端的影响: 无。
- 等级: 低

### 9. 书库包有三处绕开了产物路径的出口，架构检查的正则没覆盖到
- 位置: `merge/coverage.rs:140-142,173-174`（手拼 `data_root/jobs/<任务>/artifacts/translation_diagnostics.json`，没用已有的 `retainpdf_core::storage_paths::resolve_translation_diagnostics`，`StoredFile` 里也没有这一项）；`merge/sources.rs:8,127`（`resolve_ocr_markdown_images_dir`）；`reading.rs:110`（`resolve_job_root`）；`backend/api/scripts/check_architecture.py:1698-1702`（`ARTIFACT_PATH_RESOLVER` 只认 `resolve_(output_pdf|source_pdf|markdown_images_dir|…)`，`resolve_ocr_markdown_images_dir`、`resolve_job_root`、`resolve_translation_diagnostics` 和手拼字符串都漏掉了）。
- 现状: CLAUDE.md 要求「别处要读已经落盘的产物，调任务包的 `stored_path(任务, data_root, StoredFile::…)`」，`KNOWN_ARTIFACT_PATH_READERS` 已经清空。这三处能过检查，是因为正则没写全。`coverage.rs` 手拼的路径假设任务目录一定是 `data_root/jobs/<id>`，今天成立（`config/paths.rs:79`），流水线换布局就会悄悄读不到，`kept_origin_blocks` 全变成 0。
- 建议: `StoredFile` 加 `TranslationDiagnostics`、`OcrSourceImagesDir`（或 `JobRoot`），这三处改调 `stored_path`；`ARTIFACT_PATH_RESOLVER` 补上 `ocr_markdown_images_dir|job_root|translation_diagnostics`（补之前先看还有哪些文件会被扫出来，按规矩不许往冻结名单里加）。对前端的影响: 无。
- 等级: 低

### 10. `GET /reading` 里生成合并目录不排队、不去重
- 位置: `reading.rs:41-55`（`spawn_blocking` 直接调 `resolve_reading_target`）；`merge/build.rs:208-277`（两步 Python 子命令，各 120 秒时限）；对照 `media.rs:167-190`（书的文件走 `download_generation`：同一份产物同时请求只生成一次，全局只有 2 个名额）。
- 现状: 同一本书同时打开两次（前端一次、AI 服务一次），会在各自的临时目录里各生成一遍，后发布的那个丢掉自己的结果（`build.rs:271-274`），结果是对的，只是白干一遍；几本书同时打开时子进程数量没有上限。
- 建议: 把生成放进 `DownloadGeneration`，键用 `document:<id>:merged:<指纹>`（指纹在生成前就能算出来）。对前端的影响: 无。
- 等级: 低

### 11. 没人用的接口：书的文件清单、字体上传、资产
- 位置: `router/documents.rs:21`（`GET /documents/:id/files`）；`router/fonts.rs:11`（`POST /fonts/upload`）；`router/ai.rs:16,18`（`/assets`）。调用方是对 `frontend/`、`backend/ai`、`backend/cli`、`scripts`、`ops` 做 `git grep` 得出的。
- 现状:
  - `GET /documents/:id/files`：没有调用方。前端在 6 处手拼 `/files/source.pdf`，在 `answer-enhance.ts:227` 手拼 `/files/pages/{n}.jpg`，没有读清单里的 `url`、`url_template`。只有 `file-responses.txt` 快照和文档在用。
  - `POST /fonts/upload`：`frontend/packages/api/src/fonts.ts:22,50` 有封装，`frontend/web` 里没有任何界面调用；设置页只调 `listFonts`。
  - `/assets`：给收藏截图用的，收藏 2026-09 删掉以后没有调用方；多用户下 `closed`。
- 建议: `/files` 清单保留（是书的文件的正式目录，问题 1 的方案 (a) 正好要前端改读它）。`/fonts/upload` 保留（管理员功能，可能以后接界面），在 `docs/core/api/index.md` 标「目前没有界面入口」。`/assets` 请用户定：要么跟收藏一起删（连 `assets` 表、`data/assets/`、`AssetConfig`），要么在接口目录里写明「保留给 AI 附件，现在没有调用方」。对前端的影响: 删 `/assets` 对前端无影响（没人调）。
- 等级: 低

### 12. 资产：MIME 信客户端说的、缓存头是 `public`，同一份字节换个 MIME 再传会留孤儿文件
- 位置: `backend/api/src/routes/library_extras.rs:46-49`（`field.content_type()` 原样当 MIME）、`:69-74`（`Cache-Control: public, max-age=31536000, immutable`）；`backend/packages/retainpdf-library/src/assets.rs:25-30,77-97`（文件名后缀跟着 MIME 走，`save_asset` 之后才返回）；`database/retainpdf-db/src/db/assets.rs:9-24`（`ON CONFLICT(asset_id) DO NOTHING`）。
- 现状: 同一份字节第一次按 `image/png` 传、第二次按 `image/jpeg` 传，第二次会再写一个 `<id>.jpg`，库里保留第一次的 `png`。第二次的响应却说 `mime: image/jpeg`，`created_at` 是这次的时间，和库里不一样；`.jpg` 那份没人会读，成了孤儿。缓存头是 `public`，而书的文件都是 `private`（登录后才能看的内容不进共享缓存）。
- 建议: 存之前按文件头判断真实类型，不信客户端的 `Content-Type`；已存在时返回库里那条记录；缓存头改成 `private, max-age=31536000, immutable`。如果问题 11 决定删掉 `/assets`，这一条就不用做。对前端的影响: 无（没有调用方）。
- 等级: 低

### 13. 字体：列表返回服务器绝对路径、`data` 是裸数组，扫描和上传在 async 线程上跑子进程，上传同名文件直接覆盖
- 位置: `backend/packages/retainpdf-platform/src/fonts/service.rs:14-19,109-113`（`files` 是绝对路径，上传的响应也是）；`backend/api/src/routes/fonts.rs:14-19`（`ApiResponse<Vec<FontInfo>>`）；`service.rs:44-58,84-91,252-293`（`fc-list`、每个文件一次 `fc-query`/`fc-scan`、`fc-cache -f`）；`routes/fonts.rs:21-30`（没有 `spawn_blocking`）；`service.rs:77-84`（`std::fs::write` 直接覆盖同名文件，不检查内容是不是字体）。
- 现状: 别的列表都是 `{ items: [...] }` 一类的对象，这里 `data` 是数组，前端 `fonts.ts:16-18` 两种都接。缓存没命中时（10 分钟一次，或目录变了）要对每个字体文件起一次 `fc-query`，都在 tokio 工作线程上同步跑；上传还要等 `fc-cache -f`。`files` 把 `/usr/local/share/fonts/...`、`data/fonts/...` 这类服务器路径直接给客户端，前端只用 `family`。
- 建议: 处理函数里用 `spawn_blocking` 包住 `list_fonts`、`upload_font`（`token_usage.rs` 的 `blocking` 是现成写法）。只增不改地加一个 `{ fonts: [...] }` 的形状要改返回类型，前端已经兼容 `data.fonts`，可以等下一次动这块再做；`files` 改成只给文件名（前端没读它，`ThemeAppearancePanel.tsx` 只用 `family` 和 `available`，改之前再确认一遍）。上传遇到同名文件 409 或自动改名，内容用 `fc-query` 的结果判断是不是字体，不是就 400。对前端的影响: `files` 改成文件名，前端不受影响；改返回形状前端已经兼容。
- 等级: 低

### 14. `GET /documents/:id/jobs` 分页已合规，剩下三个小缝
- 位置: `backend/packages/retainpdf-jobs/src/presentation/listing.rs:47-68`；`views/common.rs:108-116`（`ListDocumentJobsQuery` 没有 `deny_unknown_fields`）；`database/retainpdf-db/src/db/jobs.rs:247-276`（坏行跳过）；`frontend/packages/api/src/documents.ts:402-421`、`frontend/web/src/features/library/domain/documents/job-actions.ts:81`。
- 现状: 现在合规：`limit` 越界 400（`support.rs:30 checked_limit`），回显 `limit`、`offset`、`has_more`、`total`，书不存在走 `from_lookup` 是 404、库错误 500，坏行跳过不再让整个列表 500（A 组 7 已修）。剩下的：(a) 未知查询参数静默忽略，`03-分页.md` 没写这个接口；(b) `total` 是 `COUNT(*)`，列表跳过了坏行，有坏行时 `offset + 返回条数 < total` 一直成立，最后一页的 `has_more` 会是 `true`，客户端多翻一页拿到空页（不会死循环）；(c) 前端 `fetchDocumentJobs` 固定 `limit=50, offset=0`，不看 `has_more`，一本书超过 50 个任务时界面只显示前 50 个。
- 建议: (a) 在 `03-分页.md` 写明，前端只发 `limit`、`offset`，可以直接加 `deny_unknown_fields`；(b) 改成多取一条判断 `has_more`（和 `GET /jobs` 一样），`total` 照旧；(c) 记进前端整理记录：书籍详情的任务列表按 `has_more` 翻页。对前端的影响: (a) 无（前端没有多余参数）；(c) 是前端自己的事。
- 等级: 低

### 15. 翻译覆盖里的 `jobs[]` 是第三种任务摘要，排序、空值和任务列表都不一样，最多只看 500 个任务
- 位置: `merge/coverage.rs:31-62`（`CoverageJobView`）；`:230-231`（`list_jobs_for_document(…, 500, 0)` 后按 `created_at` 倒序）；`merge/reading.rs:97`（阅读入口也只取 500 个，按 `updated_at` 取前 500）；对照 `JobListItemView`（`GET /documents/:id/jobs`，按 `updated_at` 倒序，失败信息在结构化的 `failure` 里）。
- 现状: 同一本书的任务，在任务列表里是 `JobListItemView`（`failure` 对象），在覆盖里是 `CoverageJobView`（`failure_summary` + `error_head` 两个截到 160 字的字符串）；纯 OCR 任务的 `model` 是空串，同一个结构里的 `finished_at`、`note` 没有时是 `null`。两边一个按提交时间排、一个按更新时间排。任务超过 500 个时，更新时间最早的那些既不参与合并，也不出现在覆盖里，没有任何提示。
- 建议: `model` 没有时给 `null`（前端类型 `model: string` 要一起改成可空，或者保留空串并在契约里写明）；在契约（问题 5）里写清楚 `jobs[]` 的排序。500 的上限改成查全部（这本书的任务数就是 `count_jobs_for_document`），或者超过时在回包里给 `truncated: true`。对前端的影响: `model` 改 `null` 要前端同步；其余只增。
- 等级: 低

### 16. 登记位置：书的用量登记在备份的路由文件里，资产登记在 AI 里
- 位置: `backend/api/src/app/router/backups.rs:11`（`GET /documents/:document_id/usage`）；`docs/api/00-接口目录.md:231-232`（因此排在备份组）；`router/ai.rs:16,18`（`/assets`）。
- 现状: 文件头注释写了理由（用量和备份都是「设置」页的数据），但别的 `/documents/:id/...` 都在 `router/documents.rs`，查接口目录的人在「书」那一组找不到用量。
- 建议: 挪到 `router/documents.rs`（登记表换个文件，地址不变），重新生成接口目录。或者把 `/usage`、`/documents/:id/usage` 一起单独开一个 `router/usage.rs`。对前端的影响: 无。
- 等级: 低

## 已知问题的现状

- 删书不清 `data/documents/<id>/`（正确性审计 2-书库 第 5 条）：**没变**。`backend/packages/retainpdf-library/src/documents.rs:139-200` 仍然只删任务目录和上传目录；封面、缩略图、原文页预览、合并阅读缓存 `merged/<指纹>/` 都留着，书删掉以后 `prune_stale` 也不会再被触发。
- 有收藏记录的老库删书 500（第 6 条）：没有细看，`crud.rs:521-534` 的 `delete_document` 注释还写着「FK 级联清 favorites」，看不出有改动。
- 合集 agent 工作区链到在跑的半成品（第 7 条）：**修了一部分**。`reading.rs:91-115` 现在对「一个任务覆盖整本」（`ReadingTarget::Job`）也按阅读入口链，不再看书卡；但没有任何可读译文时（`ReadingTarget::None`：只有 OCR、或者只有在跑或失败的任务）仍然退回 `collection_workspace.rs:193` 按 `active_job_id` 链，书卡指着在跑的任务时还是链到半成品目录。
- 产出者被删以后阅读入口 500（第 2 条，不在说明的已知清单里，但已经报过）：**缓解了**。删任务时有依赖者会 409 `job_has_dependents`（`crud.rs:541 jobs_depending_on`）；但 `force=true` 仍然放行，`merge/sources.rs:121-124` 仍然不检查 `translations_dir/translation-manifest.json` 在不在，强删以后这本书的 `/reading` 照样 500。

## 查过、没问题的

- 地址和登记：本组 11 个接口在登记表、`docs/api/00-接口目录.md`、`routes.txt` 里一致，路径参数统一叫 `:document_id`，没有旧别名；`/files/pages/:page` 和 `/files/:name` 段数不同，不会冲突。
- 书的文件只有一个出口 `media.rs::open_document_file`，读文件、按需生成都经过 `DownloadGeneration`（同一份产物并发请求只生成一次）；页码越界在起渲染进程之前就 404，`page_count` 为 0 时交给渲染脚本用退出码 3 判越界，也是 404；页预览和封面渲染都有 60 秒时限，先写临时文件再改名。
- 报错和约定一致：书不存在 404 `document not found`；未知文件名、`0.jpg`、`3.png` 都是普通 404 `no such file`；原文不在磁盘是不带 `not_ready` 的普通 404，和 `02-认证与错误.md`「暂时不在这条规矩里的」一节写的一样。
- `?download=1` 和 `inline` 的切换、下载文件名（书名 → 原文件名 → 编号）和任务下载同一套规则；`file-responses.txt` 钉了清单和三个文件的响应头。
- 虚拟任务 id 只在读路径上认（`load_supported_job`）；写操作走 `load_job_or_404`，自然拒绝；`/jobs/:id/data` 现在也认虚拟 id（任务审计 C 组第 6 条已修）；`/jobs/:id/reports/usage` 对虚拟 id 给空汇总。虚拟任务的产物从 `JobArtifacts::default()` 开始，只填合并目录里真有的东西，不会挂上某个参与任务的诊断、日志。
- 合并目录：按内容指纹命名，指纹包含合并器版本、源 PDF 的大小和修改时间、每页来源、每个参与任务的输出 PDF、译文清单、OCR 文档；在临时目录里生成完再整体改名，并发生成同一指纹时后到的丢掉自己的；`reading.json` 先写临时文件再改名，崩溃后下一次请求会补写。版本提交换了源 PDF 也会换指纹。
- 合并计划：排序键 `(产出者提交时间, 完成时间, 提交时间, job_id)` 有充分的测试（换字体不翻盘、原地精修胜出、同一秒按 `job_id` 定序、越界页跳过）；只认成功的任务和排队中的原地重跑，页数对不上就不拼。
- `DocumentReadingView` 的空值：没有可读译文时 `job_id: null`、`merged: false`、`contributing_job_ids: []`，前端 `resolve-reading-target.ts` 按 `null` 退回调用方给的任务；接口失败也退回，打开阅读器不会因为这一步坏掉。
- 用量：`token-usage.v1` 契约由 `api_tests/token_usage.rs:159` 双向钉键，前端类型从契约生成；读台账放在阻塞线程里。
- `GET /documents/:id/jobs`：见问题 14 开头，分页、404/500 都已合规。
- 资产、字体上传都按流式读并限额，不信 `Content-Length`（`library_data.rs` 测试 `asset_upload_enforces_configured_stream_limit_without_trusting_content_length`、`fonts.rs` 的两个测试）；字体文件名去掉了目录部分，`..` 会退回 `upload.otf`；字体列表的缓存按目录修改时间加 10 分钟过期失效，上传后清掉。
- 页预览的 `?width=`（240～2400）、`?dpi=`（≤300）是静默夹取，不是 400；范围在 `下载总览.md:84` 里写了，和任务的页预览一致，不算违反分页那条规矩（那条只管列表的 `limit`）。
