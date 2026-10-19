# 书、书库列表、搜索、合集

审计范围：`/documents`（列表、一本书的读改删）、`/library/books`（书架列表、详情、删除）、`GET /search`、`/collections` 全部 7 个接口（含 `closed` 的 `agent-workspace`）。
只读代码（main，`f4574fc0`）和现有快照得出，没有跑测试、没有调用运行中的服务。「今日调用」取自本机 `data/diagnostics/route-usage.json`（2026-10-18 08:06 起到 2026-10-19 08:41，单机开发栈）。标「未实跑」的是读代码推断的结论。

## 接口一览

| 方法 地址 | 处理函数 | 返回类型 | 契约 | 文档 | 调用方（前端文件:行） | 今日调用 | 备注 |
|---|---|---|---|---|---|---|---|
| GET `/api/v1/documents` | `routes/library_data.rs::list_documents_route` → `retainpdf-library` `documents.rs::list_documents` | `DocumentListView { documents, total }` | 无 | `docs/core/frontend/library-api.md` §1（陈旧）、`docs/core/api/index.md:164-180`（陈旧） | `frontend/packages/api/src/documents.ts:154 fetchDocumentList`（书库主网格 `document-library-source.ts`、合集 `collections/domain/controller.ts:43,51,65`、问答选书 `ask/domain/document-picker.ts:31,94`、搜索岛 `library/ui/island/index.ts:97`）；`documents.ts:170 fetchDocumentByJobId`（阅读器 `reader-session/session-assets.ts:322`）；AI `rust_client.py:59 list_documents`、`:81 get_document_by_job` | 1520 | `?job_id=` 是另一种语义（见第 10 条） |
| GET `/api/v1/documents/:document_id` | `library_data::get_document_route` → `documents.rs::get_document` | `DocumentRecord` | 无 | 同上 | `documents.ts:182 fetchDocument`（`book-detail/ui/useDocumentMeta.ts:62`）；AI `rust_client.py:74`（`tools.py:272,535`） | 6 | |
| PATCH `/api/v1/documents/:document_id` | `library_data::patch_document_route` → `documents.rs::patch_document` | `DocumentRecord` | 无 | 同上（写着 `tags?`） | `documents.ts:190 patchDocument`（`library/domain/documents/update-document.ts:31`、`library/ui/island/library-search-app.tsx:127`） | 0 | 只认 `title`、`reading_status` |
| DELETE `/api/v1/documents/:document_id` | `library_data::delete_document_route` → `documents.rs::delete_document` | `DocumentDeleteResultView` | 无 | `library-api.md`「两个必须处理的边界」、`docs/api/00-约定/02-认证与错误.md:90-98` | `documents.ts:339 deleteDocument`（`library/domain/documents/delete-actions.ts:29,52`） | 0 | `?force=true` 先取消再删 |
| GET `/api/v1/library/books` | `routes/library.rs::list_books` → `book_projection.rs::build_library_book_list_view` | `LibraryBookListView { items }` | `library-books.v1`，`job_view_contract.rs:291` 钉键集合 | `docs/core/api/index.md:50-118`（说「优先使用」，与现状相反） | `frontend/packages/api/src/library-books.ts:17 fetchLibraryBookList`：文档卡片叠加任务活态 `library/domain/documents/shape-documents-with-books.ts:60`；最近任务分页 `recent-jobs/library-books-resource.ts`、`recent-jobs/loader.ts:85` | 12 | 实质是「非 OCR 任务列表」 |
| GET `/api/v1/library/books/:job_id` | `library::get_book` → `books.rs::get_library_book` | `LibraryBookDetailView` | `library-books.v1`，同上钉住 | `docs/core/api/index.md:119-139` | **没有调用方**（前端、`backend/ai`、`backend/cli`、脚本都没有；只有 `api_tests`） | 0 | |
| DELETE `/api/v1/library/books/:job_id` | `library::delete_book` → `books.rs::delete_library_book` | `LibraryDeleteResultView` | `library-books.v1`（没有测试钉） | `docs/core/api/index.md:141-158`、`02-认证与错误.md:90-98` | `library-books.ts:35 deleteLibraryBook`，只在卡片没有 `document_id` 时走（`delete-actions.ts:76` → `recent-jobs/actions.ts:75`） | 0 | 实际是「删一个任务」，见第 1 条 |
| GET `/api/v1/search` | `library_data::search_blocks_route` → `search.rs::search_blocks` | `SearchResultView { query, hits }` | 无 | `library-api.md` §3 | `frontend/packages/api/src/search.ts:6 searchLibrary`（`library/ui/island/library-search-app.tsx:99`）；AI `rust_client.py:45 search_fulltext`（`tools.py:451`） | 0 | |
| GET `/api/v1/collections` | `routes/collections.rs::list_collections_route` | `CollectionListView { collections }` | 无 | `library-api.md` §7 | `frontend/packages/api/src/collections.ts:6`（`collections/ui/use-collections-list.ts:36`、`book-detail/ui/useDocumentCollections.ts:87`、`ask/domain/document-picker.ts:39`）；阅读器终端 `reader/domain/terminal-scope.ts:60` 直接 fetch | 61 | 不分页 |
| POST `/api/v1/collections` | `collections::create_collection_route` | `CollectionRecord` | 无 | 同上 | `collections.ts:12`（`collections/ui/CollectionDialog.tsx:129`） | 0 | |
| PATCH `/api/v1/collections/:collection_id` | `collections::patch_collection_route` | `CollectionRecord` | 无 | 同上 | `collections.ts:25`（`CollectionDialog.tsx:134`，只发 `name`） | 0 | 登记表说能改「上级」，代码不能 |
| DELETE `/api/v1/collections/:collection_id` | `collections::delete_collection_route` | `CollectionMutationResult { deleted }` | 无 | 同上 | `collections.ts:40`（`CollectionDialog.tsx:161`） | 0 | |
| POST `/api/v1/collections/:collection_id/documents` | `collections::add_collection_documents_route` | `CollectionRecord` | 无 | 同上 | `collections.ts:51`（`collections/domain/controller.ts:37`） | 0 | |
| DELETE `/api/v1/collections/:collection_id/documents/:document_id` | `collections::remove_collection_document_route` | `CollectionMutationResult { removed }` | 无 | 同上 | `collections.ts:66`（`controller.ts:38`） | 0 | |
| GET `/api/v1/collections/:collection_id/agent-workspace`（closed） | `collections::collection_agent_workspace_route` → `reading.rs::collection_agent_workspace` | `CollectionWorkspace` | 无 | 只有 `docs/core/api/index.md:937` 一句 | `frontend/web/src/features/reader/domain/terminal-scope.ts:91`（阅读器终端切到「文件夹」范围前调一次） | 0 | GET 带副作用，见第 6 条 |

