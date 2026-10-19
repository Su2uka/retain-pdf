# 报告与译文（reports + translations）

审计范围：`router/jobs/{reports,translations}.rs`、`routes/jobs/{reports,translations}.rs`、`retainpdf-jobs` 的 `reports/`、`translations/`、`views/translation.rs`、`views/common.rs`（`ListTranslationItemsQuery`）；契约 refine-report.v1、token-usage.v1、translation-revisions.v1。只读，没跑 cargo。

## 接口一览

路径省略前缀 `/api/v1/jobs/:job_id`。`routes/…` 指 `backend/api/src/routes/jobs/`。

| 方法 地址 | 处理函数 | 返回类型 | 契约 | 文档 | 调用方 | 今日调用 |
|---|---|---|---|---|---|---|
| GET /reports/failure | `reports::get_job_diagnostics` | `JobDiagnosticsView` | 无 | `docs/api/08-调试诊断/01-失败结构.md`（字段已过期）、`docs/core/api/index.md:482`、`docs/core/rust_api/02-…:214`、`api-spec/jobs-*.md` | 前端 `frontend/packages/api/src/jobs-actions.ts:104`（`fetchJobDiagnostics`） | 15 |
| GET /reports/translation | `reports::get_translation_diagnostics` | `TranslationDiagnosticsView{job_id,summary}` | 无 | `api-spec/translation-diagnostics.md`、`docs/api/08-…/02-translation-debug.md` | 前端 `translation-debug.ts:7`（`fetchTranslationDiagnostics`，任务详情「翻译调试」） | 0 |
| GET /reports/translation-qa | `reports::get_translation_qa` | `JobReportView{job_id,report}` | 无 | `api-spec/translation-diagnostics.md`（schema 名写错） | CLI `retainpdf-agent translation qa`（`backend/api/src/bin/retainpdf-agent.rs:332`）→ AI 终端 agent；前端无 | 0 |
| GET /reports/layout-fit | `reports::get_fit_report` | `JobReportView` | 无 | 同上 | CLI `fit-report`（agent.rs:334）；前端无 | 0 |
| GET /reports/refine | `reports::get_refine_report` | `JobReportView` | refine-report.v1（`RefineReportView`，Python 产出方有测试 `test_translation_refine.py:793`；Rust 无测试钉） | 同上 | CLI `refine-report`（agent.rs:333）、`backend/ai/retainpdf_ai/agent_translation_runner.py:98`；前端无 | 0 |
| GET /reports/quality | `reports::quality_summary_route` | `QualitySummaryView` | 无（前端手写类型，注释说“契约发布后改成引类型”） | 只有接口目录一行，无字段文档 | 前端 `quality.ts:96` → `QualityPanel.tsx:19` | 5 |
| GET /reports/quality/items | `reports::quality_items_route` | `QualityItemsView` | 无 | 只有接口目录一行 | 前端 `quality.ts:107`（`fetchQualityItems`） | 0 |
| GET /reports/usage | `reports::job_usage_route` | `UsageSummaryView` | token-usage.v1（`reports/usage/tests.rs:159` 逐字段比对） | 契约自带说明；docs/api 只有目录行 | 前端 `usage.ts:29`（`fetchJobUsage`） | 0 |
| GET /translations | `translations::list_translation_items` | `TranslationDebugListView{items,total,limit,offset}` | 无 | `api-spec/translation-diagnostics.md`、`docs/api/08-…/02-translation-debug.md` | 前端 `translation-debug.ts:28`（`fetchTranslationItems`） | 0 |
| GET /translations/:item_id | `translations::get_translation_item` | `TranslationDebugItemView` | 无 | 文档写错（见问题 9） | 前端 `translation-debug.ts:37`；CLI `translation item` | 0 |
| PATCH /translations/:item_id | `translations::revise_translation_item_route` | `TranslationRevisionView` | translation-revisions.v1（`api_tests/translation_debug/revisions.rs` 的 `assert_keys_match` 钉字段） | `api-spec/translation-diagnostics.md` 写得全 | CLI `translation revise`；AI 终端 agent；前端 mock 之外无调用 | 0 |
| GET /translations/:item_id/revisions | `translations::get_translation_item_revisions` | `TranslationRevisionHistoryView` | translation-revisions.v1 | 同上 | CLI `translation revisions`；前端无 | 0 |
| POST /translations/:item_id/replay | `translations::replay_translation_item_route` | `TranslationReplayView{job_id,item_id,payload}` | 无 | 文档两处相互矛盾（见问题 9） | 前端 `translation-debug.ts:46`（`replayTranslationItem`） | 0 |

