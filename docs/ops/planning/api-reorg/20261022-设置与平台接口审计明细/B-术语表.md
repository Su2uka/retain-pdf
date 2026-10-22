# 术语表

审计范围：`/api/v1/glossaries` 下全部 8 个接口（列表、新建、单个读/改/删、`export.csv`、`import`、`parse-csv`），路由 `backend/api/src/app/router/glossaries.rs`，处理函数 `backend/api/src/routes/glossaries.rs`，领域包 `backend/packages/retainpdf-glossaries`，库层 `database/retainpdf-db/src/db/glossaries.rs`；顺带看了建任务时怎么用术语表（`resolve_task_glossary_request`）和重跑、重试怎么带上它。
只读代码（main，`35043e3f`）得出，没跑测试、没调用运行中的服务。标「未实跑」的是读代码推断的结论。

## 接口一览

| 方法 地址 | 处理函数 | 返回类型 | 契约 | 文档 | 调用方（前端文件:行） | 备注 |
|---|---|---|---|---|---|---|
| GET `/api/v1/glossaries` | `glossaries::list_glossaries_route` → `api.rs::list_glossaries_view` | `GlossaryListView { items: [GlossarySummaryView] }` | 无 | `backend/api/docs/api-spec/glossaries.md:18-48`、`docs/core/api/index.md:748`、`docs/api/04-翻译/03-术语表.md:28` | `frontend/packages/api/src/glossaries.ts:6 fetchGlossaries`（管理框 `features/glossaries/domain/controller.ts:110`、建任务下拉 `features/ingest/domain/workflow/glossary-options.ts:47`）；`retainpdf-agent glossary list`（`backend/api/src/bin/retainpdf-agent.rs:374`，AI `agent_translation_runner.py:391` 按名字找「本书术语」表） | 不分页；查询参数 `enabled/source_lang/target_lang/q` 前端都不发 |
| POST `/api/v1/glossaries` | `create_glossary_route` → `create_glossary_view` | `GlossaryDetailView` | 无 | 同上 | `glossaries.ts:20 createGlossary`（`controller.ts:193`）；agent `glossary create`（`retainpdf-agent.rs:387`，AI `agent_translation_runner.py:341`） | |
| GET `/api/v1/glossaries/:glossary_id` | `get_glossary_route` → `get_glossary_view` | `GlossaryDetailView` | 无 | 同上 | `glossaries.ts:12 fetchGlossary`（`controller.ts:137`）；agent `glossary get`（`retainpdf-agent.rs:380`，AI `agent_translation_runner.py:383,398`） | |
| PUT `/api/v1/glossaries/:glossary_id` | `update_glossary_route` → `update_glossary_view` | `GlossaryDetailView` | 无 | 同上；`docs/core/api/index.md:824-830`（「按整个 `entries` 数组替换」） | `glossaries.ts:24 updateGlossary`（`controller.ts:192`）；agent `glossary update`（`retainpdf-agent.rs:394`，AI `agent_translation_runner.py:335` 的 `term-set`） | 整份替换，见第 2、6 条 |
| DELETE `/api/v1/glossaries/:glossary_id` | `delete_glossary_route` → `delete_glossary_view` | `GlossaryDetailView`（删掉的那份，含全部词条） | 无 | 同上 | `glossaries.ts:39 deleteGlossary`（`controller.ts:213`） | 不查任务引用 |
| GET `/api/v1/glossaries/:glossary_id/export.csv` | `export_glossary_csv_route` → `export_glossary_csv_view` | `text/csv; charset=utf-8` 原文（不是信封） | 无 | `api-spec/glossaries.md:60-64` | `glossaries.ts:50 exportGlossaryCsv`（`controller.ts:239`，走 `downloadProtectedResponse`） | 没有 `Content-Disposition`，见第 8 条 |
| POST `/api/v1/glossaries/import` | `import_glossary_route` → `import_glossary_view` | `GlossaryDetailView` | 无 | `api-spec/glossaries.md:73-78`（写对了：JSON）；登记表和 `docs/api/00-接口目录.md:196` 写「从 CSV 导入」（错） | **没有调用方**（前端、`backend/ai`、`backend/cli`、`frontend/desktop`、脚本都没有；只有 `api_tests/glossaries.rs:63` 和包内 `tests.rs:229`） | 等于 POST + PUT，见第 5 条 |
| POST `/api/v1/glossaries/parse-csv` | `parse_glossary_csv_route` → `parse_glossary_csv_view` | `GlossaryCsvParseView { entry_count, entries }` | 无 | `api-spec/glossaries.md:50-58`、`docs/core/api/index.md:832-850` | `glossaries.ts:61 parseGlossaryCsv`（`controller.ts:266`，管理框里「粘贴 CSV → 解析」） | 不碰库；和 import 不重复 |

