# 操作与进度（actions + progress）

审计范围：`router/jobs/{actions,progress}.rs`、`routes/jobs/{actions,progress}.rs`、`retainpdf-jobs` 的 `actions/`、`progress/`，以及它们的返回类型（`views/job_types.rs`、`views/common.rs`、`views/translation.rs`、`views/live_translation.rs`）。只读审计，没有跑 cargo，没有调接口。

## 接口一览

| 方法 地址 | 处理函数 | 返回类型 | 契约 | 文档 | 调用方 | 今日调用 |
|---|---|---|---|---|---|---|
| GET `/jobs/:job_id/actions` | `actions::get_stage_actions` | `StageActionsView`（每阶段一个 `StageRetryActionView`） | 无 schema | `docs/api/06-阶段操作/01-stage-actions.md`（示例落后）、`api-spec/jobs-retry-and-control.md` | `frontend/packages/api/src/jobs-actions.ts:fetchJobStageActions`，`TranslationStageActions.tsx`、`failure-recovery-stages.ts` | 5 |
| GET `/jobs/:job_id/actions/resume` | `actions::get_resume_plan` | `JobResumePlanView` | 无 | 只在 `api-spec/jobs-retry-and-control.md` 一段里；`docs/api` 没有单独页 | `jobs-actions.ts:fetchResumePlan` -> `use-book-detail-resume.ts:69`、`job-resume-model.ts` | 15 |
| POST `/jobs/:job_id/actions/resume` | `actions::resume_job`（直接转调 `rerun_job_response`） | `JobSubmissionView` | 无 | api-spec 一段话，且写错（见问题 3） | `jobs-actions.ts:resumeJob` -> `use-book-detail-resume.ts:93` | 0 |
| POST `/jobs/:job_id/actions/rerun` | `actions::rerun_job` | `JobSubmissionView` | 无 | api-spec 一段话 | 前端只拿后端给的 `actions.rerun.url` 调：`jobs-actions.ts:rerunJob`，`task-center-api.ts:149`，`resume-port.ts:26` | 0 |
| POST `/jobs/:job_id/actions/cancel` | `actions::cancel_job` | `JobSubmissionView` | 无 | api-spec 有，`docs/api` 无 | `jobs-actions.ts:cancelJob`（含 `cancelOcrJob` 别名） | 0 |
| POST `/jobs/:job_id/actions/retry-stage` | `actions::retry_stage` | `RetryStageSubmissionView` | 无 | `docs/api/06-阶段操作/02-retry-stage.md`（响应示例缺字段）、api-spec | `jobs-actions.ts:retryJobStage`；AI 服务 `backend/ai/retainpdf_ai/agent_translation_runner.py:_retry`；Rust AI 能力 `retainpdf-ai/src/agent_capabilities.rs:494` | 0 |
| POST `/jobs/:job_id/actions/resolve-ocr-ambiguity` | `actions::resolve_ocr_ambiguity` | `OcrAmbiguityResolutionView{resolution,provider,operation,submission: RetryStageSubmissionView}` | 无 | 只有 api-spec | `jobs-actions.ts:resolveOcrAmbiguity` | 0 |
| GET `/jobs/:job_id/progress/events` | `progress::get_job_events` | `JobEventListView` | `contracts/job-events.v2.schema.json`（与 Rust 一致，见"没问题的"） | `docs/api/02-进度事件/01-事件总览.md`（缺列表外壳）、`docs/core/api/event-feed-v2.md` | `frontend/packages/api/src/jobs-events.ts` | 2 |
| GET `/jobs/:job_id/progress/stream`（SSE） | `progress::get_live_translation_events` | SSE，事件名 `translation_units_committed`，数据 `LiveTranslationCommitEventView` | `reader-data.v1.schema.json` 的 `LiveTranslationCommitEvent`（一致） | `api-spec/live-translation.md` | `frontend/packages/api/src/live-translation.ts:streamLiveTranslationEvents` | 0 |
| GET `/jobs/:job_id/progress/layout` | `progress::get_live_translation_layout` | `LiveTranslationLayoutView` | `reader-data.v1` `LiveTranslationLayout`（一致） | `api-spec/live-translation.md` | `live-translation.ts:fetchLiveTranslationLayout` | 0 |
| GET `/jobs/:job_id/progress/pages/:page` | `progress::get_live_translation_page` | `LiveTranslationPageView` | `reader-data.v1` `LiveTranslationPageSnapshot`（一致） | `api-spec/live-translation.md` | `live-translation.ts:fetchLiveTranslationPage` | 0 |