## 问题

### 1. 译文列表 `limit` 不设上限，配置里写的上限没用上
- 位置: `backend/packages/retainpdf-jobs/src/translations/index.rs:25-27`、`views/common.rs:137-152`、`backend/packages/retainpdf-core/src/config/limits.rs:7-8`
- 现状: `ListTranslationItemsQuery.limit` 是 `u32`，默认 20（`DEFAULT_LIST_LIMIT`），读出来直接 `start + limit`，不夹、不校验。`limits.rs` 注释写着 `MAX_LIST_LIMIT` 是“documents, jobs, translations”的上限，但译文列表根本没用它。`limit=0` 返回空 items 但 200；`limit=4000000000` 一次返回全部，回显的 `limit` 也原样是 4000000000。
- 问题: 同一个 jobs API 里的几个分页，规则全不一样：
  - 译文列表：默认 20，无上限，不报错；
  - `/reports/quality/items`：默认 200，1..=1000，越界 400；
  - `/data/*`：默认 200，1..=1000，越界 400；
  - 任务列表：默认 20，`clamp(1, 500)` 静默夹；
  - 事件：默认 500，1..=500。
  
  译文列表的响应形状（`items/total/limit/offset`）和 quality/items（多一个 `kind`）一致，但没有 `has_more`，只能靠 `total`。
- 建议: 给译文列表加上限。要和 `data/`、`quality/items` 保持一致就用越界 400（`limit` 1..=500 或 1000），把 `limit`/`offset` 回显成实际生效的值。顺便决定一条全接口通用的规则（“越界 400”还是“静默夹”），写进 `docs/api`。对前端的影响: 前端传 `limit=20`（`translation-debug.ts:21`），不受影响；传了更大值的调用方才会看到 400。
- 等级: 中

### 2. 8 个报告在“报告文件还没生成”时的行为不一致
- 位置: `reports/translation_reports.rs:17-49`（404）、`reports/quality/mod.rs`（`read_json` 失败给 `None`）、`reports/usage/api.rs:31-36`（全 0）、`reports/failure.rs`（永远 200）
- 现状（已逐个读过代码）:

  | 接口 | 任务不存在 | 报告文件不存在（任务存在） | 任务失败 |
  |---|---|---|---|
  | reports/failure | 404 | 不依赖报告文件；任务没失败也 200，`summary:"任务当前没有失败诊断"` | 200 |
  | reports/translation | 404 | 404 `translation diagnostics not found: <id>` | 同左，看文件在不在 |
  | reports/translation-qa、layout-fit、refine | 404 | 404（消息分别是 `translation qa report not found` / `fit report not found` / `refine report not found`） | 同上 |
  | reports/quality | 404 | 200，缺的段是 `null`（`preparation/qa/layout/refine`），`untranslated` 全 0 | 200 |
  | reports/quality/items | 404 | 200，`items:[]`、`total:0` | 200 |
  | reports/usage | 404 | 200，全 0 的汇总 | 200 |
  | translations（列表/单条/revisions） | 404 | 404 `translation manifest not found` | 看产物 |