`docs/api/00-接口目录.md:160-196` 与登记表一致（同源生成），`routes.txt` 也一致；问题出在登记表自己的说明文字（第 2、11 条）。

## 问题

### 1. `/library/books` 和 `/documents` 是两套东西描述同一本书；书架这套其实是「非 OCR 任务列表」，该冻结并迁走
- 位置: `backend/api/src/app/router/library.rs:1`（注释已写「旧的一套，书的新接口在 documents」）；`backend/packages/retainpdf-library/src/views.rs:327-375`（`LibraryBookListItemView`、`LibraryBookDetailView`）；`book_projection.rs:162-210`（`list_books_filtered`，`exclude_ocr: true`）；`books.rs:36-98,100-107,120-128`；`backend/packages/retainpdf-jobs/src/query.rs:40-87`（`/jobs` 已支持 `job_ids`）；`retainpdf-jobs/src/views/job_types.rs:551-590`（`JobListItemView`）；前端 `shape-documents-with-books.ts:59-85`、`document-card-item.ts:92-131`、`delete-actions.ts:67-77`。
- 现状:
  - 「书」的身份是 `document_id`（内容哈希），一本书名下可以有多个任务。`/documents` 是书；`/library/books` 的每一项是一个任务：`id` 和 `job_id` 都是任务编号，列表项和详情里**都没有 `document_id`**，同一本书翻两次就是书架上两项。
  - 前端已经是「文档优先」：书库网格先拉 `/documents`，再把各书的 `active_job_id` 拼成 `job_ids` 调 `/library/books` 取任务活态（状态、进度、阶段）叠上去；没命中（OCR 任务被 `exclude_ocr` 挡掉）就逐个 `GET /jobs/:id` 补（`shape-documents-with-books.ts:70-85`）。卡片的封面、标题、页数都以文档为准（`document-card-item.ts:118-129`），书架返回的 `cover_url` 本来就是 `/documents/:id/files/cover.jpg`，和文档的同一个地址。
  - `GET /jobs?job_ids=` 现在已经生效（`query.rs:48-70`，测试 `jobs_list.rs:141`），`JobListItemView` 是书架列表项的超集（多 `workflow`、`stage_snapshot`、`invocation` 等，少 `title`/`progress`/`stage_detail`/`authors`，前端 `flattenStageSnapshot` 已经会摊平 `stage_snapshot`）；库里还有 `jobs_view_parity_tests.rs` 专门保证两套视图对得上。
  - `GET /library/books/:job_id` 没有任何调用方，今日 0 次。
  - `DELETE /library/books/:job_id` 删的是「这个任务 + 它的 `-ocr` 子任务」（`books.rs:100-107`），书（documents 行、上传、原文）留着；它也是全站**唯一能删单个任务的接口**（没有 `DELETE /jobs/:job_id`）。工作流是 OCR 的任务被 `load_library_job` 挡成 404（`books.rs:122-125`），所以单独的 OCR 任务除了整本删除外删不掉。前端只在卡片没有 `document_id` 时才走它。