`docs/api/00-接口目录.md:195-202`、`backend/api/docs/routes.txt:86-93` 与登记表一致（同源生成），全部 `signed`。三份快照里只有 `submissions.txt` 带术语表字段（任务请求里的 `glossary_*`），`file-responses.txt` 没有 `export.csv`。

## 问题

### 1. 重跑、重试时把上一次的术语快照当成「内联术语」再并一次：表改了用旧译法，表删了直接 404
- 位置: `backend/packages/retainpdf-glossaries/src/entries.rs:13-44`（`resolve_task_glossary_request`：读表、把请求里的 `glossary_entries` 当内联条目覆盖在表上，结果写回 `glossary_entries`）；`backend/packages/retainpdf-jobs/src/lifecycle/creation/prepare.rs:41,65`（`book` / `translate` 都先过它）；`backend/packages/retainpdf-jobs/src/actions/rerun.rs:230-256`（`build_rerun_request` 原样复制 `translation`）、`actions/stage_retry_request.rs:24`（同样复制）；`records.rs:108-111`（`load_glossary_or_404`）；前端建任务只发 `glossary_id` 和空的 `glossary_entries`（`features/ingest/domain/workflow/payload.ts:162-163`）；AI `term-set` 的提示（`backend/ai/retainpdf_ai/agent_translation_runner.py:368-372`，「术语表只影响以后用它的翻译」）。
- 现状:
  - 建任务时任务请求里存的是**快照**：`glossary_id`、`glossary_name` 加上合并后的全部 `glossary_entries`，`glossary_inline_entry_count` 等计数也在。翻译只读快照，所以任务建好以后改表、删表都不影响这次翻译——这一点是对的。
  - 但续跑（`POST …/rerun` 出新任务）、阶段重试（`retry-stage`，出新任务的那几种）都把源任务的 `translation` 原样复制进新请求再走 `submit_job` → `resolve_task_glossary_request`。这时 `glossary_entries` 里装的是上次的整份快照，被当成「内联条目」覆盖在**当前**的表上（未实跑）：
    - 表删了：`load_glossary_or_404` 报 404 `glossary not found`，重跑整个失败。前端在重跑按钮上看到的是 404，不知道是术语表的问题。
    - 表改了某个词的译法（包括 AI 终端 `term-set` 刚改的）：快照里的旧译法在覆盖层，同一个 `source` 以覆盖层为准，**新译法不生效**；表里删掉的词也会从快照里回来。`term-set` 的提示「术语表影响以后的翻译」对这本书的重译恰好不成立。
    - 计数失真：`glossary_inline_entry_count` 变成上次快照的条数，`glossary_overridden_entry_count` 变成和表重合的条数，任务详情里的术语表摘要（`GlossaryUsageSummaryView`）就不对了。
    - 表加了词，合并后超过 200 条时重跑 400 `merged glossary entry count exceeds 200`。
- 问题: 「快照」和「用户内联」存在同一个字段里，下一次解析分不清，重跑时的语义既不是「用当时的快照」也不是「用现在的表」。
- 建议: 后端定一个语义并写进 `docs/api/04-翻译/03-术语表.md`。推荐「重跑用现在的表，内联条目照旧」：
  - 请求里另存用户真正传的内联条目（只增：`translation.glossary_inline_entries`，建任务时由 `resolve_task_glossary_request` 写入；老任务没有这个字段时，用 `glossary_inline_entry_count == 0` 判断「没有内联」，否则退回把快照当内联的旧行为）；重跑、重试时 `glossary_entries` 先换回 `glossary_inline_entries` 再解析。
  - 表已经删了：重跑改成 409 `conflict_because("glossary_missing", …)` 并在 message 里说清，或者退回用快照并在提交结果里给一句提示——二选一，倾向前者（用户明确删了表）。
  - 加测试：表改了某词译法后重跑，新任务的 `glossary_entries` 用新译法；表删了重跑的状态码。
  - 对前端的影响：没有字段变化；重跑在表被删时的状态码从 404 变成 409（前端重跑失败只显示 message，`retry` 相关的提示文案可以按 `reason === "glossary_missing"` 给「术语表已删除，请改选后重新翻译」）。