- 问题: “任务不存在”和“还没产出”都是 `NOT_FOUND`，错误码分不开（契约 `refine-report` 的 errors 里也只能写成一句“job does not exist or the job has no refine report yet”）。前端 `translation-debug.ts:30` 把译文列表的 404 当“空列表”，`:9` 把诊断的 404 当“请先完成翻译”，其实是它自己在猜。同样是“没数据”，质量卡给 200+null，原始报告给 404，用量给 200+0，调用方每个接口都要单独处理。
- 建议: 不改现有状态码（前端依赖），在两个 404 上加区分：任务不存在保持 `NOT_FOUND`，报告缺失在 `error.details` 里放 `{"reason":"report_not_ready"}`（`AppError` 目前 `NotFound` 没有 details，需要像 `TranslationRevision` 那样加一种带 details 的错误，或给 `NotFound` 加可选 details）。把规则写进 `docs/api`（“原始报告：没有就 404；摘要类：没有就 200 + null”），摘要类和原始报告分开写清楚。对前端的影响: 无（只加字段）。
- 等级: 中

### 3. 译文列表读的是翻译结束时写的索引，PATCH 之后不刷新，列表和单条对不上
- 位置: `translations/index.rs:44-48`、`translations/artifacts.rs:11-23`；Python 侧 `backend/translation/retainpdf_translation/workflow/revision.py`（没有碰 `translation_debug_index.json`）
- 现状: `load_translation_debug_index` 先读 `artifacts/translation_debug_index.json`，只有文件不存在才从 manifest 重建。这个索引只在翻译阶段结束时写一次（`translate_only_pipeline.py:190`、`book_pipeline.py:120`）。`translation-revise` 只改页文件、checkpoint 和 `revisions.v1.jsonl`。
- 问题: 手改或精修之后，`GET /translations` 的 `translated_preview`、`final_status` 还是旧的；`GET /translations/:item_id` 和 PATCH 返回的是新的。`?q=` 搜索也搜的是旧译文。精修（`source=refine`）会批量改，影响更大。
- 建议: 二选一。(a) `translation-revise` 写回后同步更新索引里对应条目（Python 侧，和 checkpoint 一起提交）；(b) 列表不再信索引，直接读 manifest 重建（页文件不大，`GET item` 本来就每次全扫）。我倾向 (b)，少一份要维护的派生产物。需要补一条 PATCH 后再 GET 列表的测试（`translation_debug/` 现有测试没覆盖）。对前端的影响: 无，字段不变，只是变准。
- 等级: 中

### 4. 重翻（replay）：没有校验、没有超时、找不到条目返回 500，和 PATCH 不对等
- 位置: `translations/replay.rs:11-98`（对比 `translations/revision.rs:46-58, 101-148`）；`backend/pipeline/devtools/replay_translation_item.py:97`、`:396-405`
- 现状:
  - 没调 `validate_item_id`（PATCH 和 revisions 都调了）。`item_id` 直接做 argv 传给 Python。
  - 没有超时、没有 `kill_on_drop`（PATCH 的 `translation-revise` 有 120 秒超时 + `kill_on_drop`）。replay 要调模型，慢的时候请求线程一直挂着，客户端断开后 Python 进程还在跑。
  - 条目不存在时 Python 抛 `RuntimeError("translation item not found")`，没人接，进程非 0 退出，Rust 返回 500 `translation replay failed: …`（对外只看到 `internal server error (ref …)`）。前端 `translation-debug.ts:48` 为 404 写了“未找到该翻译 item”的提示，实际永远走不到。
  - 没有虚拟任务（`merged-…`）保护。PATCH 对虚拟 id 返回 409 `read_only_job`，replay 对它去找 `resolve_job_root` 然后 404。
  - 任务在跑时可以 replay（这是只读调用，所以是否合理要看产品意图，至少文档没说）。
- 问题: 同一个“译文条目”的两个动作，错误处理一个精细一个粗糙；用户拼错条目编号拿到 500。
- 建议: 在 Rust 侧先复用 `load_translation_debug_item_view` 查条目（不存在直接 404，顺便得到 `item_id` 合法性），再加 `validate_item_id`、一个超时（比如 300 秒）和 `kill_on_drop(true)`；Python 的“条目不存在/没有 key”改成退出码或 JSON 里带 `error`，Rust 映射成 404/409。对前端的影响: 无（只是原来的 500 变 404，前端已有 404 分支）。
- 等级: 中