- 问题: 两个前缀都叫「书」，名字和内容对不上；书架列表与 `/jobs?job_ids=` 功能重复，还各有一套 `q` 搜索规则（`/jobs` 会搜文档标题 `title_contains`，书架不会，`docs/api/01-任务/03-任务列表.md:37` 已记录）和 `limit` 规则（`/jobs` 越界 400，书架静默夹、带 `job_ids` 不限条数）。「删一个任务」挂在「书架」下面，OCR 任务没有删除入口。
- 建议:
  - 留 `/documents` 作为书的唯一一套；`/library/books` 冻结，不再加字段。
  - `GET /library/books/:job_id`：没有调用方，直接删（连同 `get_library_book*`、`build_library_book_detail_view` 和契约里的 `LibraryBookDetailView`，`job_view_contract.rs:292` 那一行一起改）。
  - `DELETE /library/books/:job_id`：在任务的 `lifecycle` 类下加新地址 `DELETE /api/v1/jobs/:job_id`（同一处理逻辑，顺便允许删 OCR 任务：OCR 任务被别的任务 `artifact_job_id` 引用时 `ensure_no_dependents` 已经会 409），旧地址行尾写 `old "/api/v1/jobs/:job_id"`。
  - `GET /library/books`：前端两处改调 `GET /jobs?job_ids=...&limit=<n>`（`n` ≤ 500，超了分批）之后删。`/jobs` 不排除 OCR，正好省掉前端「没命中再逐个取详情」那段。
  - 对前端的影响（写进 `docs/ops/planning/api-reorg/` 的整理记录）：`frontend/packages/api/src/library-books.ts:17` 旧写法 `library/books?job_ids=a,b&limit=n` → 新写法 `jobs?job_ids=a,b&limit=n&include_live_stage=true`，返回读 `items[].stage_snapshot`（已有 `flattenStageSnapshot`）、标题用 `display_name`；`recent-jobs/loader.ts:85` 的分页回退改读 `/jobs` 的 `has_more`；`library-books.ts:35` 旧写法 `DELETE library/books/:id` → 新写法 `DELETE jobs/:id`。后端只增不改，旧地址在前端改完前保留。
- 等级: 中

### 2. 文档标签 2026-09-26 已删，但前端、AI 工具、登记表、文档还在发 / 读 / 写 `tag(s)`，全被静默忽略
- 位置: 删除提交 `c3d44ff8`（`refactor(library): 删掉文档标签`）；`backend/packages/retainpdf-library/src/views.rs:37-54,65-70`（`ListDocumentsQuery`、`PatchDocumentInput` 都没有 `tag(s)`，也没有 `deny_unknown_fields`）；`backend/ai/retainpdf_ai/rust_client.py:59-72`（发 `tag`）、`backend/ai/retainpdf_ai/tools.py:530-565,672-687`（`list_documents` 工具把 `tag` 参数暴露给模型、结果里放 `"tags": document.get("tags")`）；`backend/api/src/app/router/documents.rs:12`（「改书的信息（书名、标签、阅读状态……）」）；前端 `frontend/packages/api/src/documents.ts:55-63,163`（发 `tag`）、`library/domain/documents/update-document.ts:22,42`、`library/ui/page/LibraryFilterMenu.tsx:140-172`（按 `item.tags` 过滤，恒为空）；`docs/core/frontend/library-api.md:20-34`；`docs/core/api/index.md:178`。
- 现状: 后端早就没有标签了（`DocumentRecord` 没有 `tags` 字段），但 `GET /documents?tag=x` 照样 200、返回全部文档；`PATCH {tags:[...]}` 照样 200、什么也不改。
- 问题: AI 的 `list_documents` 工具告诉模型「按标签过滤，可选」，模型传了标签拿到的是整个书库，还以为过滤过了（未实跑，依据是查询结构里没有这个字段且不拒绝未知参数）；每条结果都带 `"tags": null`。登记表和接口目录的说明还写着「标签」。
- 建议: 后端这边：AI 工具去掉 `tag` 参数和结果里的 `tags`，`rust_client.list_documents` 去掉 `tag`（`backend/ai` 属于后端，跑 AI 服务的 pytest）；登记表说明改成「改书的信息（书名、阅读状态）」后 `UPDATE_ROUTES=1` 重新生成接口目录；`library-api.md`、`index.md:178` 删掉标签。前端（写进整理记录）：`documents.ts` 的 `DocumentListQuery.tag` 和 `params.set("tag")`、`update-document.ts` 的 `tags` 分支、`LibraryFilterMenu` 的标签过滤去掉。两边都不发了以后，`ListDocumentsQuery`、`PatchDocumentInput` 再加 `deny_unknown_fields`（第 7 条）。
- 等级: 中