- 等级: 高

### 2. PUT 整份替换，前端只发 `name` 和 `entries`：存一次就把描述、语言、`context` 清空，`enabled` 被重置；`enabled` 本身建任务时也不看
- 位置: `records.rs:35-54`（`update_glossary` 每个字段都取请求值）；`views.rs:8-24`（`GlossaryUpsertInput`：`description/source_lang/target_lang/entries` 缺省为空，`enabled` 缺省 `true`）；前端 `frontend/web/src/features/glossaries/domain/glossaries-store.ts:44-50`（编辑行没有 `context`）、`:136-144`（`normalizeEntryForRow` 读详情时丢掉 `context`）、`:151-181`（`readEditorPayload` 只出 `name`、`entries`，`context` 写死 `""`）、`controller.ts:179-193`；`entries.rs:13-44`（解析任务术语时不看 `glossary.enabled`）；`glossary-options.ts:47-60`（建任务下拉也不按 `enabled` 过滤）。
- 现状:
  - 术语表的 `description`、`source_lang`、`target_lang`、`enabled` 只有 agent / API 能写（AI `term-set` 会保留它们，`agent_translation_runner.py:325-331`），前端界面不显示也不发。界面上点一次「保存」，PUT 把这几项清成空、`enabled` 设回 `true`（未实跑，依据是缺省值和前端载荷）。
  - 词条的 `context`（Python 侧会用：`backend/translation/retainpdf_translation/core/terms/glossary.py:145-150`，只在附近文本含它时生效）：前端读详情就丢了，保存时全写成空。通过 `parse-csv` 导入带「上下文」列的 CSV，解析结果里有 `context`，进编辑器那一步就没了。
  - `enabled=false` 的表照样能被任务用：`resolve_task_glossary_request` 不看它，下拉也照样列出。这个字段目前只是个标签。
- 问题: 两个写入方（界面、AI 终端）对同一张表的字段集合不一致，界面保存会静默抹掉另一方写的东西；`context` 这个功能在界面上等于不可用。
- 建议:
  - 后端（只增不改）：PUT 照旧整份替换（文档已写），但对**请求里没出现**的顶层字段保留原值——把 `GlossaryUpsertInput` 的 `description/source_lang/target_lang/enabled` 改成 `Option`，`None` 时用旧值；`name`、`entries` 仍必填。词条级的 `context` 没法这样分辨（前端明确发了空串），所以需要前端改。另加 `PATCH /api/v1/glossaries/:glossary_id` 只改给出的字段也行，但先不必。
  - `enabled` 定语义：要么建任务时 `enabled=false` 的表 400（`glossary is disabled`），要么从视图里拿掉不再宣传。倾向前者，并让列表 `?enabled=true` 给下拉用。
  - 对前端的影响（写进 `docs/ops/planning/api-reorg/` 的整理记录）：`glossaries-store.ts` 的 `GlossaryEntryRow` 加 `context`，`normalizeEntryForRow` 保留 `entry.context`，`readEditorPayload` 用 `row.context` 而不是 `""`；编辑器可以先不显示这一列，但要原样带回。下拉若要按启用过滤，`glossary-options.ts:47` 改调 `glossaries?enabled=true`。
- 等级: 中

