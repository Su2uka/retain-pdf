# 后端改动交接（给前端）

> 注：本文是带日期的交接记录，里面写的旧地址已于 2026-10-18 全部删除（返回 404），新旧对照见 [任务接口分类](../../api/01-任务/04-任务接口分类.md) 和 [下载总览](../../api/07-产物下载/01-下载总览.md)。

本文只记**需要前端配合才能落地**的后端改动：新端点、新字段、新错误码，以及
前端具体要改哪个文件。纯后端的加固（事务边界、超时收尾之类）不在这里。

按后端提交倒序排列，最新在上。

---


## 0000000000. 书库标签残留清掉、`/assets` 两个接口删掉

分支 `chore/drop-tags-and-assets`（2026-10-19）。对应书库审计 A2、B11、B12，决定和取舍见
[删标签残留和资产接口](../../ops/planning/api-reorg/20261019-删标签残留和资产接口.md)。

### 变了什么

1. `POST /api/v1/assets`、`GET /api/v1/assets/:asset_id` 删了（404）。前端没有调用方，不用改。
2. 后端不再提标签：AI 的 `list_documents` 工具去掉 `tag` 参数和结果里的 `tags`；接口目录里书的 PATCH 说明改成「书名、阅读状态」；
   `library-api.md` 去掉 `tag=`、`tags[]`、`tags?`。**接口行为没变**：书列表的 `tag`、PATCH 的 `tags` 照旧被静默忽略。

### 前端要做什么

前端已在 `refactor/drop-document-tags` 改完（那个分支还没合），这里记下对照：

| 文件 | 旧写法 | 新写法 |
|---|---|---|
| `frontend/packages/api/src/documents.ts` | `DocumentListQuery.tag`，非空时 `params.set("tag", …)` | 去掉；`GET /documents` 只发 `limit`、`offset`、`reading_status`、`collection_id`、`q` |
| `frontend/web/src/features/library/domain/documents/update-document.ts` | `payload.tags !== undefined` 时把 `tags` 发给 PATCH 并回填卡片 | 去掉；`PATCH /documents/:id` 只发 `title`、`reading_status` |
| `frontend/web/src/features/library/ui/page/LibraryFilterMenu.tsx` | 按 `item.tags` 过滤（恒为空） | 去掉标签过滤 |

### 以后

前端和 AI 都不发以后（`refactor/drop-document-tags` 合进 main 之后），后端会给书列表的查询参数（`ListDocumentsQuery`）和
PATCH 的请求体（`PatchDocumentInput`）加 `deny_unknown_fields`：再发 `tag`、`tags` 或别的不认识的字段会 400。这一步这次不做。

---
## 000000000. 书架冻结：删单个任务改用 `DELETE /jobs/{job_id}`，书架列表改调 `GET /jobs?job_ids=`

分支 `refactor/library-books-freeze`（2026-10-19）。对应书库接口审计 A1，做法和取舍见
[书架冻结与删任务新地址](../../ops/planning/api-reorg/20261019-书架冻结与删任务新地址.md)，新接口见 [删除任务](../../api/01-任务/05-删除任务.md)。

### 后端变了什么（只增不改，前端现在不改也能用）

1. 新增 `DELETE /api/v1/jobs/{job_id}`（`?force=true` 可选）：删这个任务和它的 `{job_id}-ocr` 子任务，书留着。
   返回和书架删除一样（`deleted`、`job_id`、`removed_paths`、`removed_child_jobs`），409 / `job_has_dependents` / `force`
   的规则也一样；不同的只有：**OCR 任务也能删**（书架删除把它挡成 404），以及单删 OCR 子任务 `P-ocr` 而父任务 `P` 还在时
   409 + `job_has_dependents`。
2. `DELETE /api/v1/library/books/{job_id}` 变成旧地址（接口目录里标了），行为不变。
3. `GET /api/v1/library/books/{job_id}` 删了（前端、AI、CLI 都没调过）。同一地址还有 DELETE，所以 GET 返回 405。
4. `GET /api/v1/library/books` 冻结：不再加字段，前端改完下面第 1 条以后删。
5. 契约 `library-books.v1` 里 `LibraryBookDetailView` 定义暂留（标了废弃），因为前端还引用它（见下面第 4 条）。

### 前端要改的

1. `frontend/packages/api/src/library-books.ts` 的 `fetchLibraryBookList`（调用方：文档卡片叠加活态
   `features/library/domain/documents/shape-documents-with-books.ts:60`，文件夹 `features/collections/domain/controller.ts:67`）：
   - 旧：`GET library/books?limit=n&offset=0&job_ids=a,b`
   - 新：`GET jobs?job_ids=a,b&limit=n&include_live_stage=true`（`limit` 必须在 1..=500，越界 400；名单超过 500 个分批请求）。
     返回 `JobListView`：`items[]` 是任务列表项，标题用 `display_name`（没有 `title`），阶段和进度在 `items[].stage_snapshot`
     （`stage`、`stage_detail`、`progress`，已有的 `flattenStageSnapshot` 能摊平；终态任务的 `stage_snapshot` 是 `null`），
     `has_more` 表示还有没有下一页。`/jobs` 不排除 OCR 任务，`shape-documents-with-books.ts:70-85` 那段「没命中再逐个
     `GET /jobs/:id` 补」以后基本用不上（可以留着兜底）。
   - mock：`frontend/web/src/platform/mock/index.ts:90` 的 `synthesizeMockBook` 要改成按 `/jobs?job_ids=` 的形状造数据。