说明：`contracts/jobs-control.v1.schema.json` 是壳与 `retain-jobsd` 之间的进程间契约（`/internal/v1/jobs/{id}/launch|cancel`、`/processes/{pid}/terminate`），和公开的 `actions/*` 没有对应关系；公开的 7 个 actions 接口一个契约都没有（见问题 7）。

## 问题

### 1. "任务在跑，不能操作"同一种情况，状态码一会儿 400 一会儿 409
- 位置: `backend/packages/retainpdf-jobs/src/actions/stage_plan.rs:93-99,120-130`（`stage_plan_with`、`resume_plan` 的"在跑"分支只是把 `can_retry=false` / `reason`）；`actions/rerun.rs:100-107`（`build_rerun_request` 用 `AppError::bad_request(plan.reason)`）；`actions/stage_retry.rs:92-94`（`if !plan.can_retry { bad_request(plan.disabled_reason) }`）；`actions/stage_retry.rs:176-185`（refine 单独预判成 409）；`actions/rerun.rs:80-83`（`prepare_in_place_render_job` 的 409）；`actions/cancel.rs:14-19`、`support.rs:29-37`（取消非运行任务 409）。测试 `api_tests/jobs_retry/retry_refine.rs:213-219` 钉住 refine 的 409。
- 现状: 同样是 queued/running 的任务：
  - `POST actions/cancel` 对"已不在跑"返回 409；
  - `POST actions/retry-stage stage=refine` 返回 409（"job is already queued or running"）；
  - `POST actions/retry-stage` 的 `ocr/translation/render`（含原地 `create_new_job=false`）返回 400（"job is queued or running; cancel it before retrying a stage"）：`stage_plan` 检查在 `prepare_in_place_render_job` 之前，后者的 409 分支实际走不到；
  - `POST actions/rerun`、`actions/resume` 返回 400（"job is queued or running; cancel it before resuming"）；
  - 同一种"状态不允许"，还有：产物缺失（`disabled_reason_for_stage`，400）、Rust 模型任务不支持（refine 409，其它阶段 pre-check 409，但 render 在 plan 里被放行）、OCR 不确定（409）。"状态/前置条件不满足"一部分 400、一部分 409。
- 问题: 调用方没法靠状态码区分"请求写错了"和"现在状态不对"。已经有真实调用方在赌 409：`backend/ai/retainpdf_ai/agent_translation_runner.py:_retry` 对 409 才提示"任务正在运行，等它结束再试"，但它调的 `_rerender`（`stage=render, create_new_job=false`）在任务运行时走的是 400，用户看到的只是笼统的"重渲染没有提交"。
- 建议: 约定"任务当前状态/产物状态不允许 = 409，请求体本身不合法 = 400"。具体：`resume_plan`/`stage_plan` 的 `can_resume/can_retry=false` 转成错误时统一用 `AppError::conflict`（`build_rerun_request`、`retry_stage_submission` 里的 `bad_request(plan.reason/disabled_reason)`、`resolve_ocr_ambiguity` 里同款），请求体字段错误（`mode` 不支持、`refine` 放错 stage、`create_new_job=false` 对 ocr/translation 等）保持 400。同时删掉 `retry_stage_submission` 里 refine 单独的"先判在跑"重复分支。需要同步改 `retry_refine.rs`、`jobs_retry/rerun.rs:166`、`resume.rs` 里对 400 的断言和 `submissions.txt`（如有）。
- 对前端的影响: 前端 `submitJson` 对 400/409 的处理要看一下（`frontend/packages/api/src/http.ts`）；没有按 400 判断"在跑"的代码被搜到，影响应很小，但要在合并前 `git grep` 确认。
- 等级: 中

### 2. cancel 的返回把状态硬写成 `canceled`，与真实状态不符
- 位置: `backend/packages/retainpdf-jobs/src/actions/cancel_submission.rs:16-19`（`build_submission_view(..., JobStatusKind::Canceled, ...)`）；`actions/cancel.rs:36-39`（竞态下返回已是终态的任务）、`:76-96`（OCR 任务非 queued 时直接 `return Ok(job)`，没改状态）、`:110-112,128-129`（CAS 没赢时重新读库返回别人写的终态）。
- 现状: `cancel_job` 在三种情况下返回的任务状态不是 `Canceled`：任务在取消前一刻自己跑完（测试 `cancel_job_clears_registry_entry_when_job_finishes_during_the_race` 就断言返回 `Succeeded`）；`workflow=ocr` 且已离开 queued（进程被杀，终态由 runner 稍后写，api-spec 自己承认"轮询可能还读到 running"）；CAS 没赢。`cancel_submission` 无视这些，一律给调用方 `status: "canceled"`。
- 问题: 响应是错的：一个其实已经 `succeeded` 的任务被告知"已取消"；OCR 任务告知"已取消"但库里还是 running。
- 建议: 用 `job.status` 返回真实状态（OCR 非 queued 的情况可返回 `running`，或在 `cancel_job` 里对这一情况明确返回"取消已受理"的状态——需要先决定是否新增一个字段，如 `cancel_requested: true`，只增不改）。竞态返回 `succeeded` 时 `actions.cancel.enabled=false` 已经由 `build_job_actions` 根据状态算出，不会再自相矛盾。
- 对前端的影响: 前端 `cancelJob` 之后是否依赖返回的 `status === "canceled"` 需确认（`frontend/web/src/features` 里 `cancelJob` 的调用点）；多数是再去轮询。
- 等级: 中

