# 书上的动作与元数据（documents 的 translate / ocr / metadata-suggestions / agent-versions，以及上传和书的关系）

审计范围：`POST /documents/:document_id/translate`、`POST /documents/:document_id/ocr`、
`GET|POST /documents/:document_id/metadata-suggestions`、`POST /documents/:document_id/metadata-suggestions/:suggestion_id/apply`、
`GET /documents/:document_id/agent-versions`；`POST /uploads` 只看它和书的关系（其余部分任务审计 A 组已看过）。
只读代码和现有快照得出（main，`f4574fc0`），没跑测试、没调运行中的服务。标「未实跑」的是读代码推断的结论。
今日调用数取自 `data/diagnostics/route-usage.json` 的汇总，这几条全部是 0。

## 接口一览

| 方法 地址 | 处理函数 | 返回类型 | 契约 | 文档 | 调用方（前端文件:行） | 备注 |
|---|---|---|---|---|---|---|
| POST `/api/v1/documents/:document_id/translate` | `routes/library_data.rs:216 translate_document_route` → `retainpdf-library/src/translate.rs:18 translate_document` → `JobsFacade::create_document_submission` → `submit_job` | `JobSubmissionView`（拆段时多 `sibling_job_ids`） | 请求：`create-job.v1`（和 `POST /jobs` 同一个 `CreateJobInput`）；响应：无 | `api-spec/jobs-submission.md:146,206`、`docs/api/01-任务/01-创建任务.md:57`、`docs/core/api/index.md:170`、接口目录 | `frontend/packages/api/src/documents.ts:350 translateDocument` ← `features/library/domain/documents/submit-actions.ts:40` ← `book-detail/ui/use-book-detail-translate.ts:104` | signed；`submissions.txt` 有 4 个用例 |
| POST `/api/v1/documents/:document_id/ocr` | `library_data.rs:236 ocr_document_route` → `library/src/ocr.rs:13 ocr_document` → 同上 | `JobSubmissionView` | 同上 | 同上（只列地址） | `documents.ts:369 ocrDocument` ← `submit-actions.ts:71` ← `book-detail/ui/use-book-detail-ocr.ts:50` | signed；`ocr_document` 是 `async fn` 但里面没有 await |
| GET `/api/v1/documents/:document_id/metadata-suggestions` | `library_data.rs:163` → `library/src/metadata_suggestions.rs:150 list_metadata_suggestions` | `{ suggestions: [DocumentMetadataSuggestionView] }` | 无 | `api-spec/document-metadata.md` | `documents.ts:291 fetchDocumentMetadataSuggestions` ← `app/home/composition/create-runtime-features.ts:130` ← `document-auto-naming.ts:76` | signed |
| POST `/api/v1/documents/:document_id/metadata-suggestions` | `library_data.rs:150` → `metadata_suggestions.rs:30 create_metadata_suggestion` | `DocumentMetadataSuggestionView` | 无 | 同上 | `documents.ts:269 createDocumentMetadataSuggestion` ← `create-runtime-features.ts:131` ← `document-auto-naming.ts:83`（任务成功后自动发，带 `apply_if_default: true`） | 不调外部 AI，纯本地规则 |
| POST `/api/v1/documents/:document_id/metadata-suggestions/:suggestion_id/apply` | `library_data.rs:176` → `metadata_suggestions.rs:168 apply_metadata_suggestion` | `{ suggestion, document: DocumentRecord }` | 无 | 同上 | `documents.ts:311 applyDocumentMetadataSuggestion` 有封装，但前端没有任何地方调用它 | signed；实际没人用 |
| GET `/api/v1/documents/:document_id/agent-versions` | `routes/public_document_operations.rs:22` → `retainpdf-ai/src/public_document_operations.rs:195 list_document_agent_versions` | `DocumentAgentVersionListView { versions, active_version_id, total, limit, offset, has_more }` | 无 | `api-spec/jobs-document-history.md:101-121`、`api-spec/jobs.md:57`、`docs/api/README.md:68`、`docs/core/api/index.md:172` | 没有调用方（前端、`backend/ai`、`backend/cli`、脚本都没有），只有 `api_tests/public_document_operations.rs:576` | closed；`ops/reports/api-endpoint-audit.md:45` 已记"前端未接" |
| POST `/api/v1/uploads`（只看和书的关系） | `routes/uploads.rs::upload_pdf` → `retainpdf-ingest/src/uploads/staging.rs:83 publish_upload` → `db/uploads.rs:50 save_upload_with_document_for` | `UploadView { upload_id, document_id, ... }` | 无 | `api-spec/jobs-submission.md`（缺 `document_id`，A 组已记） | `config-port.ts:7`、`jobs-submit.ts:41` | 上传即建书：书号 = 文件字节的 sha256，和上传行同一事务 |