### 3. 数据库读错一律说成 404（书、合集）
- 位置: `backend/packages/retainpdf-library/src/documents.rs:47-50`（`require_document_upload`）、`:110-117`（`get_document`）、`:120-122`（`document_delete_job_ids`，DELETE 先走这里）、`:225-232`（`patch_document`：`update_document_fields` 的任何错误，包括库被锁、写失败）、`:82-84`（`?job_id=merged-…` 分支 `.ok()` 吞错）；`collections.rs:62-64`（`patch_collection`）、`:96-101`（`add_collection_documents`：`get_document(..).is_err()` → 404）。对照 `books.rs:117-121` 已经用了 `AppError::from_lookup`（`retainpdf-api-base/src/error.rs:285-294`，注释明说「不要再写 `map_err(|_| not_found(..))`」）。
- 现状: 同一个 `get_document` 失败，书架那边（`from_lookup`）分得清「没有」和「读不出来」，文档这边全是 `document not found`。
- 问题: 违反规矩第 5 条；库忙、行解码失败时用户看到「书不存在」，前端会把它从网格里拿掉或提示已删除，排障也找错方向。`patch_document` 和 `patch_collection` 是写操作，写失败报 404 尤其误导。
- 建议: 这几处统一换成 `AppError::from_lookup(err, "document not found: …")`；`patch_collection` 里 `update_collection` 的「没有这一行」目前是 `with_context` 出来的 anyhow，要么先 `get_collection` 判 404 再更新，要么让 `update_collection` 返回 `Option`。对前端的影响: 只有坏数据或库忙时由 404 变 500，前端对非 OK 统一走「读取失败」文案，无功能影响。
- 等级: 中

### 4. 列表的分页规矩都没跟上：`/documents`、`/library/books`、`/search` 静默夹、不回显 `limit`、没有 `has_more`
- 位置: `documents.rs:96-103`（`query.limit.clamp(1, MAX_DOCUMENT_LIMIT)`）；`views.rs:56-61`（`DocumentListView` 只有 `documents`、`total`）；`book_projection.rs:194-197`；`views.rs:357-360`（`LibraryBookListView` 只有 `items`）；`search.rs:13-16`（夹到 100）、`views.rs:150-162`；`database/retainpdf-db/src/db/documents/crud.rs:162-163`（`ORDER BY d.added_at DESC`，没有第二排序键）；`backend/packages/retainpdf-core/src/config/limits.rs:4-6`（已注明这几个「暂时还是静默夹」）；`docs/api/00-约定/03-分页.md:4,37`。前端 `documents.ts:65-71` 的 `DocumentListView` 类型声明了 `limit`、`offset`，后端从来没给。
- 现状: `/documents` 有 `total`，前端用 `offset + documents.length < total` 算有没有下一页（`document-library-source.ts:98-109`），能用；书架列表连 `total` 都没有；搜索只有 `hits`，`limit` 传 500 实际给 100，调用方不知道被截了。`/documents` 只按 `added_at` 排序，同一秒入库的书（批量上传）跨页可能重复或漏掉（未实跑）。`/collections` 不分页（合集数量小，可以接受，但应在约定里写明「不分页的列表」）。
- 问题: 违反规矩第 3 条。
- 建议: 只增不改：`DocumentListView` 加 `limit`（生效值）、`offset`、`has_more`（已有 `total`，直接算）；`SearchResultView` 加 `limit`、`has_more`（多取一条）；`LibraryBookListView` 冻结（第 1 条），不再加。排序加 `d.document_id DESC` 作第二键。`limit` 越界改 400 要先和前端确认：前端发的最大值是 500（`collections/domain/controller.ts:43,51,65`），正好等于上限，不受影响；搜索前端发 20、AI 工具夹到 1..=30（`tools.py:453`），都在 100 以内；AI 的 `list_documents` 工具上限 200，也在 500 以内。`?job_id=` 分支也回显这几个字段。`03-分页.md` 表里补上这两行。对前端的影响: 只加字段；前端可以改读 `has_more`。
- 等级: 中