### 5. quality、usage 不走 `load_supported_job`，和其余 11 个接口的“任务加载”规则不同
- 位置: `reports/quality/api.rs:39-43`、`reports/usage/api.rs:31-36`；对比 `reports/queries.rs`、`translations/queries.rs`（都用 `load_supported_job`）
- 现状: quality 直接 `db.get_job(...)`，**任何** 数据库错误都被当成 404（`.map_err(|_| not_found)`）；usage 在数据库查不到但目录还在时仍给 200。两者都跳过 `ensure_supported_job_layout`（其余接口遇到老目录格式的任务返回 409 `LEGACY_JOB_UNSUPPORTED_MESSAGE`）和合并阅读的虚拟 id（`merged-<文档>-<指纹>`，其余读接口认）。
- 问题: 同一类任务 id，`reports/translation-qa` 能读、`reports/quality` 404；老格式任务读 `reports/failure` 是 409，读 `reports/quality` 是 200 或空。数据库真出错时（锁、损坏）被说成“job not found”。
- 建议: quality 改用 `load_supported_job` 再算路径；usage 的“库里没有但目录在”若是有意（删库后仍能看用量），在代码注释和文档里写明。对前端的影响: 无（正常任务行为不变）。
- 等级: 中

### 6. 报告命名：`translation` / `translation-qa` / `quality` / `layout-fit` 对不上文件名，也对不上彼此的关系
- 位置: `app/router/jobs/reports.rs`；文件名对照见 `retainpdf-core/src/storage_paths/resolvers.rs:162-192`
- 现状:
  - `reports/translation` 实际是 `translation_diagnostics.json`（运行统计），不是“翻译报告”；`reports/translation-qa` 是 `translation_qa.v1.json`（QA 违规清单）。看地址分不清谁是谁。
  - `reports/layout-fit` 对应 `fit_report.v1.json`，旧地址是 `/render/fit-report`；`refine`、`failure` 的名字和文件名/内容一致。
  - `quality` 是 `translation_qa` + `fit_report` + `refine_report` 的摘要；`quality/items` 的 `kind=qa|layout|escalated` 分别是这三份的逐条投影（`reports/quality/mod.rs:327-420`），`kind=untranslated` 则是扫 `translated/page-*.json`，不来自任何报告。内容有重叠，但是有意设计成“摘要 + 按块清单”，没有重复实现。
  - 响应的外层键也不统一：原始报告是 `{job_id, report}`，translation 诊断是 `{job_id, summary}`，quality 是平铺 `{job_id, preparation, qa, layout, refine, untranslated}`。
- 问题: 不是 bug，是认知成本。新增报告时没有规则可循。`quality` 里的 `qa` 段和 `reports/translation-qa` 的 `report` 容易被当成同一个东西。
- 建议: 不动地址（前端和 CLI 都在用），在 `docs/api/01-任务/04-任务接口分类.md` 的 reports 小节补一张“接口 → 文件 → 用途 → 是否摘要”的表，写明 `reports/translation` 其实是翻译诊断。若以后要改名，用 `reports/translation-diagnostics`，旧地址按本仓库规则保留一轮。把 `{job_id, summary}` 改成 `{job_id, report}` 会破坏前端，不建议现在动。对前端的影响: 文档改动无；改名才有。
- 等级: 低