### 3. 去重只按「原词去空格、ASCII 小写」一个键：区分大小写的词、按上下文区分的同一个词被静默并成最后一条，和 Python 的键也不一样
- 位置: `entries.rs:141-158`（`dedupe_glossary_entries`、`glossary_entry_key = source.trim().to_ascii_lowercase()`）、`:101-117`（`merge_glossary_entries` 同一个键）；Python `retainpdf_translation/core/terms/glossary.py:63`（命中去重键是 `(source.casefold(), target.casefold(), level, context.casefold())`）。
- 现状: `match_mode=exact` 的 `US`（美国）和 `us`（微秒）、或者同一个 `bond` 配两个 `context`（化学「键」、金融「债券」），保存后只剩最后一条，接口照样 200，`entry_count` 少了也没有提示。`parse-csv` 返回的 `entry_count` 是去重后的，用户看不出哪几行被吞了。去重只认 ASCII 小写，`Ä`/`ä` 又算两个词。
- 问题: `context`、`exact` 这两个功能在「同词不同义」的场景下用不了；Rust 和 Python 对「同一条」的定义不一致。
- 建议: 键改成 `(match_mode == exact ? source : source.to_lowercase(), context.trim().to_lowercase())`——精确匹配的词区分大小写，带不同上下文的同词各留一条；`merge_glossary_entries`（内联覆盖表）用同一个键。`parse-csv` 的返回加 `duplicate_count`（只增），让前端提示「有 N 行重复，已按最后一行」。AI `term-set` 用 `casefold` 找要替换的词（`agent_translation_runner.py:304-312`），它写的都是 `case_insensitive`，不受影响。对前端的影响：只多一个字段；以前被吞掉的条目以后会留下。
- 等级: 中

### 4. CSV 解析：粘贴框提示的列序和无表头时的默认列序对不上；只有原词的一行让整份 400，报错没有行号
- 位置: `csv.rs:62`（无表头时默认 `source=0, target=1, note=2`，第 4 列以后不读）、`:72`（没有「类型」列时 `level` 默认 `preferred`）、`:81`（只在原词、译文、备注都空时跳过，`context` 不算）；`entries.rs:57-66`（JSON 这边四项都空才跳过；非 `preserve` 缺译文就 400）；前端 `features/glossaries/ui/GlossaryImportPanel.tsx:22`（占位符 `原词,译文,类型,匹配模式,备注`）、`glossaries-store.ts:140,159`（界面新行默认 `level=preserve`、`match_mode=case_insensitive`）；文档 `docs/core/api/index.md:780`（「`exact`：默认」）。
- 现状:
  - 用户照占位符的列序粘贴、但没写表头（例如 `DFT,密度泛函理论,专业译法,忽略大小写,固定译法`），第 3 列「专业译法」被当成备注，类型、匹配模式丢掉，`level` 变成 `preferred`（未实跑）。
  - 粘贴一列要保留的英文名（每行只有原词），界面上的习惯是「默认保留」，CSV 却默认 `preferred`，于是第一行就 400 `glossary entry requires both source and target`，整份不导入，也不说是第几行。
  - 只有备注的行（`,,说明`）也让整份 400；只有上下文的行在 CSV 里被跳过、在 JSON 里 400，两边规则不一样。
  - 从 Excel 直接复制的是制表符分隔，解析器只认逗号，整行落进「原词」一格，再因为缺译文 400（未实跑）。
- 问题: 默认值在后端、界面、文档三处各写一套；出错时用户没法定位。
- 建议: 后端：无表头时默认列序改成和占位符一致（`source,target,level,match_mode,note,context`）——这会改变「无表头三列」的解析结果，先和前端确认（目前只有管理框一个调用方，前端测试里有没有固定三列的用例要查一下）；或者反过来只改前端占位符成 `原词,译文,备注`，后端不动，影响最小。缺译文时 CSV 默认 `level` 跟界面一致取 `preserve`，或者报错里带行号（`row 3: glossary entry requires both source and target`）。首行没有逗号但有制表符时按 TSV 读（`csv::ReaderBuilder::delimiter(b'\t')`）。跳过空行的规则 CSV 和 JSON 统一成「四项都空才跳过」。文档 `docs/core/api/index.md` 写明无表头列序和各默认值。对前端的影响：选只改占位符的方案时，前端改 `GlossaryImportPanel.tsx:22` 一处；选改后端默认列序时前端不用动。
- 等级: 中