## 问题

### 1. 生成元数据建议在 async 处理函数里直接做重活：整本 PDF 用 lopdf 全量解析、最多 256MB 的 JSON 读进内存再解析
- 位置: `backend/api/src/routes/library_data.rs:150-161`（处理函数直接调同步的 `create_document_metadata_suggestion_view`）；`backend/packages/retainpdf-library/src/metadata_suggestions.rs:39`（`read_pdf_metadata_title` → `:397 lopdf::Document::load(path)`）、`:26,68-90`（`MAX_NORMALIZED_DOCUMENT_BYTES = 256MB`，`fs::read` 整个读入，再 `serde_json::from_slice` 成 `Value`）。对照同一个包的 `reading.rs:48,65,77`（`document_reading`、`translation_coverage` 都放进了 `spawn_blocking`）。
- 现状: 为了取 PDF Info 里的一个 `Title`，把整本 PDF 的对象全部解析一遍；规范化文档整份读入并建成 `serde_json::Value`（内存是文件大小的数倍），然后算 sha256。全程在 tokio 工作线程上。前端在**每个 OCR/翻译任务成功后自动**调一次（`document-auto-naming.ts:83`），所以这不是偶发操作。
- 问题: （未实跑）几百页、上百 MB 的书，这一调用会占住一个运行时线程数秒，同时间的其它请求（进度、事件流）跟着卡。lopdf 全量加载对加密或损坏的 PDF 还可能很慢。
- 建议: 处理函数里用 `spawn_blocking` 包住（照 `document_reading` 的写法），这是一行级别的改动。另外两处可以顺手降成本：PDF 标题只读 trailer 和 Info（`lopdf::Document::load_metadata` 或只解析 trailer），规范化文档只取前 8 页的块（现在 `extract_ocr_title_candidates` 本来就只看前 8 页），证据哈希可以流式算。对前端的影响: 无。
- 等级: 中

### 2. 同一本书重新上传（文件名不同）以后，自动命名永远不再生效，而且没有任何提示
- 位置: `database/retainpdf-db/src/db/documents/crud.rs:207-236`（`upsert_document_from_upload_on` 的 `ON CONFLICT DO UPDATE` 覆盖 `source_filename`，不动 `title`）；`database/retainpdf-db/src/db/document_metadata_suggestions.rs:184-193`（应用前要求 `title == default_title_from_filename(source_filename)`）；`metadata_suggestions.rs:124-145`（`apply_if_default` 遇到 `TitleChanged` 静默返回未应用的建议）、`:455-458`（`can_apply`）。
- 现状: 先传 `a.pdf`，书名是 `a`、`title_source=filename`。再传同一内容的 `b.pdf`：书号相同，`source_filename` 变成 `b.pdf`，书名还是 `a`，`title_source` 仍是 `filename`、没锁。之后任务成功、前端自动建建议：`title != "b"`，判成"标题已被改过"，`apply_if_default` 不应用，`can_apply=false`；手动 apply 得到 409 `DOCUMENT_TITLE_CHANGED`（"文档标题已经被修改"），但用户从没改过。
- 问题: （未实跑）"标题是不是还是默认值"靠的是 `title` 和 `source_filename` 字符串相等，而 `source_filename` 会被重新上传改掉。`title_source` 已经单独记了来源（`filename`/`user`/`ocr`…），用它判断就够了。和正确性审计 2-#10（重新上传覆盖文档元数据）同一个根因，后果不同，这里单列。
- 建议: 守卫只看 `title_locked` 和 `title_source == "filename"`，去掉和 `source_filename` 的字符串比较（数据库层 `document_metadata_suggestions.rs:186-189` 和视图层 `metadata_suggestions.rs:456-458` 两处一起改）；或者按 2-#10 的建议让 `ON CONFLICT` 不再覆盖 `source_filename`。对前端的影响: 无字段变化；之前不会自动命名的书会开始自动命名。
- 等级: 中