### 3. api-spec 说 resume/rerun "不接受客户端 overrides"，代码和前端都接受
- 位置: `backend/api/docs/api-spec/jobs-retry-and-control.md`（"Resume Plan, Resume, and Generic Rerun"一节："both execute the server-selected plan rather than accepting client overrides"）；实际 `backend/api/src/routes/jobs/actions.rs:36-55,57-95`（`parse_rerun_overrides`，注释写明接受 `{"overrides":{"translation":{api_key|credential_ref|reviewer_api_key|reviewer_credential_ref}}}`）；前端 `jobs-actions.ts:resumeJob` 的注释和 `translation-secret.ts` 就在用。
- 现状: 文档与代码相反；且这个请求体格式（只认 translation 的 4 个 key，一律 `deny_unknown_fields`）在任何 `docs/` 页面里都没有写。`docs/api` 里压根没有 resume/rerun/cancel/resolve-ocr-ambiguity 的页面（只有 `00-接口目录.md` 一行）。
- 问题: 文档误导；自动化调用方不知道可以带 key。另外原地重渲染分支（`rerun.rs` 里 `resume_workflow == Render`）会直接忽略 `overrides`，调用方带了 key 也无提示。
- 建议: 改 api-spec 这一节，写清请求体、只认哪 4 个 key、原地重渲染会忽略；在 `docs/api/06-阶段操作/` 补 `03-resume-rerun.md`、`04-cancel.md`、`05-resolve-ocr-ambiguity.md`（把现有 api-spec 的内容搬成面向使用者的页面）。
- 对前端的影响: 无。
- 等级: 中

### 4. `actions/resume` 与 `actions/rerun` 是同一个实现的两个名字，`rerun` 的说明还写错了
- 位置: `backend/api/src/routes/jobs/actions.rs:136-143`（`resume_job_response` 只是调 `rerun_job_response`）；`router/jobs/actions.rs:12-16`（`/// 整个任务重跑`）；`actions/rerun.rs:20-97`。
- 现状: POST resume 与 POST rerun 走同一个函数 `rerun_submission`，请求/响应/错误完全一样。`rerun` 的登记说明写"整个任务重跑"，实际行为是"按 `resume_plan` 从断点续跑"（有译文就原地重渲染，只有 OCR 产物就起新任务从翻译开始），不是整个重跑。前端同时用两条：书详情用 `resume`（`use-book-detail-resume.ts:93`），任务详情/任务中心用后端给的 `actions.rerun.url`（`task-center-api.ts:149`、`resume-port.ts:26`），而 `GET actions/resume` 的计划恰是 rerun 的计划（`overview-renderer.ts:61` 里 `resumePlan?.can_resume || actions.rerun`）。
- 问题: 同一动作两个名字，地址上"GET resume 看计划，POST resume 执行"是对的，`rerun` 则是多余且名不副实的别名；文档（api-spec）自己也写了 "resume is currently an alias of rerun"。
- 建议: 保留 `actions/resume` 为正名，把 `actions/rerun` 登记为 `old "<.../actions/resume>"`（行为不变），把 `rerun` 的说明改成"同 resume（旧名）"。后端返回的 `JobActionsView.rerun` 链接先保持不动（前端依赖其 url）；等前端改读 `resume` 再删。
- 对前端的影响: 立即无；若要彻底去掉 `rerun`：`frontend/web/src/features/job-detail/domain/page/overview-renderer.ts:46,61`、`task-center/domain/task-center-api.ts:149`、`job-detail/domain/page/resume-port.ts:26`、`snapshot/snapshot.ts:254`、`packages/api/src/jobs-actions.ts:rerunJob` 要改成用 `resume` 链接，`JobActionsView.rerun` 字段保留一段时间。
- 等级: 低