### 5. `POST /glossaries/import` 名不副实、和 POST / PUT 重复、没有调用方
- 位置: `backend/api/src/app/router/glossaries.rs:9-10`（说明「从 CSV 导入成新术语表」）；`api.rs:80-89`（请求体是 `GlossaryUpsertInput` JSON：`glossary_id` 空就新建，非空就更新那一张）；`backend/api/docs/api-spec/glossaries.md:73-78`（写的是 JSON，和代码一致）；`docs/api/00-接口目录.md:196`。
- 现状: 它不收 CSV，收的和 `POST /glossaries` 一模一样，多一个「带编号就改那一张」的分支，等于 `PUT /glossaries/:id`。真正从 CSV 进来的路径是 `parse-csv`（只解析、不存）→ 前端预览 → `POST` / `PUT`，`api-spec/glossaries.md:66-71` 推荐的也是这条。全仓库除了两个测试没有人调 `import`。多用户下它把要改的编号放在请求体里而不是路径里（`access.rs:63` 有按请求体字段 `glossary_id` 归类的分支，按约定不细查）。
- 问题: `parse-csv` 和 `import` 功能并不重复；`import` 和 `POST`/`PUT` 重复，而且登记表的一句话说明把人往错的方向引。
- 建议: 删掉 `import`（连同 `import_glossary_view`、两个测试），`UPDATE_ROUTES=1` 重新生成接口目录，`api-spec/glossaries.md`、`docs/core/api/index.md:756`、`docs/api/04-翻译/03-术语表.md:27` 去掉这一行。不想删就先把登记表说明改成「按 JSON 新建或覆盖（旧，用 POST / PUT）」。对前端的影响：无（前端没用）。
- 等级: 低

### 6. 并发：PUT 读改写没有版本号，后写覆盖；PUT 和 DELETE 撞上会把删掉的表写回来
- 位置: `records.rs:35-54`（先 `load_glossary_or_404`，再 `save_glossary`，不在一个事务里）；`database/retainpdf-db/src/db/glossaries.rs:10-39`（`save_glossary` 是 `INSERT … ON CONFLICT DO UPDATE`）；AI `agent_translation_runner.py:297-341`（`term-set`：GET 整张表 → 改一条 → PUT 整张表）；前端管理框打开时读一次详情（`controller.ts:137`），保存时 PUT 编辑器里的整份。
- 现状:
  - 两个写入方都是「读整张 → 改 → 写整张」，没有 `If-Match` / `updated_at` 前提。用户开着术语表管理框，同时让 AI 终端 `term-set` 加一个词，再在管理框点保存，AI 加的词就没了（未实跑）。
  - DELETE 和 PUT 并发：PUT 先读到旧行，DELETE 删掉，PUT 的 upsert 走 INSERT 分支把整张表插回去，`owner_user_id` 落成默认 `local`（未实跑；单机下没有可见影响，多用户已冻结只记录）。
- 问题: 单机也有两个写入方（界面、AI 终端），丢写是现实场景。
- 建议: 只增不改：PUT 请求体加可选 `expected_updated_at`，给了就在 `UPDATE … WHERE glossary_id=? AND updated_at=?` 里比，不等返回 409 `conflict_because("glossary_changed", …)`；不给照旧后写覆盖。`update_glossary` 改成 `UPDATE`（没有行就 404），不再走 upsert，顺带堵住复活。AI `term-set` 带上读到的 `updated_at`，409 时重读重试一次。对前端的影响（写进整理记录）：`controller.ts:192` 保存时带上详情里的 `updated_at`；409 时提示「术语表已被别处修改，请刷新后再改」。不改也照旧能用。
- 等级: 中

### 7. 读错说成 404；建任务引用不存在的术语表也是 404
- 位置: `records.rs:108-111`（`load_glossary_or_404`：`map_err(|_| not_found(..))`，GET / PUT / DELETE / 导出 / 建任务全经过它）；`entries.rs:28`（建任务时）；对照 `retainpdf-api-base/src/error.rs:289-298`（`from_lookup`，注释写明不要再这样写）。库层 `get_glossary` 用 `with_context` 包 `query_row`，`QueryReturnedNoRows` 留在错误链里，`from_lookup` 可以直接用。
- 现状: 库被锁、读不出来时，GET 报「术语表不存在」；建任务时 `translation.glossary_id` 指向一张已删的表，`POST /jobs` 回 404（请求体里引用了不存在的东西）。
- 问题: 违反规矩第 5 条（读错不能说成 404）；建任务那一处按规矩第 1 条（请求写错 400）更像 400——和书库审计第 13 条「建合集时上级不存在 404」是同一类。
- 建议: `load_glossary_or_404` 换成 `AppError::from_lookup(err, format!("glossary not found: {id}"))`；建任务那一处另写：没有这张表时 400 `translation.glossary_id not found: …`，读错 500。对前端的影响: 读错由 404 变 500，前端对非 OK 统一提示失败，无影响；建任务由 404 变 400，前端建任务失败只显示 message（下拉已经会过滤掉已删的表，`glossary-options.ts:54-58`），不受影响——改之前在 `features/ingest` 里 `git grep 404` 确认一遍。
- 等级: 中