### 3. `apply` 和 `PATCH /documents/:id` 返回的书不一样，乐观锁只有 apply 有；apply 实际没人用
- 位置: `metadata_suggestions.rs:179-185`（`document` 直接用数据库读出的 `DocumentRecord`）；对照 `library/src/documents.rs:203-234 patch_document` 和 `:107-117 get_document`（都过一遍 `with_document_media_urls`）；`retainpdf-core/src/models/library.rs:31-39`（`source_pdf_url/cover_url/thumbnail_url` 是 `skip_serializing_if = "String::is_empty"`）；`database/.../crud.rs:344-381 update_document_fields`；`frontend/packages/api/src/documents.ts:311`。
- 现状:
  - apply 回来的 `document` 少三个键：`source_pdf_url`、`cover_url`、`thumbnail_url`（空串被跳过，键直接不存在）。PATCH、GET 都有。
  - apply 支持 `expected_document_updated_at`（过期 409），用 IMMEDIATE 事务；PATCH 没有任何版本护栏，后写的覆盖先写的。
  - PATCH 改标题一律设 `title_source=user, locked=1`，哪怕改成和现在一样的值；之后所有建议都不能再应用（这是设计，文档里写了）。
  - 前端只调"生成（带 `apply_if_default`）"和"列表"，`applyDocumentMetadataSuggestion` 有封装但全仓没有调用点；`backend/ai`、`backend/cli` 也不调。
- 问题: 同一本书两个写接口回两种形状，前端若用 apply 的回包刷新书卡会丢封面链接。apply 是一个没有调用方的写接口。
- 建议: apply 的 `document` 也过 `with_document_media_urls`（处理函数已经能拿到 `headers`，加一个 `base_url` 参数即可）；PATCH 加可选的 `expected_updated_at`（只增，不传不检查）。apply 留着（规则完整、有测试），在文档里注明"前端目前只用 `apply_if_default`"。对前端的影响: 只增字段，无破坏。
- 等级: 中

### 4. `/documents/:id/ocr` 和 `/translate` 悄悄改写客户端字段，和 `POST /jobs` 的严格程度不一致
- 位置: `library/src/ocr.rs:30-33`；`library/src/translate.rs:36-43,62-66`；`submissions.txt:398-460`。
- 现状:
  - `/ocr`：不管请求里 `workflow` 写的是什么，都改成 `ocr`；`source.artifact_job_id`、`source.source_url` 有值也直接清掉。发 `{"workflow":"book"}` 到 `/ocr` 会成功建一个 OCR 任务。
  - `/translate`：`workflow=ocr|render` 返回 400（不改写）；`source.source_url` 有值则悄悄清掉；`source.upload_id` 必须等于"这本书最新的那份上传"，否则 400（同一本书有多份上传时，用旧的那个 `upload_id` 也会被拒，见 `crud.rs:418-433` 按 `uploaded_at DESC` 取一条）。
  - 两个接口反序列化都用同一个 `CreateJobInput`（`deny_unknown_fields`，未知字段 400），这一点和 `POST /jobs` 一致；不一致的是上面这些"认得但不该出现"的字段的处理。
  - `submissions.txt` 只覆盖了 `/translate` 的正常、带 `runtime.job_id`、`workflow=ocr`，以及 `/ocr` 的正常、带 `runtime.job_id`；没有 `/ocr` 带 `workflow=book`/`artifact_job_id`、`/translate` 带 `source_url`、复用 OCR、拆段这几种。