### 5. 删除时「还有任务在跑」的 409 没有 `reason`
- 位置: `backend/packages/retainpdf-library/src/books.rs:130-137`（`ensure_deletable` 用的是 `AppError::conflict`，不是 `conflict_because`）；`DELETE /documents/:id` 和 `DELETE /library/books/:job_id` 共用；`docs/api/00-约定/02-认证与错误.md:98`（已写明「目前没有 `reason`」）；前端 `library/domain/documents/error-messages.ts:40-51`（没有 reason 的 409 一律当成「任务在跑」）。
- 现状: 同一个删除接口的两种 409，一种带 `job_has_dependents`，一种 `details` 是 `{}`。
- 问题: 违反规矩第 1 条（409 带 `error.details.reason`）。前端现在靠「409 且不是 dependents」推断，以后删除再多一种 409 就会被错说成「任务在跑」。
- 建议: 改成 `AppError::conflict_because("job_active", …)`，和任务操作的 `job_active` 同一个值；文档表里补一行。对前端的影响: 只多一个 `reason`，前端现有判断不变；方便时改成按 `reason === "job_active"` 判断。
- 等级: 中

### 6. `GET /collections/:id/agent-workspace`：不查合集存在、编号不校验、GET 会删目录重建；合集删了目录不清
- 位置: `backend/packages/retainpdf-library/src/reading.rs:71-86`；`collection_workspace.rs:83-88`（`data_root.join("collections").join(collection_id).join("ai")`）、`:146-153`（`list_documents(MAX_BOOKS, …)` 后直接 `remove_dir_all(books/)`、`create_dir_all`）、`:229-231`（写 `collection.v1.json`）；`collections.rs:68-80`（`delete_collection` 只删行）；前端 `reader/domain/terminal-scope.ts:91`、`frontend/web/dist/reader.bundle.js` 里的 `Ax`（空 `books` 时提示「这个文件夹里还没有翻译好的书」）。
- 现状:
  - 不存在的合集编号也返回 200：建出 `data/collections/<任意编号>/ai/books/` 和一份空清单。前端会提示「还没有翻译好的书」，而不是「合集不存在」。
  - 编号原样拼进路径。axum 0.7 会把路径参数百分号解码，`%2E%2E`、`%2F` 能进到 `collection_id` 里（未实跑），`<data_root>/collections/../<x>/ai/books` 会被 `remove_dir_all` 后重建、再写一份清单，落在 `collections/` 之外。AI 服务那边读工作区时专门挡了这个（`backend/ai/retainpdf_ai/fx_workspace.py:99-101`），Rust 写的这一侧没挡。多用户下这个接口是 `closed`，单机下调用方是本机前端。
  - GET 每次都删掉重建 `books/`、改写清单，是有副作用的读。
  - 合集删除后 `data/collections/<id>/` 永远留着（里面还有 agent 的产出）。
  - 一次最多 200 本（`MAX_BOOKS`），超出的书既不在 `books` 也不在 `skipped`，agent 以为合集只有这么多。
  - 已知（正确性审计第 7 条「链到在跑的半成品」）现状：`workspace_reading_book` 现在对 `ReadingTarget::Job` 也返回任务根目录（`reading.rs:88-115`），这一半修了；`None`（这本书还没有任何可读译文）仍退回 `document.active_job_id`，只查任务存在、不查状态（`collection_workspace.rs:177-216`），书卡指着在跑的任务时还是会链到半成品目录。
- 问题: 不存在的合集看起来像空合集；路径穿越能在数据目录里删、写别处的 `ai/books`（影响面有限，但这是 `remove_dir_all`）；GET 有副作用不合 HTTP 语义。
- 建议: 先 `get_collection` 判 404；`collection_id` 校验成 `^col-[A-Za-z0-9_-]+$`（`new_collection_id` 生成的就是这个形状），不合格 400；新增 `POST /collections/:id/agent-workspace`（物化），GET 旧地址行尾标 `old "/api/v1/collections/:collection_id/agent-workspace"`（同址换方法，按登记表的写法登记成新行）；`delete_collection` 成功后删 `data/collections/<id>/`（失败只记日志）；超过上限的书写进 `skipped`（reason「超过 200 本上限」）；`None` 分支直接进 `skipped`，不再退回 `active_job_id`。对前端的影响: 404 是新行为（前端已经把非 OK 当错误显示）；换 POST 要改 `terminal-scope.ts:91` 的请求方法，写进整理记录。
- 等级: 中