### 8. 导出 CSV：公式注入、没有 BOM、没有 `Content-Disposition`
- 位置: `views.rs:106-137`（`glossary_to_csv_export`、`escape_csv_cell` 只处理逗号、引号、换行）；`views.rs:126`（算了 `file_name` 却没人用）；`backend/api/src/routes/glossaries.rs:74-86`（只设 `Content-Type`）；前端 `controller.ts:232`（自己拿名字拼 `fallbackName`）、`frontend/web/src/platform/utils/downloads.ts:309`（会读 `content-disposition`）。
- 现状:
  - 单元格以 `=`、`+`、`-`、`@`（以及制表符、回车）开头时原样写出，用 Excel / WPS 打开会当公式执行。来源可以是同步过来的表、AI 写的条目、别人给的 CSV 先导入再导出。化学术语里 `-OH`、`+1` 这种合法开头也会被当公式，显示成错误值。
  - 没有 UTF-8 BOM：中文 Windows 的 Excel 双击打开会把中文显示成乱码（导入方向没问题：`sanitize_csv_cell` 会去掉 BOM，`csv_text` 走 JSON 只能是 UTF-8）。
  - 没有 `Content-Disposition`，下载的文件名全靠前端拼。
- 问题: 导出给表格软件看的文件，打开即出错或有执行风险（单机、数据基本是自己的，风险低）。
- 建议: 导出时以 `= + - @ \t \r` 开头的单元格前加 `'`，`parse-csv` 读到「`'` 后面跟这几个字符之一」时去掉这个 `'`，导出再导入能还原；文件开头加 `\u{feff}`；响应加 `Content-Disposition: attachment; filename*=UTF-8''<百分号编码的 file_name>`。补测试：导出 → `parse-csv` 往返相等。对前端的影响：无（前端下载拿到的文件名会变成后端给的，和现在拼的一致）。
- 等级: 低

### 9. 规矩没跟上：列表不分页也没登记、查询参数不拒未知字段、`?q=` 空串返回空表、异步线程直接跑 SQLite、没有契约
- 位置: `api.rs:34-44`、`records.rs:56-106`（全量读出、在内存里过滤）；`views.rs:144-154`（`ListGlossariesQuery` 没有 `deny_unknown_fields`）；`records.rs:92-104`（`needle` 为空时 `!needle.is_empty() && …` 为假，**所有**表都被滤掉）；`backend/api/src/routes/glossaries.rs` 8 个处理函数都在 async 里直接调同步的 `Db`；`docs/api/00-约定/03-分页.md:51`（「不分页的列表」只列了合集）；`contracts/` 没有术语表的 schema，前端 `glossaries.ts` 全是 `Promise<any>`，类型在 `glossaries-store.ts:34-50` 手写。
- 现状: 列表一次全给，每张表都会把 `entries_json` 整个反序列化出来只为了数条数。`GET /glossaries?q=` 返回 `items: []`，书库那边空 `q` 是不过滤。请求体两个类型已经 `deny_unknown_fields`，查询参数没有。
- 问题: 违反规矩第 3、5 条的几个小点；量都很小，不影响功能。
- 建议: 术语表数量小，在 `03-分页.md` 写明「`GET /glossaries` 不分页」；`ListGlossariesQuery` 加 `deny_unknown_fields`（前端、agent 都不带参数，安全）；空 `q` 视为不过滤；处理函数包 `spawn_blocking`（照 `20261019-书库读错和阻塞线程.md` 的写法）；列表可以在库层直接 `SELECT json_array_length(entries_json)`，不必读全量词条。契约等术语表接口定型后再补 `glossaries.v1`。对前端的影响：无。
- 等级: 低