- 问题: 规矩 1、4 说请求写错一律 400、不悄悄忽略。同一组接口一个 400 一个静默改写，调用方以为提交的是 A、实际建了 B。
- 建议: `/ocr` 对 `workflow` 非空且不是 `ocr`、`source.artifact_job_id` 非空、`source.source_url` 非空返回 400；`/translate` 对 `source.source_url` 非空返回 400；`upload_id` 校验放宽成"属于这本书的任一上传"（`content_hash == document_id`）。`workflow` 有默认值（缺省是 `book`），要区分"没发"和"发了 book"，得在处理函数里先收 `Value` 看键是否存在（`POST /jobs` 本来就是先收 `Value`）。补对应的 `submissions.txt` 用例（`UPDATE_SUBMISSIONS=1`）。对前端的影响: 前端 OCR 发的是 `{workflow:"ocr", ocr:{page_ranges}}`（`use-book-detail-ocr.ts:36-44`），翻译不发 `source.upload_id`/`source_url`（`submit-payloads.ts` 里没有），收紧后不受影响；改之前再 grep 一遍 `submit-payloads.ts`。
- 等级: 中

### 5. 错误形状：元数据接口的 409/404 没有 `error.details.reason`，同一个码两种信封；422 不在约定里；"原文 PDF 不在盘上"报成普通 404
- 位置: `backend/packages/retainpdf-api-base/src/error.rs:65-69,363-373,435-450`（`DocumentMetadata` 变体：顶层 `code` 是字符串，`error.details` 恒为 `{}`）；`metadata_suggestions.rs:62-101,186-207,231-257`；`library/src/documents.rs:58-63`（`source pdf missing on disk` → 404）；对照 `error.rs` 的 `OcrArtifactReuse`（`OCR_JOB_NOT_FOUND` 带 `reason`、`can_fallback_to_ocr`）和 `docs/api/00-约定/02-认证与错误.md` 的 `artifacts_missing`。
- 现状:
  - 元数据的 7 个稳定码（`DOCUMENT_TITLE_CHANGED`、`DOCUMENT_REVISION_CONFLICT`、`DOCUMENT_METADATA_JOB_MISMATCH`、`DOCUMENT_METADATA_SOURCE_NOT_READY` 等）只放在 `code`/`error.code`，`details` 是空的。
  - `OCR_JOB_NOT_FOUND` 这个字符串码两处在用：建任务复用 OCR 时带 `reason=job_not_found`、`can_fallback_to_ocr=true`；元数据接口里同名码什么都不带。
  - `DOCUMENT_METADATA_SOURCE_INVALID`、`DOCUMENT_TITLE_CANDIDATE_NOT_FOUND` 是 422，约定文档只有 400/401/404/409/500。
  - `/translate`、`/ocr`、生成建议三处共用 `require_document_upload`：书在、原文 PDF 文件丢了 → 404 `source pdf missing on disk for document`，没有 `reason`。同样的情况在任务操作里是 409 `artifacts_missing`。
- 问题: 违反规矩"409/404 带 `error.details.reason`"；客户端只能靠 `error.code` 或 message 分支，同名码信封又不一样。
- 建议: `DocumentMetadata` 变体把码同时写进 `error.details.reason`（取小写，如 `title_changed`、`revision_conflict`、`job_mismatch`、`source_not_ready`），只增；元数据接口的"任务不存在"改用不和 OCR 复用撞名的码（如 `DOCUMENT_METADATA_JOB_NOT_FOUND`），旧码先保留在顶层 `code`；422 两条要么在约定里补一行"证据存在但无法使用"，要么并入 409；原文丢失改 409 + `reason=artifacts_missing`。状态码变化（404→409）先和前端确认：前端翻译错误走 `friendlyTranslateError`、OCR 错误只取 `message`（`documents.ts:384`、`submit-actions.ts:85-92`），不按 404 分支。
- 等级: 中