### 5. 三套名字描述"复用什么、重跑什么"，resume 计划与阶段计划用词不一致
- 位置: `views/job_types.rs:245-253`（`JobResumePlanView`: `reuses_artifacts`, `reruns_stages`）、`:328-342`（`StageRetryActionView`: `will_reuse`, `will_rerun`）、`:420-430`（`RetryStageSubmissionView`: `reused_artifacts`, `rerun_stages`）；取值见 `actions/stage_plan.rs:114-117,142-146`（resume: `translations_dir`、`normalized_document_json`、`"rendering"`）vs `:204-218`（stage: `translation_result`、`ocr_result`、`"render"`）。
- 现状: 同一个概念 "reuse/rerun" 三种字段名；取值词表也不同：同一种恢复（用已有译文重渲染）在 `GET actions/resume` 里是 `from_stage:"render"`、`reruns_stages:["rendering"]`、`reuses_artifacts:["source_pdf","translations_dir","normalized_document_json"]`，在 `GET actions` 里是 `render` 阶段 `will_rerun:["render"]`、`will_reuse:["source_pdf","ocr_result","translation_result"]`。`from_stage` 取 `render|translate`（动词），而 stage 枚举是 `ocr|translation|render|refine`（名词）。前端 `job-resume-model.ts:70-71` 靠这些字符串做判断。
- 问题: 客户端要维护两份映射；`GET actions`（四个阶段）与 `GET actions/resume`（"续跑从哪开始"）本质上是同一份计划的两个视图（`resume_plan` 其实就是在 `stage_plan` 的基础上选一个阶段）。`StageRetryActionView.reason` 与 `disabled_reason` 还是同一个值（`stage_retry_view.rs:96-97`），前端到处写 `disabled_reason || reason`（`TranslationStageActions.tsx:150,205`）。
- 建议: 新增而不改：在 `JobResumePlanView` 旁边加 `from_stage_kind`（取 `RetryStageKind`）或直接加 `reuses`/`reruns` 用 stage 词表，与 `will_reuse/will_rerun` 对齐，旧字段保留；把 `reason` 标记为旧字段（`disabled_reason` 为正名）并在文档里写明。长期可考虑让 `GET actions` 里带一项 `resume: {...}`，让 `actions/resume` 的 GET 变成它的旧别名。
- 对前端的影响: 只增不改，前端无需立即动；之后统一时前端要改 `job-resume-model.ts`、`failure-recovery-stages.ts`、`TranslationStageActions.tsx`。
- 等级: 中

### 6. 三个 POST 动作返回三种"提交结果"形状
- 位置: `views/common.rs:7-20`（`JobSubmissionView`: job_id, status, workflow, ocr_reused, source_artifact_job_id, stages, links, actions, sibling_job_ids）、`views/job_types.rs:420-430`（`RetryStageSubmissionView`: job_id, source_job_id, status, workflow, rerun_from_stage, reused_artifacts, rerun_stages, ambiguous_request_policy, links, actions）、`:296-302`（`OcrAmbiguityResolutionView` 内嵌 `submission: RetryStageSubmissionView`）。
- 现状: cancel/rerun/resume 返回 `JobSubmissionView`；retry-stage 返回 `RetryStageSubmissionView`（没有 `stages`、`ocr_reused`，多了 `source_job_id` 等）；resolve-ocr-ambiguity 再包一层。这三个动作都是"对已有任务动手并返回一个（可能是新的）任务"，但 rerun/resume 的返回没有 `source_job_id`，客户端分不清"原地重跑"还是"起了新任务"，只能比较 `job_id`；retry-stage 返回的 `status` 恒为 `queued`（`stage_retry_view.rs:56`）。
- 问题: 前端被迫写 `result?.job_id || result?.id || jobId` 之类兜底（`jobs-actions.ts:retryJobStage`）；文档里的响应示例（`docs/api/06-阶段操作/02-retry-stage.md:56-67`）漏了 `ambiguous_request_policy`、`links`、`actions`。
- 建议: 不改现有字段，给 rerun/resume 的 `JobSubmissionView` 补 `source_job_id`（可选，原地时等于自身），文档按实际返回补全。是否要把两种视图合并留到下一轮（会动前端）。
- 对前端的影响: 只增字段，无。
- 等级: 低