2. `frontend/packages/api/src/library-books.ts` 的 `deleteLibraryBook`（调用方 `features/library/domain/recent-jobs/actions.ts:75`，
   只在卡片没有 `document_id` 时走，见 `documents/delete-actions.ts:76`）：
   - 旧：`DELETE library/books/{id}`（`?force=true`）
   - 新：`DELETE jobs/{id}`（`?force=true`），返回形状和错误处理（`error.details.reason`）不变。
     `stripOcrSuffix` 可以留着：传 `-ocr` 子任务编号本来就该删它的父任务；要单删 OCR 任务时别去后缀。
3. 最近任务分页 `features/library/domain/recent-jobs/loader.ts:85`（`fetchLibraryBookList` 选项）→
   `recent-jobs/library-books-resource.ts:29` → `recent-jobs/pagination.ts:81`：没有 `fetchJobList` 时回退调书架列表，
   书架不给 `has_more`，只能靠「短页」猜。改成只用 `fetchJobList`（`GET /jobs`，回包有 `has_more`），去掉这条回退和
   `fetchLibraryBookList` 选项（`app/home/composition/create-runtime-features.ts:196` 已经同时传了 `fetchJobList`）。
4. 契约类型：`frontend/packages/api/src/types.ts:19` 重新导出的 `LibraryBookDetailView`，和
   `frontend/web/tests/contracts/library-books-contract.test.mjs:66` 那条「LibraryBookDetailView 字段存在于契约」的测试，删掉。
   删完告诉后端，后端再从 `contracts/library-books.v1.schema.json` 删这个定义和那条标了 `removed` 的端点。
5. 上面 1、3 都改完、不再调 `GET library/books` 以后告诉后端，后端删书架列表和 `DELETE library/books/{id}` 旧地址。

## 0000000. 还没产出：404 带 `error.details.reason = "not_ready"`，任务不存在是不带 reason 的 404