### 6. 数据库读错一律说成"书不存在"（404）
- 位置: `library/src/documents.rs:46-49`（`require_document_upload`，三个动作接口都经过它）；`metadata_suggestions.rs:155-158`（列表）、`:231-237`（`get_job` 任何错误 → 404 `OCR_JOB_NOT_FOUND`）；`retainpdf-ai/src/public_document_operations.rs:205-206`（agent-versions）。对照 `database/retainpdf-db/src/db/documents/crud.rs:266-273`：`get_document` 已经特意用 `QueryReturnedNoRows` 标出"没有这本书"，注释写着"其余错误是 500"。
- 现状: 上面几处都是 `.map_err(|_| AppError::not_found(...))`，把数据库层做好的区分丢了。库被锁、行解码失败都回 404。
- 问题: 违反约定（`02-认证与错误.md`：只有确实没有这条记录才是 404）。A 组 #7 记过任务侧的同一问题，这里是书这一侧。
- 建议: 在 `retainpdf-api-base` 加一个 `document_or_404`（看 `QueryReturnedNoRows`，其余转 Internal），这几处统一调用；`get_job` 那处复用已有的 `load_job_or_404` 的修法。对前端的影响: 只在坏数据/库锁时 404 变 500。
- 等级: 中

### 7. 生成、应用建议的请求体不拒绝未知字段
- 位置: `retainpdf-library/src/views.rs:72-99`（`CreateDocumentMetadataSuggestionInput`、`ApplyDocumentMetadataSuggestionInput`、`ListDocumentMetadataSuggestionsQuery` 都没有 `#[serde(deny_unknown_fields)]`）。
- 现状: 拼错的字段（如 `apply_if_defualt`、`expected_updated_at`）会被悄悄忽略：前者变成"不自动应用"，后者变成"不检查版本"，都是静默的语义变化。
- 问题: 违反规矩 4。
- 建议: 两个请求体加 `deny_unknown_fields`。前端发的是 `{job_id, fields:["title"], apply_if_default:true}`（`document-auto-naming.ts:83-87`）和 `{}`（apply 没人调），收紧不影响前端。查询串的未知参数按 `03-分页.md` 第 5 条，等列表统一收紧时一起做。
- 等级: 低

### 8. 建议列表、Agent 版本列表的分页还是老规矩
- 位置: `metadata_suggestions.rs:27,161`（`limit.clamp(1, 100)`）、`views.rs:86-91,141-144`（返回只有 `suggestions`）；`retainpdf-ai/src/public_document_operations.rs:207`（`clamp(1, 100)`）。
- 现状: 建议列表静默夹 `limit`，不回显 `limit`、不给 `has_more`，也没有 `offset`；Agent 版本列表有 `total/limit/offset/has_more`（形状是对的），但 `limit` 也是静默夹。两个列表的键分别叫 `suggestions`、`versions`，而 `/documents/:id/jobs` 叫 `items`。
- 问题: 不符合 `03-分页.md` 的规矩 1、2。前端自动命名只看前 20 条建议（`fetchDocumentMetadataSuggestions` 默认 `limit=20`），找不到就再 POST 一次；因为 POST 是幂等的（见"查过、没问题的"），超过 20 条也只是多一次请求，不会出错。
- 建议: 建议列表加 `limit`、`has_more`（多取一条），`limit` 越界 400；Agent 版本列表越界改 400。旧键 `suggestions`/`versions` 不动。改完把这两个列表加进 `03-分页.md` 的表。对前端的影响: 前端发 `limit=20`，在范围内，不受影响。
- 等级: 低