### 10. 校验缺口：坏正则静默永不命中、`context` 没有长度上限、未知 `level` / `match_mode` 静默改默认、库里词条读坏静默变空表
- 位置: `entries.rs:46-99`（只校验 `source/target` ≤200、`note` ≤500，`context` 不限；`match_mode=regex` 不编译检查）、`:119-139`（未知值落到 `preferred` / `exact`）；Python `glossary.py:133-138`（正则编译失败换成永不匹配的 `(?!x)x`）；`database/retainpdf-db/src/db/rows.rs:161`（`entries_json` 解析失败 `unwrap_or_default()`，同文件已经有 `json_column_decode_error` 可用）。
- 现状: 用户写错一个正则，保存成功、任务跑完，这条术语从没生效，没有任何提示；`level: "canonicl"` 变 `preferred`，和 `translation.rs:5-15` 那段注释批评的「静默归一化」是同一种问题。库里某张表的 `entries_json` 坏了，GET 返回 0 条，任务用它时也是 0 条，用户以为术语表是空的、再保存一次就真没了。
- 建议: `match_mode=regex` 时用 `regex` crate 或者至少检查能编译（Rust 和 Python 正则语法有出入，只拒绝明显编译不过的即可），400 带词条原词；`context` 上限和 `note` 一样 500；JSON 接口（不是 CSV）收到不认识的 `level` / `match_mode` 返回 400，CSV 那边仍然宽松认别名；`row_to_glossary_record` 改用 `json_column_decode_error`，读坏的表 500 而不是空表。对前端的影响：前端只发三个合法 `level`、三个合法 `match_mode`，不受影响。
- 等级: 低

### 11. 文档几处和代码对不上
- 位置: 登记表 `router/glossaries.rs:9`（「从 CSV 导入」，见第 5 条）；`backend/api/docs/api-spec/glossaries.md:66-71`（「前端读用户的 CSV 文件」——前端只有粘贴框，没有选文件）；`docs/core/api/index.md:779-780`（`match_mode` 默认 `exact`，界面新行默认 `case_insensitive`、`level` 默认 `preserve`，CSV 默认 `preferred`）；`docs/api/04-翻译/03-术语表.md` 没写任务存的是快照、建好后改表删表不影响这次翻译、重跑时怎么处理（第 1 条）；没写 PUT 会把没给的顶层字段清空（第 2 条）；上限（每表 200 条、原词译文 200 字、备注 500 字、名字 120 字、描述 500 字、合并后 200 条）只在代码里。
- 建议: 跟着第 1、2、4、5 条一起改；`docs/api/04-翻译/03-术语表.md` 加「上限」「快照与重跑」两小节。
- 等级: 低

## 查过、没问题的

- 路由全部在 `crate::endpoints!` 表里，没有直接写 `.route(`；`routes.txt`、`00-接口目录.md` 与表一致；`glossary_id` 已在 `access.rs:44,63` 归类。
- 请求体 `GlossaryUpsertInput`、`GlossaryCsvParseInput`、词条 `GlossaryEntryInput` 都有 `deny_unknown_fields`；前端发的字段（`name`、`entries[].source/target/level/match_mode/context/note`）都在里面。
- 大小上限：术语表接口走默认的 2 MB 请求体上限（登记表没给这几条关掉上限），`parse-csv` 的 `csv_text` 也受它限；条数在去重后卡 200，单条长度有上限；`csv` 解析失败 400，message 只带 csv 库的错误位置，不回显内容。
- 编码：导入只收 JSON 字符串（UTF-8），`sanitize_csv_cell` 去掉首格 BOM；表头识别中英文别名都在（`csv.rs:104-143`）。
- 导出的引号转义正确（双写 `"`），含逗号、换行的格会加引号；导出表头 `source,target,note,level,match_mode,context` 能被 `parse-csv` 原样认回。
- 任务请求里存的是术语快照（加 `glossary_id`、`glossary_name`、计数），建好的任务不受之后改表、删表影响；在跑的任务不会因为删表失败；原地重渲染（`relaunch_in_place`）不重新解析术语表。
- 内联条目和表合并后再卡 200 条（`entries.rs:33-37`，有测试）。
- 返回里没有服务器路径、密钥；路径参数 `glossary_id` 只进 SQL 参数，不拼路径。
- 名字 1..=120 字、描述 ≤500、语言 ≤64 都是 400。
- 多用户相关（`owner_filter` 列表过滤、新建记 owner、`isolation.rs:239-251` 的隔离测试）只看了有没有，按约定不细查。