### 7. 公开的 actions 接口没有 JSON Schema 契约；"契约"与实现的唯一对应是内部的 jobs-control
- 位置: `contracts/jobs-control.v1.schema.json`（内部 jobsd 接缝，4 个 `/internal/v1/...`）；`contracts/` 目录下没有 `StageActionsView`/`RetryStageRequest`/`JobResumePlanView`/`OcrAmbiguityResolutionRequest`。
- 现状: 前端自己手写 `OcrAmbiguityResolutionRequest`、`JobStageActionsView`、`LastRefineView`（`jobs-actions.ts:4-103`），没有从契约生成，多个字段以 `[key: string]: unknown` 兜底；`fetchResumePlan` 返回 `Promise<any>`。后端的测试（`api_tests/jobs_retry/*`）钉了行为，但没有机器可读的请求/响应定义。
- 问题: 请求体（`RetryStageRequest`、`RefineRetryRequest`、`OcrAmbiguityResolutionRequest`、rerun/resume overrides）是对外最复杂的一组，却只有 Markdown 描述；Rust 改字段名前端无编译期提示。
- 建议: 新增 `contracts/jobs-actions.v1.schema.json`，覆盖 7 个接口的 request/response（参考 `reader-data.v1` 的写法），并在 `contracts/tests` 里加一个与 `api_tests/jobs_retry` 对应的快照测试。
- 对前端的影响: 新增契约，不强制；可后续把 `jobs-actions.ts` 里手写类型换成生成类型。
- 等级: 中

### 8. `RetryStageRequest` 没有 `deny_unknown_fields`，拼错字段会悄悄变成默认行为
- 位置: `views/job_types.rs:368-385`（`RetryStageRequest`），对比同文件 `OcrAmbiguityResolutionRequest`（`:282`）、`RefineRetryRequest`（`:393`）、`routes/jobs/actions.rs:31` 的 `RerunRequest` 都有 `#[serde(deny_unknown_fields)]`。
- 现状: 顶层多写/拼错的字段被忽略。最危险的是 `create_new_job`：写成 `createNewJob: false` 就被忽略，`render` 会变成"新建任务"而不是"原地重渲染"；`ambiguous_request_policy` 拼错则回落到 `block`。API 文档也只写"Unknown sections are rejected"（指 overrides 里）。
- 问题: 与同一组其它三个请求体不一致；对 AI agent 调用方（`agent_translation_runner.py` 构造 body）尤其容易静默出错。
- 建议: 给 `RetryStageRequest` 加 `#[serde(deny_unknown_fields)]`。顺带评估去掉只有一个合法值的 `mode`（保留字段兼容，文档标注"固定 from_stage"）。
- 对前端的影响: 前端发的字段是 `stage`、`mode`?、`create_new_job`、`overrides`、`ambiguous_request_policy`、`refine`、以及 `retryJobStage` 的 `payload`——其中 `payload` 是整块 `...payload` 展开，里面可能夹带 `document_id/title/cover_url` 等书籍元数据（`jobs-actions.ts:retryJobStage` 读 `bookMeta.document_id` 等，说明调用方会往 payload 里放这些）。如果这些也进了请求体，加 `deny_unknown_fields` 会让前端 400。改之前必须 `git grep retryJobStage` 逐个调用点确认实际发出的 body。
- 等级: 中（需先查前端实际 body 再动）

### 9. `page` 路径参数在同一个任务下一处 0 起、一处 1 起
- 位置: `router/jobs/progress.rs:14`（`progress/pages/:page`，handler 把它当 `page_idx`，`routes/jobs/progress.rs:51-58`，`load_live_translation_page(... page_idx)`，内部 `format!("page-{:03}-", page_idx + 1)`）；对比 `routes.txt:131` `files/pages/:page`（`docs/api/07-产物下载/01-下载总览.md:61` 明确 `pages/3.jpg` 是第 3 页，页码从 1 开始）；refine `start_page/end_page` 也是 1-based（`02-retry-stage.md`）。
- 现状: `GET /jobs/x/progress/pages/0` 是第一页，`GET /jobs/x/files/pages/1.jpg` 才是第一页。路径参数都叫 `page`，响应里字段叫 `page_idx`（0 起）。
- 问题: 阅读页同时用这两个接口（预览图 + 实时译文），极易差一页。契约里 `page_idx` 的 0 起写得清楚（`reader-data.v1`），但路径名 `:page` 看不出来。
- 建议: 不改行为。把路径参数改名 `:page_idx` 并在 `classify_param` 里已有 `page_idx` 分类（`retainpdf-accounts/src/access.rs:46`，`"page_idx"` 已列）；文档（api-spec/live-translation.md 与目录说明）写明 "0-based"。若以后想统一成 1 起，必须新增 `progress/pages/by-number/...` 之类并存地址，不能直接改。
- 对前端的影响: 改名不影响调用（URL 形状不变），前端无需改。
- 等级: 中