### 7. 契约覆盖不均，质量卡的前端类型已经和后端不一致
- 位置: `frontend/packages/api/src/quality.ts:8-60`（手写类型）对比 `reports/quality/mod.rs:109-150`（Rust 视图）；`contracts/token-usage.v1.schema.json` 的 `rust_view`；`contracts/README.md` 的表
- 现状: 13 个接口里，有契约的是 refine（响应）、usage（响应）、PATCH 和 revisions（请求+响应+错误结构）。没有契约的：failure、translation、translation-qa、layout-fit、quality、quality/items、译文列表、单条、replay。前端自己写了 `QualitySummaryView`，注释写“契约发布后改成引类型”。已经对不上的地方：
  - `layout.min_scale`、`min_final_font_size` 前端写 `number`，Rust 是 `Option<f64>`（没排版数据时为 `null`）；
  - `refine.qa_before/qa_after` 前端写非空，Rust 是 `Option`；
  - `QualityItem.page` 前端写 `number`，Rust 是 `Option<u64>`。
  
  `layout-fit`、`translation-qa` 的 JSON 只有 Python 侧有 schema 名（`fit_report_v1`、`translation_qa.v1`），没有发布到 `contracts/`。refine-report 的 Rust 路径没有测试去钉它，只有 Python 产出方有（`test_translation_refine.py:793`）；因为 Rust 原样透传，所以问题不大。token-usage 和 refine 契约里的 `rust_view`（`services/jobs/reports/usage ::`、`JobReportView`）是拆包前的旧路径。
- 问题: quality 是前端 5 次/天在用的接口，却没有契约，类型已经可能在 `min_scale: null` 时出错（前端把它当数字格式化）。
- 建议: 给 `reports/quality`、`quality/items` 补 `quality.v1.schema.json`，并加一个 Rust 测试（仿 `reports/usage/tests.rs:159`）；failure 的 `JobDiagnosticsView` 也一并进契约（见问题 8）。`min_scale` 等在前端类型里改成 `number | null`。更新 token-usage 里的 `rust_view` 路径。译文列表和单条条目是调试接口，可以只写文档不进契约。对前端的影响: 类型改 `number | null`；用生成的类型替换手写类型（`contracts` 已有生成流程）。
- 等级: 中

### 8. `reports/failure` 的行为和命名不符，字段文档已过期，空值风格混用
- 位置: `reports/failure.rs:36-96`、`views/job_types.rs:198-212`；文档 `docs/api/08-调试诊断/01-失败结构.md:14-26`
- 现状:
  - 任务没失败（成功、运行中）也返回 200，`summary:"任务当前没有失败诊断"`，`failed_stage` 取的是 `job.stage`（`failure.rs:75`）。成功的任务会返回 `failed_stage:"render"` 之类，名字叫“失败阶段”，值是“当前/最后阶段”。
  - 空值风格混用：`failure_code`、`detail`、`suggestion`、`ocr_ambiguity` 用 `null`；`render_diagnostics`、`translation_request_recovery` 用 `skip_serializing_if` 整个不出现。
  - `01-失败结构.md` 的示例字段（`failure_category`、`provider`、`raw_exception_type`、`traceback`、`raw_excerpt`）在 `JobDiagnosticsView` 里一个都没有；真实字段（`failure_code`、`failed_stage`、`failed_substage`、`summary`、`detail`、`suggestion`、`retryable`、`resume_available`、`render_diagnostics`、`translation_request_recovery`、`ocr_ambiguity`）里，`docs/core/rust_api/02-…:216-226` 只列了 7 个，`translation_request_recovery` 在 docs/api 下没有字段说明。
- 问题: 前端按文档的字段去读会拿到 `undefined`；对非失败任务返回 `failed_stage` 容易被误当成失败位置。
- 建议: 文档先改成与 `JobDiagnosticsView` 一致（把 `failure_category` 等删掉，补全 11 个字段，写明“任务没失败时 `failure_code:null`、`failed_stage` 是当前阶段，仅 `summary` 有意义”）。代码上不动（前端 15 次/天在用），长期可在非失败任务时把 `failed_stage` 置 `null`——这会改变现有响应，需要前端确认 `controller-overview.ts` 没有依赖。对前端的影响: 文档改动无；改 `failed_stage` 才有。
- 等级: 中

