# retry-stage

> 地址已按 9 类整理（新地址如 `/jobs/{job_id}/actions/retry-stage`）。对应的旧地址已于 2026-10-18 删除（返回 404）。新旧对照见 [任务接口分类](../01-任务/04-任务接口分类.md)。

## 接口

```http
POST /api/v1/jobs/{job_id}/actions/retry-stage
```

用于用户主动从某个阶段重新执行后续流程。

## 请求示例

```json
{
  "stage": "translation",
  "mode": "from_stage",
  "create_new_job": true,
  "overrides": {
    "translation": {
      "model": "deepseek-flash",
      "workers": 100
    },
    "render": {
      "compile_workers": 8
    }
  }
}
```

## 阶段语义

- `ocr`: 复用 source PDF，重跑 OCR -> translation -> render。
- `translation`: 复用 source PDF + OCR 结果，重跑 translation -> render。
- `render`: 复用 source PDF + OCR 结果 + 翻译结果，只重跑 render（不精修）。
- `refine`: 复用全部产物，原地（只能 `create_new_job=false`，省略时默认就是 false）先精修已提交的译文、再渲染一次。
  可带 `"refine": {"mode": "review_only" | "review_and_fix", "start_page": 3, "end_page": 5, "max_items": 0, "max_tokens": 0}`，页码 1-based 闭区间，省略 = 全书、`review_and_fix`。
  `max_items` / `max_tokens` 是这次精修的上限（`0` = 不限），省略 = 不限（审全书）；**不沿用**任务里存的
  `refine_max_items` / `refine_max_tokens`。上次精修中途停了的，报告里 `review.next_page` 是没审到的第一页，
  用 `start_page` 从那一页接着精修。
  这次的精修参数是一次性的，不会写进任务的 `translation.refine`；报告见 `GET /api/v1/jobs/{job_id}/reports/refine`。

## 请求规则

- 请求体里不认识的字段一律返回 `400`（例如把 `create_new_job` 拼成 `createNewJob`）。可用字段：`stage`、`mode`、`create_new_job`、`overrides`、`ambiguous_request_policy`、`refine`。
- 请求本身写错了返回 `400`：`mode` 不是 `from_stage`、`refine` 对象用在非 `refine` 阶段、`create_new_job=false` 用在 `ocr` / `translation`、`create_new_job=true` 用在 `refine`、`overrides` 不合法。这些和任务当前状态无关。
- 请求没问题但任务或产物现在不允许返回 `409`，`error.details.reason` 说明原因：`job_active`（任务在跑）、
  `artifacts_missing`（要用的产物或来源不在）、`translations_not_owned`（精修的任务渲染的是别的任务的译文）、
  `ambiguous_request`（请求结果不确定，要带 `ambiguous_request_policy=accept_duplicate_risk` 或先 `resolve-ocr-ambiguity`）、
  `request_journal_corrupt`（翻译请求日志损坏）、`model_executor_unsupported`（绑定了 Rust 模型执行器）。
  完整表格见 [错误约定](../00-约定/02-认证与错误.md#任务操作的-409-原因)。以前这些情况里「任务在跑」「产物不在」返回 400，现在统一 409。

## 响应示例

```json
{
  "job_id": "new-job-id",
  "source_job_id": "old-job-id",
  "status": "queued",
  "workflow": "book",
  "rerun_from_stage": "translation",
  "reused_artifacts": ["source_pdf", "ocr_result"],
  "rerun_stages": ["translation", "render"]
}
```

前端拿到新 `job_id` 后直接进入正常轮询。

## 与 resume 的区别

- `resume` 更偏失败后的恢复。
- `retry-stage` 是用户主动从指定阶段重跑，成功任务也可以用。