### 10. `progress/stream` 永不结束，也没有"任务已结束"的信号
- 位置: `backend/api/src/routes/jobs/progress.rs:107-130`（`next_live_event` 的 `loop` 里 `events.is_empty()` 只 `sleep(750ms)` 再查；唯一退出是查询出错）。
- 现状: 连接建立后，即使任务早已 succeeded/failed/canceled、再没有新提交事件，服务端依旧每 750ms 通过 `run_job_query`（带 `load_supported_job` 读库）轮询一次，直到客户端断开。`api-spec/live-translation.md` 只说 15 秒 keep-alive，没有说明何时结束。
- 问题: 阅读页关掉前每个连接都会一直占着服务端循环 + 每秒 1～2 次数据库查询；多开几个标签页就是常驻负载。客户端也无从知道"不会再有事件了"。
- 建议: 任务进入终态后，先把积压的事件发完，再发一个事件（建议 `event: stream_end`，数据 `{"status": "succeeded"}`）并关闭；`Last-Event-ID` 重连仍可回放。属于新增事件，旧客户端 `normalizeCommitEvent` 对未知事件名返回 null 直接忽略（`live-translation.ts:normalizeCommitEvent`），所以兼容。
- 对前端的影响: 无强制；前端要识别 `stream_end` 才能停止重连，否则流被关后 `streamLiveTranslationEvents` 会"resolve"并被上层当作断线重连（需看上层重连逻辑，`features/reader/domain` 里调用点）。
- 等级: 低（负载问题，非错误）

### 11. layout/page 两个处理函数绕过有界查询执行器，在 async 里直接做同步磁盘和数据库读
- 位置: `routes/jobs/progress.rs:42-58`（`build_jobs_facade_from_state(&state).live_translation_layout(...)` 直接在 async 处理函数里调用）对比同文件 `get_job_events` / `read_commit_events` 用 `run_job_query`；`get_stage_actions`、`get_resume_plan` 也同样直接同步调用（`routes/jobs/actions.rs:21-27,106-111`）。
- 现状: `load_live_translation_layout` 读并解析整份 normalized document JSON（`progress/live_translation.rs:30-60`），`load_live_translation_page` 遍历 checkpoint 目录求 SHA-256（`find_committed_page_snapshot`）；`stage_plans` 的注释自己写着以前"stage-actions 要 0.4～0.6 秒"。这些都占着 tokio worker 线程。
- 问题: 与同一模块里的 events/stream 的做法不一致；阅读页打开时会并发打这几个接口。
- 建议: 都改走 `run_job_query`（`JobQueries` 上已有 `live_translation_events_after` 的先例；`layout/page` 需要的 `heal_live_translation_revisions` 写库，用 `run_job_query_once` 或 spawn_blocking）。缓存键用 `layout:{job}`、`page:{job}:{page}`。
- 对前端的影响: 无。
- 等级: 低

### 12. 实时译文错误体的顶层 `code` 是字符串，其它接口是数字；文档示例与实际文案不符
- 位置: `retainpdf-api-base/src/error.rs:113-117,355-370`（`LiveTranslationErrorBody{code:&'static str,...}`，顶层 `code` 为字符串）对比普通错误（`:437` 附近，`40900` 等数字）；`backend/api/docs/api-spec/live-translation.md` 末尾示例 message 为英文 "this page has no committed translation yet"，代码里是中文 "该页尚无已提交的翻译"（`live_translation.rs` 里所有 `live_error` 文案均为中文）。
- 现状: 成功信封是 `{code:0,message,data}`，失败信封通常是 `{code:数字,message,error:{...}}`；这三个 `progress/*` 接口的失败是 `{code:"LIVE_TRANSLATION_...",message,error:{code,...}}`。文档把这标成"兼容期"字段、没有下线日期。
- 问题: 同一信封里 `code` 的类型随接口变化；前端为此特意写了 `payload?.error?.code || payload?.code`（`live-translation.ts:responseError`）。文档示例与实际文案不一致。`LIVE_TRANSLATION_PAGE_NOT_COMMITTED` 用 404 表示"还没有"，在轮询场景下是常态而非错误。
- 建议: 保留 `error.code` 为正名；文档里把示例 message 改成实际中文，并明确"顶层 `code` 为字符串是此组接口的历史行为"。不建议现在改顶层类型（会动前端）。
- 对前端的影响: none（只改文档）。
- 等级: 低