### 9. 译文接口的文档互相矛盾、有缺项
- 位置: `backend/api/docs/api-spec/translation-diagnostics.md:11,29,188-250`；`docs/api/08-调试诊断/02-translation-debug.md`
- 现状:
  - “All four endpoints are job-local…” 文件现在写了 10 个接口。
  - “Raw Translation Item” 写“same payload shape as the saved translated item”，实际返回 `{job_id, item_id, page_idx, page_number, page_path, item}`（`views/translation.rs` 的 `TranslationDebugItemView`），保存的条目在 `item` 里。
  - “Replay Translation Item: Response Projection” 小节的示例其实是 GET 单条的响应，不是 replay 的；真正的 replay 响应在下一个小节（`{job_id,item_id,payload}`），两节标题都叫 replay。
  - 示例里 schema 名写 `translation_qa_v1`，Python 实际是 `translation_qa.v1`（`qa/models.py:13`）。
  - `docs/api/08-…/02-translation-debug.md` 完全没提 PATCH、revisions、translation-qa、layout-fit、refine、quality。
  - 没有任何文档说明 `reports/quality`、`quality/items`、`reports/usage` 的字段（只有接口目录一行；usage 有契约，quality 完全没有）；`quality/items` 的 `?kind=layout|untranslated|qa|escalated[&page][&severity][&offset][&limit]` 参数只在处理函数的 `///` 里。`severity` 只对 `kind=qa` 生效，其他 kind 静默忽略，也没写。
  - `replay_translation_item.py:369` 的帮助文字还写着旧地址 `PATCH translation/items/:item_id`。
- 问题: 读文档的人会按错的形状去解析；新增的接口根本找不到文档。
- 建议: 在 `translation-diagnostics.md` 修上面几条；`quality`、`quality/items`、`usage` 在 `docs/api` 补一页。对前端的影响: 无。
- 等级: 中

### 10. 译文列表的查询参数宽松，和 quality/items 的严格不一致
- 位置: `views/common.rs:136-152`（无 `deny_unknown_fields`）对比 `routes/jobs/reports.rs` 的 `QualityItemsParams`（有）
- 现状: `/reports/quality/items?kind=qa&unknown=1` 返回 400（测试 `api_tests/quality.rs:47` 钉着）；`/translations?finalstatus=failed`（拼错）静默当没传，返回全部。`/translations` 的过滤里，`final_status`、`error_type` 做精确匹配（小写化），`route` 做子串匹配，`q` 做子串匹配，`page` 是 1 基页码；这些语义只有代码里有，文档只列了参数名。
- 问题: 拼错参数名得到“看起来正常”的全量结果，调试时容易被误导。
- 建议: 给 `ListTranslationItemsQuery` 加 `#[serde(deny_unknown_fields)]`，并在文档里写每个过滤参数的匹配方式。对前端的影响: 前端只发这 7 个参数（`translation-debug.ts:21-27`），不受影响。
- 等级: 低

### 11. PATCH 的并发控制是可选的，且这组接口读不到“当前 generation”
- 位置: `views/translation.rs`（`ReviseTranslationItemRequest.expected_generation: Option<u64>`）、`translations/edits.rs:37-58`
- 现状: `expected_generation` 不带就是“后写赢”。要带，调用方得先知道 checkpoint 的 generation，但这组读接口里没有：`GET /translations`、`GET /translations/:item_id`、`GET …/revisions` 的响应都不含 generation（revisions 的记录里有 `generation`/`previous_generation`，但在没有任何修订的新任务上是空的）。只有上一次 PATCH 的响应或实时译文的接口能拿到。
- 问题: 多个调用方（网页编辑、终端 agent、精修）同时改同一条时，乐观锁形同虚设，第一次改之前没法用。已有保护：任务在跑（queued/running）返回 409 `job_running`；Python 侧还有 checkpoint 文件锁；相同文本返回 `changed:false`。所以真实风险只是“两个人先后改，后者覆盖前者”。
- 建议: 在 `GET /translations/:item_id` 响应里加一个 `generation`（当前 checkpoint generation），或在 `revisions` 响应加 `current_generation`；文档注明 agent 应带 `expected_generation`。只增字段。对前端的影响: 无（只加字段）。
- 等级: 低