### 9. 建议生成里自己拼了规范化文档的路径，架构检查没拦住
- 位置: `metadata_suggestions.rs:270-281`（`stored_path(..., StoredFile::NormalizedDocument)` 找不到时退回 `resolve_job_root(job, data_root).join("ocr/normalized/document.v1.json")`）；`backend/api/scripts/check_architecture.py:1697-1701`（`ARTIFACT_PATH_RESOLVER` 只匹配 `resolve_output_pdf|resolve_normalized_document|...`，不含 `resolve_job_root`）。
- 现状: CLAUDE.md 规定"产物文件路径只许出口模块拼"，`KNOWN_ARTIFACT_PATH_READERS` 已清空。这里用任务根目录加硬编码的相对路径，绕过了 `stored_path`，正则也抓不到。
- 问题: 流水线改输出布局时这条退路会悄悄失效；也说明 `stored_path` 对某些老任务找不到规范化文档，该补在 `stored_path` 里，而不是调用方各自兜底。
- 建议: 把这条退路挪进任务包 `files/downloads/stored.rs` 的 `resolve_normalized_document`（如果确实有老任务需要），这里只调 `stored_path`；检查脚本的正则加上 `resolve_job_root\s*\([^)]*\)\s*\.\s*(?:map\s*\(\s*\|[^|]*\|\s*[^)]*)?join` 这一类，或者更简单地禁止出口模块之外出现 `"ocr/normalized/`、`"rendered/` 这类产物目录字面量。对前端的影响: 无。
- 等级: 低

### 10. 建任务入口的检查顺序和 `POST /jobs` 不同：书的入口先查状态再查请求
- 位置: `library/src/translate.rs:25-60`（`require_document_upload`、复用 OCR 校验、拆段校验都在前面）、`:152` 和 `retainpdf-jobs/src/lifecycle/submission.rs:28`（`reject_client_job_id` 到最后才查）；对照 `submission.rs:16`（`POST /jobs` 第一步就查）；`routes/jobs/lifecycle.rs:25-26` 对照 `routes/common/extractors.rs:39-57`。
- 现状:
  - 同一个请求带 `runtime.job_id` 且复用了一份不可用的 OCR：`POST /jobs` 回 400（`runtime.job_id is set by the server`），`/documents/:id/translate` 回 409（`OCR_*`）。约定写明"先看请求写没写对（400），再看状态（409）"。
  - 未知字段的提示两种写法：`POST /jobs` 先收 `Value` 再转，报 `invalid job payload: unknown field ...`；书的两个入口直接用 `ApiJson<CreateJobInput>`，报 `unknown field ... in the request body`；字段类型错时书的入口给固定文案，`POST /jobs` 给 serde 原文。
- 问题: 三个建任务入口（`create_submission`、`create_document_submission`、拆段）的 400 判定散在不同位置，`runtime.job_id` 的拒绝点要靠"最后都会走到 `create_document_submission`"来保证。
- 建议: `translate_document`、`ocr_document` 第一行就调 `reject_client_job_id`（连同第 4 条新增的 400 检查放在一起）；`POST /jobs` 改成 `ApiJson<CreateJobInput>`，三个入口错误文案一致（`submissions.txt` 会变，`UPDATE_SUBMISSIONS=1` 重录）。对前端的影响: 前端按 message 显示，只是文字变化。
- 等级: 低

### 11. 同一个 PDF 每上传一次就多存一份完整副本，而且永远不回收；上传回包不告诉调用方书是不是已经存在
- 位置: `retainpdf-ingest/src/uploads/staging.rs:48-81`（每次新建 `uploads/<upload_id>/` 并写全量字节）；`database/retainpdf-db/src/db/retention.rs:93-110`（孤儿上传回收排除 `content_hash` 在 `documents` 里的上传）；`retainpdf-ingest/src/uploads/view.rs:7-16`。
- 现状: 上传即建书，书号是文件 sha256；同一内容再传，`documents` 只有一行（`ON CONFLICT`），`uploads` 多一行、盘上多一份副本。只要书还在，这些副本都受"有 documents 行"的保护，回收器不碰，删书时才一起删。`UploadView` 只有 `document_id`，看不出这本书是新建的还是早就有。`/documents/:id/translate` 只用最新的一份（`crud.rs:418-433`）。
- 问题: 反复上传同一本大书，磁盘线性增长；调用方（前端上传流程）没法提示"这本书已经在书库里了"。
- 建议: 上传时若已有同哈希、文件还在盘上的上传，复用它（直接返回旧的 `upload_id`，或新行指向同一个文件 / 硬链接）；`UploadView` 只增一个 `document_existed: bool`。对前端的影响: 只增字段；如果改成复用旧 `upload_id`，前端拿到的 `upload_id` 可能和上一次相同，前端只是把它传给 `POST /jobs`，没有按 `upload_id` 去重的逻辑（改前 grep `upload_id` 确认）。
- 等级: 低