### 7. 请求体和查询参数里不认识的字段一律静默忽略；合集 PATCH 的 `parent_id` 尤其误导
- 位置: `views.rs:37-54`（`ListDocumentsQuery`）、`:65-70`（`PatchDocumentInput`）、`:150-158`（`SearchQuery`）、`:303-325`（`CreateCollectionInput`、`PatchCollectionInput`、`AddCollectionDocumentsInput`）、`:362-366`（`LibraryDeleteQuery`）——全都没有 `deny_unknown_fields`；`backend/api/src/app/router/collections.rs:12`（「改合集（名字、上级）」）；`collections.rs:50-65`（只改 `name`、`sort_order`）。
- 现状: `PATCH /collections/:id {"parent_id": "col-x"}` 返回 200，上级没变；`library-api.md:146-148` 又说 `parent_id`「传了也接受，但前端目前不需要用」。`DELETE /documents/:id?forse=true` 这种拼错会当成不强删（无害，但提示不了）。
- 问题: 违反规矩第 4 条。
- 建议: 前端实际发送：PATCH 书 `{title}` 或 `{reading_status}`；建合集 `{name, parent_id?}`；改合集 `{name}`；加书 `{document_ids}`；搜索 `q`、`limit`；书列表 `limit/offset/reading_status/collection_id/q`，外加 `tag`（第 2 条）。所以四个请求体现在就可以加 `deny_unknown_fields`；`ListDocumentsQuery` 等第 2 条的 `tag` 清掉以后再加。登记表说明改成「改合集（名字、排序）」。要不要支持改上级另议（要防环）。对前端的影响: 请求体收紧无影响；查询收紧前要先去掉 `tag`。
- 等级: 低

### 8. 返回结构：同一类东西几种写法、几个死字段、删除结果带服务器绝对路径
- 位置: `backend/packages/retainpdf-core/src/models/library.rs:8-47,105-114`；`views.rs:56-61,140-147,264-276,283-301,327-375,368-388`；`documents.rs:155-195`、`books.rs:62-96`。
- 现状:
  - `DocumentRecord`：`authors_json` 是 JSON 编码后的字符串（恒为 `"[]"`）、`year` 恒 null、`doi` 恒 `""`、`last_opened_at` 全仓没有写入点（恒 null）；空值时而 `""` 时而 null；`active_version_id` 为空时省略键，`active_job_id` 为空时给 null。
  - 列表的键名四种：`documents`、`collections`、`hits`、`items`（书架、任务）。
  - 合集：`POST …/documents` 返回整条 `CollectionRecord`，`DELETE …/documents/:id` 返回 `{removed: true}`，`DELETE /collections/:id` 返回 `{deleted: true}`——同一个 `CollectionMutationResult` 两种形状。`CollectionRecord.document_count` 的注释说「单条查询恒为 0」，实际 `COLLECTION_COLUMNS` 每次都现算（`database/retainpdf-db/src/db/collections.rs:9-10`），注释过时。
  - 书架列表项：`id` 与 `job_id` 重复、`title` 与 `display_name` 重复、`authors` 恒 null、顶层 `stage`/`stage_detail`（任务视图已经禁止顶层阶段字段，改放 `stage_snapshot`）。
  - 两个删除结果的 `removed_paths` 是服务器上的绝对路径（`path.to_string_lossy()`），别的接口都特意不暴露内部路径。
- 问题: 客户端要分别适配；死字段让人以为有功能。
- 建议: 只增不改为主：`DocumentRecord` 不删字段，在契约（第 9 条）里把 `authors_json/year/doi/last_opened_at` 标「未使用，恒为空」；`removed_paths` 改成相对 `data_root` 的路径（前端不读这个字段——`delete-actions.ts` 只看成功与否——可以直接改，或新增 `removed` 相对路径、旧字段保留）；修 `document_count` 注释。列表键名不动（改了就是破坏性的），在约定里记一笔。对前端的影响: 无。
- 等级: 低

### 9. 契约：书、合集、搜索没有任何 schema，前端类型手写成 `any`；书架契约里有已删接口
- 位置: `contracts/*.schema.json` 搜不到 `DocumentRecord`、`DocumentListView`、`CollectionRecord`、`BlockSearchHit`；`frontend/packages/api/src/documents.ts:6-12,65-71`（`DocumentRecord = Record<string, any> & {…}`，`DocumentListView` 还声明了后端不给的 `limit/offset`）、`collections.ts` 全部 `Promise<any>`、`search.ts:6 Promise<any>`；`contracts/library-books.v1.schema.json:37-42,754-790`（`POST /api/v1/library/books/batch-delete` 和 `LibraryBatchDeleteInput/ResultView`——这个接口已删，`docs/core/api/index.md:144` 写着「已删」）；同文件 `ListLibraryBooksQuery` 只有 `job_ids/limit/offset/q`，Rust 实际还认 `status/provider/include_live_stage`。`backend/contracts/` 是同一份镜像。
- 现状: 调用最多的 `GET /documents`（今日 1520 次）没有契约、没有 Rust 键集合测试；书架列表和详情有契约且 `job_view_contract.rs:291-292` 钉住，但书架正是该退场的那套。
- 问题: 改 `DocumentRecord` 不会有任何红灯，前端也拿不到类型。
- 建议: 新增 `library-documents.v1`（`DocumentRecord`、`DocumentListView`、`DocumentDeleteResultView`、`PatchDocumentInput`、`CollectionRecord`、`CollectionListView`、`SearchResultView`/`BlockSearchHit`），在 `api_tests` 里照 `job_view_contract.rs` 的写法加 `assert_exact_object_keys`；`library-books.v1` 删掉 `batch-delete` 接口和两个定义（契约测试和 TS 生成要一起跑），`ListLibraryBooksQuery` 补齐或注明「冻结」。对前端的影响: 无（只加契约，前端可以逐步换掉手写类型）。
- 等级: 中