### 12. 处理函数细节：两个接口在异步线程里读大文件，一个多余参数，缺注释
- 位置: `backend/api/src/routes/jobs/reports.rs:81`（`_headers: HeaderMap`）、`:70-109`
- 现状: 模块头注释写“读的大多是任务目录里的报告文件，放到阻塞线程里做”，但只有 quality、quality/items、usage 用了 `blocking(...)`。`translation`、`translation-qa`、`layout-fit`、`refine`、`failure` 以及整个 `translations` 都在 async 线程里 `std::fs::read_to_string` + 解析 + 脱敏（报告几百 KB，`quality/mod.rs` 的注释自己也写“各有几百 KB”；译文列表和单条每次读 manifest 和全部页文件）。`get_translation_diagnostics` 带着一个没用的 `_headers: HeaderMap`。`get_translation_diagnostics`、`get_translation_qa`、`get_fit_report`、`get_refine_report` 和 translations 的处理函数缺 `///`。
- 问题: 单用户开发下看不出，并发多时会占住异步线程。
- 建议: 把这些读文件的调用也包进 `blocking`，删掉 `_headers`。对前端的影响: 无。
- 等级: 低

### 13. 条目详情和修订历史每次都全扫所有页文件
- 位置: `translations/item.rs:15-23`、`translations/revision.rs:211-246`
- 现状: `GET /translations/:item_id` 通过 `load_manifest_pages` 读出全部页文件再逐条比对；`GET …/revisions` 先做同样的全扫（只为判断条目存在，丢弃结果），再读整个 `revisions.v1.jsonl` 过滤。agent 逐条检查时，每次调用是 O(全书)。
- 问题: 性能，不是正确性。书越大越慢。
- 建议: 条目编号形如 `p003-b004`，可以先从编号推出页号只读那一页；修订历史的存在性检查改用索引或只读对应页。对前端的影响: 无。
- 等级: 低

## 没问题的

- 路由登记：13 条都在 `endpoints!` 表里，地址前缀统一，`:job_id`、`:item_id` 命名一致；`routes.txt`、`docs/api/00-接口目录.md` 与表一致；访问级别都是 `signed`。
- 查询参数命名：`limit/offset/page` 在译文列表、quality/items 里同名同义（`page` 都是 1 基页码）。
- 脱敏：`translation`、`translation-qa`、`layout-fit`、`refine`、单条、replay 都过 `redact_json_value`，并有测试（`translation_debug/diagnostics.rs`、`items.rs`、`replay.rs`）。
- 报告文件由 Python 用 `save_json_atomic` 写（QA、fit、refine），读到半截文件的 500 风险不存在。
- PATCH 的错误模型是这组里做得最好的：`TranslationRevision` 带结构化 `details.reason`；422 校验失败、409 任务在跑/generation 过期/源任务拥有译文/只读虚拟任务，都有测试（`translation_debug/revisions.rs`），契约枚举与 Rust 一致；请求体 `deny_unknown_fields`；`translated_text`/`reason` 长度在 Rust 先挡一道；`changed:false` 时不追加修订记录。
- 修订历史对不存在的条目返回 404，不是空列表（注释说明了原因）。
- 修订写回只写任务自己的 `translated/`，借用源任务译文的渲染任务返回 409 `translations_owned_by_another_job`，不会悄悄改掉源任务。
- usage 契约和 Rust 类型有逐字段比对测试；quality/items 对 `kind`、`limit` 越界有 400 测试。
- 同一份数据没有两套实现：quality 摘要是对三份报告的有意投影，不是复制逻辑；`translation_item_id`（三位）和 `item_id`（四位阅读页编号）两套编号的差异已在 `QualityItem` 注释里写明。
- 没发现“零调用方、可以删”的接口：前端不调的 translation-qa、layout-fit、refine、PATCH、revisions 都有 `retainpdf-agent` CLI 和 AI 终端 agent 在用（`agent_capabilities` 白名单包含它们；replay 不在白名单里，只有前端调用）。今日 0 次调用的只是没被触发。