### 13. `progress/events` 的 cursor/分页语义有几处只在内部文档里
- 位置: `progress/event_feed/mod.rs:23-111`；`views/common.rs:127-136,201-203`；`event_feed/cursor.rs:6-16`；用户文档 `docs/api/02-进度事件/01-事件总览.md`。
- 现状:
  - 默认 `start` 缺省是 `tail`，且 tail 页 `has_more` 恒为 false（只能向前读到快照上界，不能往更早翻）；要读全量必须 `start=head`。`docs/api/02-进度事件/01-事件总览.md` 只描述了单条事件，没有列表外壳 `{protocol_version, items[], next_cursor, has_more, limit}`、`start/cursor/limit` 参数、`event_id`，也写"seq: 全局递增"（实际是每任务内递增）。
  - 游标里仍编码 `ocr_only` 字段（恒 false，`cursor.rs:11-12` 注释说明是为了兼容 `/ocr/jobs` 旧游标），`contracts/job-events.v2.schema.json` 的 `description` 仍写 "Authorization, OCR scope and storage checks precede cursor validation"（`/ocr/jobs` 2026-10-18 已删）。
  - "游标超过水位线"返回 400（`event_feed/mod.rs:48-52`），"epoch 变了"返回 410 `EVENT_CURSOR_EXPIRED`；这两种在客户端看来都是"这个游标没用了，重新读"，却是不同状态码。
- 问题: 用户文档不够用，和 SSE 的 `after_seq`（整数、`Last-Event-ID`）完全是另一套游标，两处文档都需要对比说明；schema 描述过时。
- 建议: 补 `docs/api/02-进度事件/01-事件总览.md` 的列表外壳与 start/cursor 说明，并写一句"progress/stream 的 seq 与这里的 cursor 无关"；schema 描述去掉 OCR scope；"超过水位线"改 410（行为变更，需同步 `cursor_contract.rs` 测试和 `docs/core/api/event-feed-v2.md`）。
- 对前端的影响: 文档部分 none；若改 410，`jobs-events.ts` 对 400 的处理要看一下是否把 400 当"INVALID_QUERY"提示用户。
- 等级: 低

### 14. 死代码与重复
- 位置:
  - `backend/packages/retainpdf-jobs/src/progress/live_translation_queries.rs:34-42`：`JobsFacade::live_translation_events_after` 与 `progress/queries.rs:8-21` 的 `JobQueries::live_translation_events_after` 完全重复；路由和测试（`api_tests/translation_debug/revision_publish.rs:145`）用的都是 `JobQueries` 那个，`JobsFacade` 版本没有调用方（待在 `cargo build` 里确认；本次未跑）。
  - `actions/cancel_submission.rs:14` 的 `ensure_cancelable` 与 `actions/cancel.rs:14-19` 的第一段状态检查是同一条规则，同一请求检查两次。
  - 原地重渲染有两份实现且清理不一致：`actions/rerun.rs:27-60`（删除 `<job>-ocr` 子任务及其目录、凭据，清空 `rendered/` 目录，再 `prepare_in_place_render_job`）；`actions/stage_retry.rs:107-129`（`retry-stage` 的 `stage=render, create_new_job=false` 只 `prepare_in_place_render_job` + 清 refine override，**没有**清 `<job>-ocr` 子任务和 `rendered/`）。`rerun.rs:48-52` 注释写明清 `rendered/` 是为了避免完成判断 `Path::exists()` 把旧 PDF 当成功。
  - `routes/jobs/actions.rs` 里 `parse_rerun_overrides`（约 40 行校验逻辑）放在处理函数里，不在包里；同包 `stage_retry_overrides.rs` 另有一套 overrides 校验。
  - `get_resume_plan` 用 `load_supported_job`（会因旧版目录布局返回 409，且认合并虚拟任务 id），其余动作用 `load_job_or_404`；`GET actions` 同样用 `load_job_or_404`。
- 问题: 可能的行为偏差（尤其第三项：用 retry-stage 原地重渲染的任务可能残留孤儿 OCR 子任务，且旧 `rendered/` 里的 PDF 没清）；其余是维护成本。第三项我只核对了代码路径，没有跑测试验证"完成判断"是否真会被旧 PDF 骗过。
- 建议: 抽出一个 `reset_for_in_place_render(job)`，rerun 和 retry-stage 共用同一套清理；`parse_rerun_overrides` 移到 `actions/rerun.rs`（或 `stage_retry_overrides.rs`），路由里只剩 `ApiJson` 解析；删 `JobsFacade::live_translation_events_after`；cancel 只留一处检查。
- 对前端的影响: 无。
- 等级: 低（第三项重复清理不一致：中，建议先写一个测试确认）