### 10. `GET /documents?job_id=` 是塞在列表接口里的另一个接口
- 位置: `documents.rs:69-91`；`views.rs:45-46`；调用方 `documents.ts:170 fetchDocumentByJobId`（阅读器）、AI `rust_client.py:81`。
- 现状: 带 `job_id` 时忽略 `limit/offset/reading_status/collection_id/q`，也不看多用户的 `owner`（多用户已冻结，只记录）；返回 0 或 1 条。合并阅读的虚拟编号走 `MergedJobId::parse`，`get_document(..).ok()` 吞掉库错误。另一方面 `GET /jobs/:job_id` 的 `book_summary.document_id` 已经能给出同样的答案（`docs/core/api/index.md:376`）。
- 问题: 同一个地址两种语义，参数组合的行为没有文档（例如 `?job_id=x&collection_id=y` 会无视合集）。
- 建议: 保留（两个调用方在用），在文档里写清「带 `job_id` 时其余参数无效」；吞错改成 `from_lookup`（并入第 3 条）。长期可以让调用方改读 `GET /jobs/:id` 的 `book_summary.document_id`，但虚拟合并编号不走 `/jobs`，不急。对前端的影响: 无。
- 等级: 低

### 11. 文档陈旧：推荐的接入路径和现实相反、列着不存在的接口、没有书库专章
- 位置:
  - `docs/core/api/index.md:50-58`：「图书馆页面优先使用书籍语义接口 `/library/books`」，现实是前端以 `/documents` 为主、书架只取任务活态（第 1 条）。
  - `docs/core/api/index.md:173`：`DELETE /api/v1/documents/{document_id}/favorites`；`:735-736`：`GET|POST /api/v1/favorites`、`PATCH|DELETE /api/v1/favorites/{favorite_id}`——收藏 9 月已删，这几个地址都是 404。`:178` 说 `total` 应用了 `tag` 过滤。
  - `docs/core/frontend/library-api.md`：头部「更新于 2026-09-02」；§1 列 `tags[]`、`body: { title?, reading_status?, tags? }`、「`tags` 是整体替换语义」；§7「v1 只做扁平文件夹」「`parent_id` 传了也接受」；「建议的迁移路径」第一步还是做收藏；§1 的字段列表少了 `has_translation`、`title_source`、`title_locked`、`source_pdf_url`、`cover_url`、`thumbnail_url`；「两个必须处理的边界」第 1 条里删除保护写了两遍、前后措辞不一。
  - `backend/packages/retainpdf-library/src/documents.rs:133-135`、`backend/api/src/routes/library_data.rs:191`：注释说「被收藏引用则 409」，代码里没有这个检查（见下面「已知」第 2 项）。
  - 登记表说明：`router/documents.rs:12`「标签」（第 2 条）、`router/collections.rs:12`「上级」（第 7 条）。
  - `docs/api/` 下没有书库一章：书、合集、搜索只有接口目录里的一行，详情散在 `core/frontend/library-api.md` 和 `core/api/index.md` 两处且互相不一致。
- 问题: 照文档接的调用方会去用要退场的书架接口、调已经 404 的收藏接口、发没用的 `tags`。
- 建议: 在 `docs/api/` 下新开「书库」一章（书、合集、搜索、删除语义），以代码为准重写字段表，指向第 9 条新加的契约；`index.md` §2 改成「书用 `/documents`，任务活态用 `/jobs?job_ids=`，`/library/books` 冻结」并删掉收藏的几行；`library-api.md` 开头标「已被 `docs/api/<书库章>` 取代」或按现状重写；删掉两处收藏注释。和代码同一分支改。对前端的影响: 无。
- 等级: 中

### 12. 书、合集、搜索的处理函数在异步线程里直接跑 SQLite 和删目录
- 位置: `backend/api/src/routes/library_data.rs:38-60,135-149,192-210,271-278`；`routes/collections.rs` 全部；对照 `routes/library.rs:23-37` 用 `run_read_query_once`、`reading.rs:77-85` 用 `spawn_blocking`。
- 现状: `list_documents`、`get_document`、`patch_document`、`search_blocks`、合集增删改都是同步 rusqlite 调用直接写在 `async fn` 里；`DELETE /documents/:id` 还在里面对每个任务目录 `remove_dir_all`（几百 MB 的书会占住一个 tokio 工作线程好几秒，未实跑）。
- 问题: 库忙（写锁等待）或删大书时，同一工作线程上的其它请求（事件流、进度）跟着卡。`GET /documents` 是调用最多的接口。
- 建议: 读接口包一层 `run_read_query_once`（书架已经这么做），删除走 `spawn_blocking`。对前端的影响: 无。
- 等级: 低