### 12. 文档和契约缺口
- 位置与现状:
  - `backend/api/docs/api-spec/document-metadata.md`：错误码表没有 HTTP 状态码；缺 `DOCUMENT_METADATA_FIELD_UNSUPPORTED`（400）和 `DOCUMENT_METADATA_SOURCE_TOO_LARGE`（413）；"候选优先级是 OCR 三种、然后 PDF Info"不准：`ocr_layout` 置信度 0.84 低于 `pdf_metadata` 的 0.92，第 2 页之后每页再减 0.03（`metadata_suggestions.rs:313-326`），所以靠后页的 OCR 标题会排在 PDF Info 后面；没写"同样的证据重复 POST 返回同一条建议"。
  - `api-spec/jobs-submission.md:206-214` 讲了 `/documents/:id/translate` 的 OCR 复用，但没写"复用 OCR + 指定页码时按连续段拆成几个任务、回包带 `sibling_job_ids`、主任务是排序后的第一个（重新 OCR 的段排最前，不是按页码）"（`translate.rs:47-50,129-159`）。前端已经在用（`use-book-detail-translate.ts:78`）。
  - `docs/api/` 下没有书库这一块的说明，`/documents/:id/translate|ocr`、元数据建议只在接口目录里有一行；用户文档 `docs/api/01-任务/01-创建任务.md:57` 只提了 `runtime.job_id`。
  - `docs/core/api/index.md:173` 列了 `DELETE /api/v1/documents/{document_id}/favorites`，`routes.txt` 里没有这个接口。
  - `JobSubmissionView` 没有 `document_id`，前端 OCR 回调写的是 `result.document_id || documentId`（`use-book-detail-ocr.ts:54`），一直取的是后备值。
  - 契约：`DocumentMetadataSuggestionView`、`DocumentMetadataSuggestionApplyView`、`DocumentAgentVersionListView` 都不在 `contracts/` 里，前端 `documents.ts:6-54` 手写类型（`DocumentRecord` 是 `Record<string, any>`）。`JobSubmissionView` 缺契约，A 组 #5 已记。
- 建议: 补 `document-metadata.md` 的状态码、两个码、真实的排序规则和幂等说明；`jobs-submission.md` 补拆段一节；删 `index.md` 里的 favorites 行；`JobSubmissionView` 只增 `document_id`（书的入口一定有，`POST /jobs` 有就填）；元数据视图并进计划中的 `library-books` 或新建 `document-metadata.v1`，在 `api_tests/library_data.rs:275` 那组测试旁边加键集合断言。对前端的影响: 只增。
- 等级: 低

### 13. `agent-versions`：关着的、没有调用方的历史接口
- 位置: `backend/api/src/app/router/documents.rs:38-39`；`routes/public_document_operations.rs:22-36`；`retainpdf-ai/src/public_document_operations.rs:195-248`。
- 现状: 多用户下 `closed`，单机可用。返回的 `download_path` 指向 `/api/v1/ai/operations/{id}/candidate.pdf`（也是 closed，存在）。前端、AI 服务、CLI 都没有调用；`ops/reports/api-endpoint-audit.md:45` 建议"Agent 功能还在规划就留，否则删"。实现本身没问题：只读、不暴露内部路径、有分页元数据，问题只有第 6 条（404）和第 8 条（静默夹）。
- 问题: 一个没人用、文档却在三处列出的接口；挂在 `/documents` 下、实现却在 AI 包里，`routes/public_document_operations.rs` 里只有它一个是书的接口。
- 建议: 和 AI 那组的文档操作接口（`/ai/operations/...`）一起定删留，不单独动；若留，在接口目录里注明"Agent 文档操作的历史，前端未接"。对前端的影响: 无。
- 等级: 低

