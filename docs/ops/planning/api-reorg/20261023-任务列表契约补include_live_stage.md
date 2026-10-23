# 任务列表契约补 `include_live_stage`

2026-10-23。前端换 `listJobs` 生成函数时发现的漏项。

## 问题

`GET /api/v1/jobs` 的查询类型 `ListJobsQuery`（`job-status.v1`，`library-books.v1` 里有一份抄的）没有 `include_live_stage`。
后端一直认这个参数：`backend/packages/retainpdf-jobs/src/views/common.rs` 的 `ListJobsQuery.include_live_stage: Option<bool>`，
`presentation/listing.rs` 只在它为 `false` 时跳过实时阶段。前端最近任务、任务中心的列表请求都带 `include_live_stage=false`，
生成的 `listJobs` 参数类型没有这个键，多余属性检查会拦下。

## 决定

- 两份契约的 `ListJobsQuery` 都加 `include_live_stage`（`boolean`，可选）：`false` 只读库里的任务摘要，不导入事件历史；
  省略或 `true` 是完整的实时阶段。两份由 `contracts/tests/job-definitions-parity.test.mjs` 锁成一致，一起改。
- 接口行为不变，只补契约和生成的类型（`contracts/src`、`contracts/dist` 的 `job-status`、`library-books`）。
- rust_api 加 `api_tests::job_view_contract::list_jobs_query_contract_names_every_accepted_param`：契约 `ListJobsQuery` 的属性
  和 Rust 认的字段一一对应（两份契约都核），全部参数能解析；解构不写 `..`，Rust 以后加字段时编不过，提醒顺手补契约。
- `ListJobsQuery` 没有 `deny_unknown_fields`，不认识的参数照旧忽略，这次不收紧。

## 前端

重新生成 `frontend/packages/api/src/generated`（前端做），之后去掉换 `listJobs` 时为绕开多余属性检查加的宽类型变量。
