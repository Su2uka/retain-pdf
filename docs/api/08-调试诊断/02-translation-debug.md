# Translation Debug API

> 地址已按 9 类整理（新地址如 `/jobs/{job_id}/translations`）。对应的旧地址已于 2026-10-18 删除（返回 404）。新旧对照见 [任务接口分类](../01-任务/04-任务接口分类.md)。

这些接口用于排查翻译缺失、错误输出、模型异常和 replay。

## Diagnostics

```http
GET /api/v1/jobs/{job_id}/reports/translation
```

读取 `artifacts/translation_diagnostics.json`，返回翻译运行统计、provider 统计、重试统计等信息。

## Item 列表

```http
GET /api/v1/jobs/{job_id}/translations
```

常用查询参数：

- `page`
- `final_status`
- `error_type`
- `route`
- `q`
- `limit`
- `offset`

每次按 translation manifest 指向的页文件现算，所以手改、精修（`PATCH`）之后 `translated_preview`、`final_status` 和 `q` 搜索都是最新的；只有找不到 manifest 的老产物才退回读 `translation_debug_index.json`（它只在翻译结束时写一次，不随修订更新）。

## 单 item

```http
GET /api/v1/jobs/{job_id}/translations/{item_id}
```

从 translation manifest 指向的 page payload 里查原始 item。

## Replay

```http
POST /api/v1/jobs/{job_id}/translations/{item_id}/replay
```

Replay 是一次即时调试调用：

- 不创建新 job。
- 不进入队列。
- 不修改原任务状态。
- 后端同步调用 `backend/pipeline/devtools/replay_translation_item.py`。
- 可使用当前 job 的翻译 API key。
- 只重算、不写回；要写回用 `PATCH /jobs/{job_id}/translations/{item_id}`。
- 条目编号不合法返回 400；条目不存在返回 404；合并阅读的虚拟任务（`merged-…`）返回 409 `read_only_job`。
- 超过 300 秒返回 500，Python 进程随请求一起被杀。

## 脱敏规则

debug/replay 返回前会对 job request 里的敏感值脱敏。前端不要假设能拿到原始 API key 或 provider token。
