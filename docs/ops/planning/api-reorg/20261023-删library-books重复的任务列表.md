# 删 `library-books.v1` 重复登记的任务列表

2026-10-23。接 [契约第 1a 期补函数名](./20261022-契约第1a期补函数名.md) 留下的一条。

## 背景

`GET /api/v1/jobs` 在 `job-status.v1`、`library-books.v1` 各登记了一次，`client_name`（`listJobs`）只写在 `job-status.v1`。
`library-books` 那条留着，是因为生成器只导出端点用得到的定义：删了它，`@retainpdf/contracts/library-books` 就不再导出
`JobListView`、`JobProgressView` 等 14 个类型，而前端 `frontend/web/tests/contracts/library-books-contract.test.mjs` 当时从
生成的 `library-books.ts` 读 `JobProgressView`。前端在 `refactor/api-client-phase1-migrate`（e9fbab5a）里改成读 `job-status.ts`，可以删了。

## 决定

- 删 `library-books.v1`（两份镜像）里 `GET /api/v1/jobs` 那条端点。这份契约只剩 `DELETE /api/v1/jobs/:job_id`（`deleteJob`），
  生成的 `library-books.ts` 只导出 `LibraryDeleteQuery`、`LibraryDeleteResultView`，前端生成的请求函数不变。
- **任务列表定义先留在 JSON 里**：前端两个测试还直接读 `library-books.v1.schema.json` 的定义——
  `library-books-contract.test.mjs` 读 `JobProgressView`、`JobStatusKind`、`WorkflowKind`、`JobListItemView`、`JobStageSnapshotView`，
  `job-failure-brief-contract.test.mjs` 读 `JobFailureBriefView`。后端不改 `frontend/`，所以这次不删定义，
  `contracts/tests/job-definitions-parity.test.mjs` 照旧把它们和 `job-status.v1` 锁成一致。前端把两个测试改读
  `job-status.v1.schema.json` 以后，再删这 14 个定义和那条一致性测试，以及 rust_api `job_view_contract` 里对 `library-books` 的
  任务列表核对。
- **前端改好了，同一个分支接着删**：前端 `test/contracts-read-job-status-schema`（8fa12fba）把两个测试改读 `job-status.v1`，
  分支还没合，就在这个分支上把 14 个定义删了（两份镜像），`definitions` 只剩 `LibraryDeleteQuery`、`LibraryDeleteResultView`；
  `job-definitions-parity.test.mjs` 删掉，`package-boundary` 改成断言 `library-books` 的定义只有这两个、不再抄回来；
  rust_api `job_view_contract` 里三处对 `library-books` 的任务列表核对去掉，只核 `job-status`。生成的 TS 不变。
- 契约测试：`endpoint-client-names` 改成断言 `library-books` 里没有 `GET /api/v1/jobs`；`package-boundary` 断言任务列表类型
  只从 `job-status` 导出、`library-books` 不再导出。
- 文档里指向 `library-books` 的 `JobFailureBriefView` 改指 `job-status`（`docs/api/01-任务/03-任务列表.md`、`docs/core/api/index.md`）。

接口行为不变。