### 已知（正确性审计已记，看了一眼现状）
- 正确性审计 2-#9(a)：`submit_job` 在落库并 `launch` 之后才做"归到这本书了没有"的检查（`retainpdf-jobs/src/lifecycle/creation/submit.rs:149-156` 现为同一段），失败时任务已在跑、客户端拿到 500。**没变化**。
- 正确性审计 2-#9(b)：拆段提交（`library/src/translate.rs:150-153`）第 k 段建失败时前 k-1 段已开跑，回包不带已建的编号。**没变化**。
- 正确性审计 2-#10：同内容重新上传用原件的页数、字节数、文件名覆盖文档元数据（`crud.rs:220-224`）。**没变化**；本组第 2 条是它的另一个后果。

## 查过、没问题的

- 三个建任务入口都走 `submit_job`：`/documents/:id/translate`、`/ocr` 经 `create_document_submission`（`submission.rs:22-34`），拆段时每段也经它；`check_job_entry_ownership` 守着。开发栈默认值、平台模型、凭据入库、预扣页额度、归书、发射都只在 `submit_job` 里写一次（`translate.rs:27` 提前补一次开发默认值是为了拆段预检，`submit_job` 里再补一次是幂等的）。
- 请求字段映射没有漂移：书的两个入口和 `POST /jobs` 反序列化成同一个 `CreateJobInput`（`deny_unknown_fields`、`create-job.v1` 契约钉住），书的入口只覆盖 `source.*` 和 `workflow`，其余字段原样透传。
- `runtime.job_id`：两个书的入口都拒（`submissions.txt:440-448` 钉住），`submit_job` 还有格式和查重兜底；只是拒绝的时机靠后（第 10 条）。
- 元数据建议不调外部 AI：只读 PDF Info 和规范化 OCR 文档，按固定规则打分；`needs_ai_review` 只是一个标记，目前没有消费方，文档也写明 AI 兜底是以后的事。
- 重复调用：生成建议是幂等的，(书, 源任务, 证据哈希, 选中标题) 上有唯一索引，`INSERT OR IGNORE` 后按这四个键读回（`document_metadata_suggestions.rs:63-110`，`v12_...sql`），并发两次也只有一行；前端另有 in-flight 去重（`document-auto-naming.ts:57-66`）。重复 apply 也幂等（已应用且标题相同直接返回）。
- 用户改名保护：PATCH 改标题设 `title_source=user, locked=1`；apply 在 IMMEDIATE 事务里检查锁和来源，用户改过的标题不会被建议覆盖（`api_tests/library_data.rs:339` 有测试）；`apply_if_default` 遇到冲突不报错、返回未应用的建议，符合"自动命名是尽力而为"。
- 指定 `job_id` 时校验它属于这本书（不属于 409），不会拿别的书的 OCR 给这本书起名。
- 上传和建书同一事务（`save_upload_with_document_for`），书号由内容决定，同一内容并发上传不会出现两本书；失败时 `PendingUpload` 的 Drop 清掉目录。
- 拆段提交在建任何任务之前先把所有段校验完（OCR 凭据、复用条件），校验失败不会留下半截任务（创建失败的情况见"已知"）。
- 路由登记：6 个地址都在 `router/documents.rs` / `router/ingestion.rs` 的登记表里，和 `routes.txt`、接口目录一致；路径参数统一 `:document_id`，`suggestion_id` 已在 `access.rs:46` 归类。