### 15. 文档小问题（合并写）
- `docs/api/06-阶段操作/01-stage-actions.md` 示例的 `action.url` 是相对路径 `/api/v1/...` 且 body 只有 `{"stage":...}`，实际是绝对地址（`stage_retry_view.rs:absolute_url`，用 `request_base_url`）且 body 带 `ambiguous_request_policy`（refine 还带 `create_new_job:false` 与 `refine`）；缺 `reason`、`last_refine`。`api-spec/jobs-retry-and-control.md` 示例是对的。
- `docs/api/06-阶段操作/02-retry-stage.md` 请求示例的 `overrides` 没写 `ambiguous_request_policy`、没写 `overrides` 只允许哪些段（api-spec 写了），响应示例漏字段（见问题 6）。
- `backend/api/docs/submissions.txt` 里的快照小标题仍是旧地址 `POST /jobs/:id/retry-stage`、`POST /jobs/:id/rerun`（`submissions.txt:238,263,288,313,355,380`）；重新生成后会变，但这是测试里的字符串。
- `docs/api/06-阶段操作/02-retry-stage.md:最后一节` "与 resume 的区别"说 resume "更偏失败后的恢复"，但 resume 对成功任务同样可用（`resume_plan` 只看产物）。
- 等级: 低

## 没问题的

- 路径/命名：`actions/*` 都是 `/jobs/:job_id/actions/<动作>`，只有 `resume` 是 GET+POST 同地址（读计划/执行），语义清楚；`progress/*` 四个都归在 `progress/` 下，`:job_id` 命名与同类一致。查询参数全是 snake_case（`after_seq`、`limit`、`start`、`cursor`）。
- `progress/events` 契约与 Rust 类型一致：`JobEventListView{protocol_version,items,next_cursor,has_more,limit}` 与 `job-events.v2.schema.json` 的 required 一致；`JobEventFeedItem` 的 `#[serde(flatten)]` 对应 schema 里的 `event_id` 加记录字段；`user_stage` 被 `skip_serializing`、schema 里不出现，匹配；`limit` 夹在 1..500 与 schema 一致；`deny_unknown_fields` 使 `offset` 被拒；有 `contracts/tests/job-events-v2.test.mjs` 和 `api_tests/jobs_events/*` 钉住（cursor_contract、seq_contract 等）。
- 实时译文 3 个接口的契约都在 `reader-data.v1.schema.json`，字段（`page_idx` 0 起、`attempt`、`generation`、`page_hash`、`changed_item_ids`、`typography` 可选）与 `views/live_translation.rs` 一致；SSE 事件名 `translation_units_committed`、`id` 等于 `seq`、`after_seq` 与 `Last-Event-ID` 取较大值，代码（`progress.rs:68-75`）、测试（`api_tests/live_translation.rs:358-374`）、文档三者一致；`X-Accel-Buffering: no` 已带。
- SSE 的事件名与 `progress/events` 条目没有混淆：前者是 DB 里的 translation commit 序列（`list_translation_commit_events_after`），后者是事件流投影；两者的 `seq` 互不相关，已在 `docs/core/api/event-feed-v2.md` 说明。
- `progress/layout`、`progress/pages` 与 `reader/*`（markdown/regions/metadata）不重复：前者来自规范化文档 + 排版预热清单，后者来自成品/区域数据；`reader/regions` 是按页的 bbox，`progress/layout` 多了源文本、`typography`，用途不同（边翻边看）。
- 并发/竞态：`cancel_job` 的取消注册表清理与 CAS 失败后的重读（`cancel.rs:36-39,110-112`）有回归测试；OCR 歧义流程的 `resolution_revision` 与 dispatch generation 比对防止过期 UI（409）；`resolve_ocr_ambiguity` 的请求体 `deny_unknown_fields` 且有"收据字段按 provider/operation 校验"的单测。
- `RetryStageRequest.creates_new_job()` 的默认（refine 原地、其余新建）文档与代码一致；`refine` 放在非 refine 阶段会 400，已有测试。
- `GET actions` 的 4 个阶段共用一份 `StageArtifactAvailability`（`stage_plan.rs`），避免重复读盘；`action.body` 直接给出可提交的默认请求体，前端无需拼装。
- 调用方：除 `actions/cancel|rerun|resolve-ocr-ambiguity|retry-stage` 与 `progress/layout|pages|stream` 今天没被调用外（本地没有触发写操作和阅读实时译文），每个接口都找到了前端或 AI 调用点，没有发现"无人调用"的候选可删；唯一的别名冗余是问题 4 的 `rerun`/`resume`。
- `routes.txt` 里这 11 个接口与 `router/jobs/{actions,progress}.rs` 一致，访问级别都是 `signed`，没有遗漏的 `old` 行（旧地址 2026-10-18 已删）。