分支 `fix/not-ready-404`（2026-10-19）。对应审计 C3、D2，规矩见 [错误约定](../../api/00-约定/02-认证与错误.md#还没产出404)，
逐个接口的改前改后见 [实现记录](../../ops/planning/api-reorg/20261019-还没产出返回404实现记录.md)。

### 变了什么

1. 原始文件和报告（`/jobs/{id}/files/output|pages|internal/…`、`reader/markdown`、`reports/translation|translation-qa|layout-fit|refine`、
   `translations/{item}` 及修订历史 / 改写 / 重翻）：任务在、东西还没产出时 `404` + `error.details.reason = "not_ready"`，
   `message` 统一成 `<名字> not ready`（如 `translated.pdf not ready`，不再带任务编号）。原来就是 404，只是多了 `reason`、换了措辞。
2. 任务不存在：照旧 `404`，`error.details` 是 `{}`，`message` 是 `job not found: …`。
3. 状态码变了的只有三处：
   - `files/output/all.zip` 任务没成功时 `409 job is not finished successfully` → `404` not_ready `all.zip not ready`；
   - `GET /jobs/{id}/translations` 还没有翻译清单时 `404` → `200` 空列表；
   - `reports/quality`、`reports/quality/items`、`data`、`data/{dataset}` 在任务还没有输出目录（排队中）时 `404 job has no output directory` → `200`
     （各段 `null`、空清单、`available: false`）。
4. 阅读器会碰到的地址（`/jobs/{id}`、`files`、`reader/regions`、`reader/metadata`、`reader/markdown`、`files/output/ocr.md`、`ocr/images/…`、
   `translated.pdf`、`/documents/{id}/files/source.pdf`、`/documents/{id}/reading`、`progress/layout`、`progress/pages/{page}`、`ai/board`）
   状态码**都没变**；其中 `reader/markdown`、`ocr.md`、`ocr/images/…`、`translated.pdf` 没产出时多了 `reason: "not_ready"`。
   边翻边看的 `error.code`（`LIVE_TRANSLATION_LAYOUT_NOT_READY` 409、`LIVE_TRANSLATION_PAGE_NOT_COMMITTED` 404）没动。

### 前端要做什么

- `frontend/packages/reader/src/hooks/reader-session/session-load-plan.ts` 的 `isStaleDocumentJobFailure`：
  旧写法 `input.status === 404 && !input.jobId && …`，正在跑的书拿到译文 PDF 的 404 也会被当成"任务丢了"退回原文；
  新写法加一条 `&& input.reason !== "not_ready"`，`session-assets.ts`（约 458 行）调用时把错误体里的 `error.details.reason` 传进去
  （抛出的错误要带上响应体里的 `reason`）。不改也不会比现在差：这些地址原来就是 404。
- `frontend/packages/api/src/translation-debug.ts`（可选）：`fetchTranslationDiagnostics`、`fetchTranslationItem` 现在 404 一律说
  「未找到…请确认已完成翻译」；可以按 `reason === "not_ready"` 说「还没生成」，没有 `reason` 说「任务不存在」。
  `fetchTranslationItems` 的 `404 → 空列表` 分支以后只在任务不存在时走到，不改也对。
- `all.zip`：前端没有按 409 判断，不用改。质量卡、取数在排队任务上从报错变成空状态，不用改。
- 判断"还没好"只看 `error.details.reason`，不要匹配英文 `message`（措辞已经变了：`pdf not ready: <id>` → `translated.pdf not ready`、
  `markdown not found: <id>` → `markdown not ready`、`refine report not found: <id>` → `refine report not ready`……）。

---

## 000000. 删除要顾到别的任务：强删先取消，被引用的任务不能直接删，删任务清全文索引

分支 `fix/force-delete-stops-runner`（2026-10-19）。对应状态一致性审计 1-#5、2-#1/#2/#4。

### 变了什么

1. `DELETE /library/books/{job_id}?force=true` 和 `DELETE /documents/{document_id}?force=true`：删之前，对其中排队 / 运行中的任务
   先走和 `POST /jobs/{job_id}/actions/cancel` 同一条路径（登记取消 + 终止 worker 进程），再删行删目录。以前强删既不取消也不杀进程，
   worker 继续花 OCR / 模型的钱，runner 的下一次落库还把已删的任务写回来（没有 `document_id` 的幽灵任务）。
   现在 runner 的落库写（CAS）在行不存在时不再 INSERT，runner 见到行没了会终止 worker 并收手。响应形状不变。
2. **新的 409 原因 `job_has_dependents`**：`DELETE /library/books/{job_id}` 删某个任务时，如果删除范围（该任务 + 它的 `-ocr` 子任务）之外
   还有别的任务在用它的产物（`request_payload.source.artifact_job_id` 指向它，典型是「换字体重新排版」产生的 render 任务，
   译文目录就是源任务的 `translated/`），返回 `409` + `error.details.reason = "job_has_dependents"`，message 里列出依赖者的编号，什么都不删。
   以前照删，之后整本书的 `/documents/{id}/reading` 因为找不到译文目录返回 500。带 `?force=true` 仍然放行（调用方明确要删）；
   `DELETE /documents/{id}` 整本删除时依赖者本来就在删除范围内，不受影响。原地重新渲染的任务引用自己，不算依赖。
3. 删除任务时同时清掉它在全文索引里的行（`blocks_fts`），再按这本书剩下的成功任务重建索引；搜索不再命中已删的任务。

### 前端要做什么

**不用改代码**（前端没有传 `force`；删书走的是 `DELETE /documents/{id}`，不会撞到 `job_has_dependents`）。
如果以后用 `DELETE /library/books/{job_id}` 删单个任务：遇到 `reason === "job_has_dependents"` 提示「先删除依赖它的任务（message 里有编号）」，
不要自动加 `force`。

---

## 00000. 请求体里不认识的字段一律 400

分支 `fix/unknown-fields-400`（2026-10-19）。所有用 `ApiJson` 读、目标类型 `deny_unknown_fields` 的请求体（邀请码、agent 会话、
译文修订 PATCH、retry-stage、resolve-ocr-ambiguity……），遇到不认识的字段从 `422 UNPROCESSABLE_ENTITY` 改成 `400 BAD_REQUEST`，
message 形如 ``unknown field `x`, expected one of ... in the request body``（只带字段名，不带值）。字段类型不对、缺字段、
枚举值不认识仍是 `422`。前端核对过没有地方按 `422` 判断，不用改。

## 0000. 任务操作被拒：状态不允许一律 409，带 `error.details.reason`

分支 `fix/actions-409-reasons`（2026-10-19）。对应审计 B1、B8。规矩见 [错误约定](../../api/00-约定/02-认证与错误.md)。

### 变了什么

1. **状态码**：任务或产物的当前状态不允许 = `409`；请求本身写错了 = `400`。
   以前 `retry-stage`（ocr / translation / render，含原地重渲染）和 `resume`/`rerun` 在「任务在跑」「产物不在」时返回 `400`，
   只有精修和取消返回 `409`。现在这些全是 `409`。仍是 `400` 的：`mode` 不支持、`refine` 对象放在非 refine 阶段、
   `create_new_job=false` 用在 ocr / translation、`create_new_job=true` 用在 refine、`overrides` 不合法、请求体里有不认识的字段。
2. **稳定原因**：每个任务操作的 `409` 都有 `error.details.reason`（信封其他部分不变）：

   | `reason` | 什么时候 | 接口 |
   |---|---|---|
   | `job_active` | 任务在排队/运行，先取消 | retry-stage（所有阶段）、resume/rerun、resolve-ocr-ambiguity |
   | `job_not_active` | 取消一个不在跑的任务 | cancel |
   | `artifacts_missing` | 要用的产物或原始来源不在 | retry-stage、resume/rerun |
   | `translations_not_owned` | 精修的任务渲染的是别的任务的译文 | retry-stage refine |
   | `ambiguous_ocr_dispatch` | OCR 派发结果不确定：走 resolve-ocr-ambiguity（绑定回执或接受重复风险），不要引导去重试翻译 | retry-stage（ocr）、resume/rerun |
   | `ambiguous_translation_request` | 翻译请求结果不确定：重复风险确认后 retry-stage(translation) + accept_duplicate_risk | retry-stage（translation）、resume/rerun |
   | `request_journal_corrupt` | 翻译请求日志损坏，确认也修不好 | retry-stage translation、resume/rerun |
   | `model_executor_unsupported` | 绑定了 Rust 模型执行器，这条旧路径不支持 | retry-stage、resume/rerun |
   | `stale_resolution` | OCR 歧义的 `resolution_revision` 过期，或被别的请求处理了 | resolve-ocr-ambiguity |
   | `no_ocr_dispatch` | 没有可处理的 OCR 歧义（既不是失败也不是已取消，或没有派发记录） | resolve-ocr-ambiguity |
   | `ocr_dispatch_unsupported` | 该服务商/操作不支持绑定收据 | resolve-ocr-ambiguity |

   `reason` 只增不改，不认识的值按「状态不允许，看 `message`」处理。
3. **GET 里同一套值**（只增字段）：`GET /jobs/:id/actions` 每个阶段多 `disabled_code`（`disabled_reason` 为空时是 `null`；
   翻译阶段在请求结果不确定时 `can_retry` 仍为 true、`disabled_code` 是 `ambiguous_translation_request`）；
   `GET /jobs/:id/actions/resume` 多 `reason_code`（能续跑时 `null`）。
4. **`retry-stage` 和 `resolve-ocr-ambiguity` 的请求体不认识的字段返回 `400`**（message 形如 ``unknown field `createNewJob`, expected one of ...``）。
   后端自己在 `GET actions` 里给的 `action.body` 都在可用字段内（有测试锁）。可用字段：`stage`、`mode`、`create_new_job`、`overrides`、
   `ambiguous_request_policy`、`refine`。
5. 翻译请求日志损坏（`request_journal_corrupt`）时，`GET actions` 里 translation 的 `can_retry` 现在是 `false`（以前是 `true`，点了必 409）。

### 前端要做什么

- `frontend/web/src/features/job-detail/domain/dialog/resume-actions.ts` 的 `isAmbiguousTranslationRerunError` 和
  `page/resume.ts` 里调用它的地方：现在靠正则匹配 `409` 和英文 `translation request outcome is ambiguous` 判断。
  改成读错误体：`error.details.reason === "ambiguous_translation_request"`（需要把 `ApiError` 里的响应体带出来）。
  OCR 的歧义是另一个值 `ambiguous_ocr_dispatch`，处理办法是 resolve-ocr-ambiguity，不能走翻译的重复风险确认。损坏的日志是 `request_journal_corrupt`，**不要**再给「接受重复风险」的入口。
  这两处不改也能用：消息文字没变，旧写法仍然匹配；改了以后不再依赖英文措辞。
- 凡是判断「任务在跑」的，用 `reason === "job_active"`，不要看 `message`。
- OCR 歧义现在在默认的 book/翻译任务上也认得出来（以前 OCR 派发记录挂在 `<id>-ocr` 子任务上，按任务编号查不到）：
  `GET /jobs/{id}/reports/failure` 的 `ocr_ambiguity` 会对这类失败任务给出值，`retry-stage`（ocr）和 `rerun` 会返回
  `409 reason=ambiguous_ocr_dispatch`，`resolve-ocr-ambiguity` 对它们可用。提交出错（超时、断连）和上传中途取消、
  进程在回执落库前重启，都会让派发记录变成 ambiguous。前端不用改；已经按 `ocr_ambiguity` 出处理入口的页面，会在这些任务上多出现该入口。
- `retryJobStage` 之类别往请求体里夹 `retry-stage` 不认识的字段（书籍元数据等），否则 `400`。后端侧已核对过目前发的只有
  `stage`、`mode`、`create_new_job`、`overrides`、`ambiguous_request_policy`、`refine`。
- 以前依赖「在跑 / 产物不在返回 400」的分支改看 `409`。

---

## 000. 续跑的正名是 `actions/resume`，任务视图加了 `actions.resume`

分支 `refactor/actions-resume-canonical`（2026-10-19）。只增不改。

- `POST /api/v1/jobs/:job_id/actions/rerun` 和 `actions/resume` 一直是同一个处理函数，现在在登记表里标成旧地址，行为不变，旧地址照常能用。
- 任务视图的 `actions` 加了 `resume`：`{enabled, method: "POST", path: ".../actions/resume", url}`，`enabled` 和 `rerun` 相同。`actions.rerun` 先留着、同值。
- 契约：`contracts/job-status.v1.schema.json` 的 `JobActionsView` 加了必有的 `resume`。
- 前端要改的：把读 `actions.rerun.url`、`actions.rerun.enabled`、调 `rerunJob` 的地方改成 `actions.resume`。等前端不再用 `rerun` 以后，后端删掉旧地址和 `actions.rerun`。

## 00. 列表项的 `failure` 进了契约：job-failure-brief 契约测试可以改比 schema

分支 `feat/contract-job-failure-brief`（2026-10-18）。JSON 不变，只是把一直存在、但没人描述过的 `failure` 补进契约。

- 定义：`contracts/library-books.v1.schema.json#/definitions/JobFailureBriefView`；`job-status.v1.schema.json` 里有结构相同的一份（两份由 `contracts/tests/job-definitions-parity.test.mjs` 锁一致）。
- `JobListItemView.failure` 是**可选**属性（只有失败的任务才有，成功、进行中的整个键不出现，不是 null）。
  `JobFailureBriefView` 的 `category`、`stage`、`retryable`、`summary` 必有；`root_cause`、`suggestion`、`provider` 没有时键也不出现。
- 生成的 TS 类型已更新：`@retainpdf/contracts/library-books` 和 `/job-status` 都导出 `JobFailureBriefView`，`JobListItemView.failure?: JobFailureBriefView`。
- 前端要改的：`frontend/web/tests/contracts/job-failure-brief-contract.test.mjs` 现在读 `backend/packages/retainpdf-jobs/src/views/job_types.rs`
  抠字段；改成对照上面这个 definition 的 `properties` / `required`，不再读 Rust 源码（这条由前端会话自己改，后端不动 `frontend/`）。
  `frontend/packages/domain` 里手写的失败简报类型也可以换成生成的 `JobFailureBriefView`。
- 后端侧有测试锁：`backend/api/src/api_tests/job_view_contract.rs` 的 `failed_job_list_item_failure_matches_published_schema`
  用真实路由取失败任务的列表项，`failure` 的键集合必须等于这个 definition。

## 0. 三阶段（OCR / 翻译 / 渲染）展示：数据够用了，但要读对地方

后端提交：`364aaaf9` `b575906e` `c85ff4af` `fce89bcc`

### 必须改代码的只有一处

`JobStageTiming` 新增两个 **required** 字段,生成的 DTO 已同步
(`@retainpdf/contracts` → `contracts/dist/job-status.d.ts`):

```ts
export interface JobStageTiming {
  stage: string; detail: string | null;
  enter_at: string; exit_at: string | null;
  duration_ms: number | null;
  terminal_status: JobStatusKind | null;
  progress_current: number | null;   // ← 新增，required
  progress_total: number | null;     // ← 新增，required
}
```

只读不构造的地方不受影响。**手工构造完整对象的地方**(测试 fixture、mock 数据
最可能)TypeScript 会报缺字段,填 `null` 即可。

同理 `RuntimeInput` 早先加过 `no_output_timeout_seconds`(也是 required,填 `0`)。

### 各读哪里

| 要显示什么 | 读哪里 |
|---|---|
| 现在在哪一段、进行到哪 | `stage_snapshot`(带 `stage` + `substage`)+ `stages.*.state` |
| 实时推送 | `GET /api/v1/jobs/:id/live-events`(SSE 长连接) |
| 细阶段流水 | `GET /api/v1/jobs/:id/events`，按 `display_stage` 分组、过滤 `lane=main` |
| **完成后回顾各段耗时与进度** | `runtime.stage_history`(新的 `progress_current/total`) |

**不要用 `stages.*.progress` 做完成后的回顾**——它只反映当前活跃阶段的瞬时
快照,阶段一结束就回到 null。这是有意为之:`stages` 回答「现在怎么样」,
`stage_history` 回答「当时怎么样」,两个问题两个地方。

另一个做法(终态回填 `stages.*.progress`)被否掉了:那样失败任务会显示
「translation: 30/50」,看上去像还在跑、实际是死在了 60% 处,前端必须同时读
`state` 才能判断这个数是活的还是死的。

### 事件长这样（字段是为三段展示设计的）

```jsonc
{ "display_stage": "ocr",              // 三段归属：ocr / translation / render
  "stage": "ocr_result_ready",         // 细阶段（全流程 15 个）
  "substage": "done",
  "lane": "main",                      // main / diagnostic / background
  "stage_detail": "Paddle 正在解析文件，第 4/4 页",   // 中文，可直接展示
  "provider": "paddle", "provider_stage": "done",
  "progress": { "unit": "step", "current": 4, "total": 4, "percent": 100.0 },
  "elapsed_ms": 0,
  "raw": { "source_kind": "ocr_child" } }              // 子任务事件已并入父流
```

三段各自的细阶段:

| 段 | 细阶段 |
|---|---|
| OCR | `ocr_submitting` → `ocr_upload` → `ocr_processing` → `ocr_result_ready` → `normalizing` |
| 翻译 | `translating` → `translate` → `continuation_review` → `page_policies` → `garbled_repair` |
| 渲染 | `rendering` → `render_prepare` → `render_preprocess` → `render` |

### 不用改代码，但行为变对了

- **`stage_history` 顺序修正**。以前是
  `translating → finished → rendering → failed`——「完成」夹在「渲染中」前面,
  没法按顺序画时间线。现在是 `translating → rendering → finished`。
  若之前为这个乱序写过绕过逻辑,可以删了。
- **进度文案不再自相矛盾**。以前会出现「进度 100% · 正在开始翻译」
  (数字来自 durable checkpoint、文字是阶段入场语),现在两者同源。
- **渲染阶段不再顶着翻译的进度**。归档刚上线时 rendering 和 finished 会带着
  翻译的 `59/59`(那是文本块数,不是页数),已修。

### 一个仍然存在的不一致

`stages.render.progress` 在任务结束后常残留一个 `2/3`,而 ocr / translation
是 null。那是 `background_snapshots` 里的残影,不是渲染的真实进度。**画完成后
的回顾一律走 `stage_history`**,就不会碰到它。

---

## 0.5 阅读视图分段读 markdown：用 HTTP Range，不新增端点

后端提交：`perf(api): markdown 原文改走文件流，直接支持 Range`

### 结论先说

你提案里的 `/markdown/metadata` 和 `/markdown/chunks` 两个端点**都不做**，
改用已有的 `GET /api/v1/jobs/:job_id/markdown?raw=true` + 标准 HTTP `Range`。

后端 API 已经 113 个,markdown 一个资源就占 3 条。加两条自定义端点去做一件
HTTP 本来就定义了的事,不划算。

### 为什么 metadata 端点不做

它的每一个字段,job detail 的 `artifacts.markdown` 里都已经有了:

| 提案的 `MarkdownMetadataView` | 已有的位置 |
|---|---|
| `ready` | `artifacts.markdown.ready` |
| `total_bytes` | `artifacts.markdown.size_bytes` |
| `raw_path` / `raw_url` | `artifacts.markdown.raw_path` / `raw_url` |
| `images_base_path` / `images_base_url` | 同名字段 |
| `chunks_path` | 不再需要 |

而且 `Content-Range: bytes 0-262143/620000` 的分母就是总字节数,第一次请求
就拿到了,连预先查一次都不用。

### 为什么 chunks 端点不做

`stream_file` 早就实现了完整的 Range(解析头、`206`、`Content-Range`、
`Accept-Ranges`、seek + 流式读),只是 markdown 这条路没接上——它原先把整篇
`read_to_string` 进堆再整个返回。现在接上了,顺带修掉了那次整篇读入
(实测最大的 `full.md` 是 620 KB)。

你否掉 Range 的理由是「按字节切会切到行/多字节字符中间,前端拼行更麻烦」。
多字节那半在浏览器里不成立:`TextDecoder(..., { stream: true })` 就是为跨块
序列设计的,是内置能力。行边界那半确实要前端做,但只有几行。

### 前端要怎么用

```js
const res = await fetch(rawUrl, {
  headers: { ...apiHeaders, Range: `bytes=${offset}-${offset + WINDOW - 1}` },
});
// 206；Content-Range: bytes <start>-<end>/<total>
const total = Number(res.headers.get("Content-Range").split("/")[1]);
const etag = res.headers.get("ETag");
```

两件必须自己做的事:

1. **字符边界**：整段读取过程共用**一个** `TextDecoder`,每次
   `decoder.decode(bytes, { stream: true })`。**不要**每段单独解码——那会在
   跨块的多字节字符上出乱码。
2. **行边界**：一段可能停在行中间,会把代码块 / 表格行 / 公式劈开。把最后一个
   `\n` 之后的残余留下,拼到下一段开头再交给渲染。

**版本检测**：比对各段的 `ETag`(size + mtime)。正文由 pipeline 一次性整篇写出,
当前没有任何路径会就地改写它(非 render 的 rerun/retry 都建新 job;render 就地
重跑只动 `rendered/` 里的译文 PDF),所以偏移量在实践中是稳定的——但那是当下 job 生命周期的
性质,不是这个端点给的保证。`ETag` 中途变了就从 0 重来,否则拼出来的是两个版本
的混合,而且是静默的。也可以直接用 `If-Range`。

### 几点提醒

- 图片仍是正文里的原始相对路径(`images/...`),用 `images_base_url` 解析。
  **复用 `frontend/packages/domain/src/job/artifacts.ts` 的
  `resolveMarkdownAssetUrl`,别在新数据层里重写一遍**——它第 281 行那个
  `while (rel.startsWith("images/"))` 是在剥双重前缀(`images_base_url` 已经
  以 `/images/` 结尾,而正文里写的是 `images/x.png`),漏掉就是 404。
- 不带 `Range` 时行为不变:`200` + 整篇,旧调用方不受影响。
- `/markdown/document` 保留。但阅读器迁走后它只剩 job-detail 页在用,而它贵在
  两处:整篇正文返回两份(原文 + 图片链接重写版),外加 `walkdir` 遍历整个 images
  目录逐个 stat + mime_guess。**如果 job-detail 那边也能改用 `?raw=true`,这条
  端点就可以删掉**,markdown 从 3 条端点收敛到 2 条。要不要做由你定,后端这边
  随时可以配合。

---

## 1. 收藏挡住删除 → 结构化 409 + 清空收藏端点

> 已撤销：收藏功能 2026-09 整体删除，`DELETE_BLOCKED_BY_FAVORITES` 和清空收藏的端点后端都已删掉（`chore/drop-favorites-errors`），
> 前端处理它的代码、批量删除的「一并删除收藏」二次确认也已删（`refactor/drop-favorites-leftovers`）。删除现在只在还有排队或运行中的任务时返回 409，见
> [图书馆数据层 API](library-api.md) 的「删除保护」。下面保留原记录。

后端提交：`feat(library): 收藏挡住删除时返回结构化 409，并给出清空收藏的端点`

### 变了什么

删除文档 (`DELETE /api/v1/documents/:document_id`) 或删除馆藏图书
(`DELETE /api/v1/library/books/:job_id`) 被收藏锚点挡住时，409 的响应体从
「一句中文 message」变成了带 `error.code` 和 `error.details` 的结构化错误。

同时新增两个端点，用来一次性清空收藏。

### 前端要做什么

**别再从 message 里正则抠数字。** 那句话会随文案改动和多语言而变。

```jsonc
{
  "code": "DELETE_BLOCKED_BY_FAVORITES",
  "message": "document is referenced by 2 favorite(s); remove the favorites first",
  "error": {
    "code": "DELETE_BLOCKED_BY_FAVORITES",
    "http_status": 409,
    "details": {
      "scope": "document",              // 或 "job"
      "document_id": "6f1c…",           // scope=job 时是 job_id
      "favorite_count": 2,
      "clear_favorites_path": "/api/v1/documents/6f1c…/favorites"
    }
  }
}
```

文档级和 run 级**共用同一个错误码**，因为前端要做的事一模一样，只有清空目标
不同，而那个目标已经在 `clear_favorites_path` 里给好了。所以只需要一个分支：

> 读 `favorite_count` → 提示用户「该文档有 N 条收藏，是否一并删除？」
> → 确认后 `DELETE clear_favorites_path` → 重试原来的删除

`DELETE clear_favorites_path` 返回 `{"data": {"deleted_count": N}}`。

几个边界：

- 幂等：目标存在但没有收藏时返回 `deleted_count: 0`，不是错误
- 目标不存在是 **404**，不要和上一条混为一谈
- `force=true` **不绕过**收藏保护，它只绕过「运行中的任务不可删」
- run 级的清空会连 `{job_id}-ocr` 子任务的收藏一起清（因为它解锁的那次删除
  会连着删子任务），所以清一次就够，不会出现「重试又 409」

详见 [图书馆数据层 API](./library-api.md#两个必须处理的边界)。

---

## 2. 新增 `no_output_timeout_seconds`（空闲超时）

后端提交：`fix(jobs): 给 worker 输出收尾加上限…` 之后的空闲超时提交

### 变了什么

任务提交的 `runtime` 块多了一个字段：

```jsonc
"runtime": {
  "timeout_seconds": 1800,
  "no_output_timeout_seconds": 0    // 新增，0 = 关闭（默认）
}
```

`timeout_seconds` 是整段执行的上限，必须按最坏情况给——一本大部头翻译几小时
是正常的，阈值就得设到几小时。于是「打完第一行就卡住不动」这种情况也要等满
那几小时才被发现。

`no_output_timeout_seconds` 独立地盯「还在不在动」：每收到一行 stdout 就重新
计时，只有 worker 彻底不出声才触发。

两者都判为 `failed` / `process_timeout`（处置方式一样），但 `stage_detail` 不同：

| 触发者 | `stage_detail` |
|---|---|
| `timeout_seconds` | `provider timeout`（normalizing 阶段是 `normalization timeout`） |
| `no_output_timeout_seconds` | `no output for {N}s` |

**这个区分是给用户看的**：前者要调阈值或降并发，后者要查上游是不是不回包了。
如果前端有失败原因的展示或文案映射，`no output for {N}s` 需要一条对应的说法。

校验：负数会被 400 拒绝（`no_output_timeout_seconds must be zero (disabled) or
a positive integer`）；`0` 和正数都放行。

### 前端要做什么

**默认关闭，所以不改也不会坏。** 想让用户能设它的话，要改两处字段白名单——
它们是手写的列表，不是从 `@retainpdf/contracts` 的类型生成的：

- `frontend/packages/api/src/jobs-submit.ts` 的 `legacyTopLevelFields`
  （在 `"timeout_seconds"` 旁边加 `"no_output_timeout_seconds"`）
- `frontend/web/src/platform/api/legacy/jobs-submit.ts`：第 33 行附近的字段
  数组，以及第 101 行附近 `appendFormField(form, "timeout_seconds", …)` 那句
  旁边补一句同形的

（原来 multipart 的 `/api/v1/translate/bundle` 也接受同名的扁平字段，该接口已于 2026-10-19 删除，
只剩 JSON 的 `runtime` 块。）

生成的 DTO 类型 `@retainpdf/contracts` 的 `RuntimeInput` 已经带上这个字段
（`contracts/dist/job-status.d.ts`），是 **required**——如果哪里在手工构造
完整的 `RuntimeInput` 对象，TypeScript 会要求补上，填 `0` 即可。

详见 [任务提交 API](../../../backend/api/docs/api-spec/jobs-submission.md) 的
Timeout note 一节。

---

## 3. 取消语义：normalizing 不再豁免

后端提交：`fix(jobs): 取消一律终止进程，normalizing 不再豁免`

### 变了什么

此前 OCR 任务在 `normalizing` 阶段点取消，后端会**跳过杀进程**，让它跑完。
结果是用户点了取消，任务却继续显示 running 直到自然结束。现在取消一律终止进程。

### 前端要做什么

**不用改代码**，但有一个时序需要知道：

OCR 专用的取消 (`POST /api/v1/ocr/jobs/{job_id}/cancel`) 只在任务还是
`queued` 时由它自己写终态行。过了这个点，它杀进程，终态由 runner 观察到进程
退出后写入。所以**取消后立刻轮询，可能还会读到一次 `running`**。

如果现在的实现是「取消成功 → 立即认定已取消 → 停止轮询」，那会显示不一致；
应该继续轮到状态真的变成 `canceled`。这个时序在改动前后都存在（改动前更糟：
它可能永远不变），只是现在窗口很短。

详见 [任务重试与控制](../../../backend/api/docs/api-spec/jobs-retry-and-control.md#cancel-job)。

---

## 重启后翻译自动续跑先看请求日志（fix/translation-resume-journal-gate）

后端提交：`fix(runner): 重启后翻译自动续跑先看请求 journal，结果不明就落 failed`

### 变了什么

API / jobsd 重启时，翻译阶段运行中的任务以前一律自动续跑，续跑的 Python 会把重启前在途、结果不明的请求原样再发一遍（可能重复计费），而手动重试要用户确认。
现在：请求日志里有「已发出、没有终态」的请求（或日志读不了）时，任务不再自动续跑，落成 `failed`，`failure_code = translation_request_ambiguous`，
`translation_request_recovery.requires_confirmation = true`。日志干净或没有日志时仍自动续跑。

### 前端要做什么

**不用改代码。** 失败任务走的就是已有的歧义路径：`GET actions` 里 translation 的 `can_retry=true`、`disabled_code=ambiguous_translation_request`，
`retry-stage` 不带 `accept_duplicate_risk` 返回 409 `ambiguous_translation_request`。`failure_code` 是新增的取值，前端按「不认识的值显示 `summary`」处理即可。

---

## 数据库恢复后对账、自动续跑次数上限（fix/recover-after-restore-and-crash-loop）

### 变了什么

1. **数据库恢复**（`POST /api/v1/backups/{id}/restore`）：备份时在排队 / 运行的任务，以前恢复出来一直是 running（没人在跑，一直转圈、rerun 409）。
   现在恢复时清掉它们的旧 pid，并立即按重启时的规则对账：有断点的回到 `queued` 自动续跑，没有的落 `failed`（`failure_code = worker_process_missing`）。
   任务跑在 jobsd 里时由 jobsd 的巡检接手，最多约 2 分半。接口的请求、返回都没变。
2. **自动续跑次数上限**：同一次运行连续 3 次自动续跑都没有新进度就又中断，第 4 次不再续跑，任务落 `failed`，
   `failure_code = auto_resume_limit_exceeded`（`failure_category=internal`、`retryable=true`、`resume_from=null`），
   `summary`「自动续跑次数超过上限，疑似导致运行时崩溃」，`recovery_hint` 提示先看日志再手动重试。

### 前端要做什么

**不用改代码。** 恢复后本来就要整页刷新，任务列表读到的就是对账后的状态。`auto_resume_limit_exceeded` 是新增的 `failure_code` 取值，
现有的失败弹窗按 `summary` / `recovery_hint` 显示（`failure-recovery.ts` 优先用后端的 `recovery_hint`），重试按钮照 `retryable` 和 `GET actions`。

详见 [失败结构](../../api/08-调试诊断/01-失败结构.md)、[数据库备份](../../api/10-备份/01-数据库备份.md)。