### 13. 搜索和合集的几处小问题
- 位置: `database/retainpdf-db/src/db/documents/search.rs:103-125`；`collections.rs:16-38,82-106`；`retainpdf-library/src/api.rs:243-251`。
- 现状:
  - 搜索不到 3 个字时走 `LIKE`，`%`、`_` 是被**删掉**而不是转义（搜 `5%` 等于搜 `5`）；书列表的 `q` 用的是正确的 `ESCAPE '\'`（`crud.rs:143-150`），两处写法不一。
  - 命中只有 `document_id/job_id/page_idx/block_id/snippet`，没有书名；搜索岛和 AI 都要再查一次书才能显示是哪本。`query` 回显未 trim 的原文。
  - 建合集时上级不存在返回 404（请求体里引用了不存在的东西，按规矩第 1 条更像 400；加书时引用不存在的书也是 404）。
  - 加书时先逐本 `get_document` 校验、再开事务插入，两步之间书被并发删掉会撞外键 → 500（未实跑）。
  - 建合集是「插入行」和「写归属」两步（`api.rs:243-251`），第二步失败时合集已建、请求报错（多用户已冻结，只记录）。
- 建议: `LIKE` 分支改成转义；命中加 `title`（只增）；上级/书不存在改 400 先和前端确认（前端只显示 message，不看状态码，`CollectionDialog.tsx`、`collections.ts` 都是）；加书的校验挪进事务。
- 等级: 低

## 已知（2026-10-19 正确性审计第二节），现状

- **删书不清 `data/documents/<id>/`**（正确性审计 2-书库与归属 第 5 条）：没变。`delete_document`（`documents.rs:136-196`）仍只删任务目录、zip、上传目录，封面/缩略图/页预览/合并阅读缓存/`ai/` 都留着；DB 删完后文件删除中途 `?` 失败会让请求报 500，而书已经没了，重删是 404。
- **有收藏记录的老库删书 500**（第 6 条）：没变。`delete_document_cascade`（`database/retainpdf-db/src/db/documents/crud.rs:587-612`）仍是先 `delete_job_rows` 再删 `documents`，`favorites.job_id ... ON DELETE RESTRICT` 会在第一步拦下；`delete_library_book` 同样。代码注释里「被收藏引用则 409」仍在（第 11 条）。
- **合集 agent 工作区链到在跑的半成品**（第 7 条）：修了一半，见第 6 条最后一项。
- 顺带确认正确性审计里另外两条已经修好：强删先取消（`routes/library.rs:58-66`、`library_data.rs:192-205` 调 `cancel_before_delete`，第 1 条）；删任务后重建全文索引（`books.rs:79-85` 调 `rebuild_document_fts`，第 4 条）。`docs/core/api/index.md:153-158` 也已写明。

## 查过、没问题的

- 路由全部在 `crate::endpoints!` 表里，`routes.txt`、`00-接口目录.md` 与表一致；本组没有直接写 `.route(` 的。
- 整本删除的数据库部分在一个 IMMEDIATE 事务里（任务、事件、上传、`blocks_fts`、文档行），文件在事务提交后才删；删任务（书架删除）也是先事务删行再删文件，删完对书卡 `reconcile_document_active_job`、重建全文索引。依赖检查 `jobs_depending_on` 排除了删除范围内的任务，整本删除只在依赖者在别的书里时才 409，和 `02-认证与错误.md:96` 一致。
- `collection_documents` 对书和合集都是 `ON DELETE CASCADE`，删书、删合集都不留悬空成员；加书 `INSERT OR IGNORE` 幂等；`document_count` 每次现算。
- `PATCH /documents/:id` 的标题校验（1..=512 字）、`reading_status` 枚举校验是 400；改标题和写「用户锁定」在同一事务里，自动命名不会覆盖手改的标题。
- `GET /documents` 的列表和 `total` 在同一事务里读，翻页计数一致；`q` 用 `LIKE … ESCAPE` 正确转义。
- 全文索引用 `trigram` 分词，中文 3 字以上能命中；`q` 里的双引号被替换，不会拼出 FTS 语法错误；空 `q` 返回空结果而不是全表。
- 书架列表 `job_ids` 不按 `limit` 截断（测试 `jobs_list.rs:146`），满足前端「按名单精确取」的用法。
- 书架和文档列表上的封面、缩略图都是 `/documents/:id/files/…` 新地址；查不到书编号时为 null，没有退回旧的任务级地址。
- 多用户相关（`owner_filter`、`classify_param` 里 `parent_id` → 合集）只看了有没有，按约定不细查。
