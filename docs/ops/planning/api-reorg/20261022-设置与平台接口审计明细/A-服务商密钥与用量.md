# 服务商、密钥和用量

审计范围：`/providers` 下 5 个（OCR 服务商清单、MinerU / Paddle / DeepSeek 的密钥校验、DeepSeek 余额）、`/credentials` 下 5 个、`GET /usage`。
只读代码（main，`35043e3f`）得出，没跑测试、没调用运行中的服务、没读 `data/`。标「未实跑」的是读代码推断的结论。

## 接口一览

| 方法 地址 | 处理函数 | 返回类型 | 契约 | 文档 | 调用方（前端文件:行） | 备注 |
|---|---|---|---|---|---|---|
| GET `/api/v1/providers/ocr`（signed） | `routes/providers.rs::list_ocr_providers` → `retainpdf-data` `ocr_provider/catalog.rs:86 provider_public_definitions` | `Vec<OcrProviderPublicDefinition>`（`retainpdf-core/src/models/ocr.rs:115`） | 无；`api_tests/providers.rs` 钉了几个字段 | `backend/api/docs/api-spec/jobs-submission.md:276-320`、`credentials.md:55-59`、`docs/core/api/index.md:854` | **没有调用方**（前端用自己写死的 `frontend/web/src/platform/config/providers.ts`；`backend/ai`、`backend/cli` 也没有） | 把 `local` 的 `options.command` 列成客户端可填的选项，见第 2 条 |
| POST `/api/v1/providers/mineru/validate-token`（admin） | `providers::validate_mineru_token` → `retainpdf-providers` `provider_probe/ocr.rs:17` | `MineruTokenValidationView` | 无 | `docs/core/rust_api/08-Provider 校验接口.md`（只有这一个接口的说明） | `frontend/packages/api/src/providers.ts:15`（`web/src/app/home/composition/create-credentials.ts:118`）；CLI `backend/cli/src/cmd/setup.rs:73` | 失败一律分不清原因，见第 3 条 |
| POST `/api/v1/providers/paddle/validate-token`（admin） | `providers::validate_paddle_token` → `provider_probe/ocr.rs:66` | `MineruTokenValidationView`（同一个类型） | 无 | 只有 `docs/core/api/index.md:856` 一行 | `providers.ts:19`（`create-credentials.ts:123`，写死 `base_url`）；CLI `setup.rs:74` | 最长 120s × 3 次重试，见第 4 条 |
| POST `/api/v1/providers/deepseek/validate-token`（admin） | `providers::validate_deepseek_token` → `provider_probe/deepseek.rs:27` | `MineruTokenValidationView`（同一个类型） | 无 | 同上 `:857` | `providers.ts:23`（`web/src/features/credentials/domain/validation.ts:192`）；CLI `setup.rs:52` | 实际是「任意 OpenAI / Responses / Anthropic 兼容服务」的探针，不止 DeepSeek |
| POST `/api/v1/providers/deepseek/balance`（admin） | `providers::query_deepseek_balance` → `deepseek.rs:248` | `DeepSeekBalanceView` | 无 | 同上 `:858` | `providers.ts:27`（`validation.ts:246`）；CLI `setup.rs:56` | 只认官方地址，余额地址固定取配置 |
| GET `/api/v1/credentials`（admin） | `routes/credentials.rs:14 list_credentials_route` → `credentials/service.rs:174 list_credentials_with_values` | `CredentialListView { credentials, revision }` | 无 | `backend/api/docs/api-spec/credentials.md`、`docs/core/api/index.md:235-243` | `frontend/packages/api/src/credentials.ts:81 listCredentials` 只被 `web/src/features/credentials/domain/credential-vault.ts:59` 调，而 `createCredentialVault` **没被任何地方引用** | `?include_values=true` 回全部明文，见第 1 条 |
| POST `/api/v1/credentials`（admin） | `credentials::create_credential_route` → `service.rs:211` | `CredentialMutationView { credential, revision }` | 无 | 同上 | `credentials.ts:85`：只在 `credential-vault.ts:154,193`（没人用）；`create-credentials.ts:147`、`browser.ts:183` 把它传下去但没人调。`backend/ai/README.md:150` 让人手动建 agent 凭据 | |
| GET `/api/v1/credentials/:credential_ref`（admin） | `credentials::get_credential_route` → `service.rs:195` | `CredentialMutationView` | 无 | 同上 | **没有调用方**（前端连函数都没有） | |
| PUT `/api/v1/credentials/:credential_ref`（admin） | `credentials::update_credential_route` → `service.rs:239` | `CredentialMutationView` | 无 | 同上 | `credentials.ts:95`：只在 `credential-vault.ts:153,192`（没人用） | 能改 `kind`、`provider` |
| DELETE `/api/v1/credentials/:credential_ref`（admin） | `credentials::delete_credential_route` → `service.rs:397` | `CredentialDeleteView { credential_ref, deleted, revision }` | 无 | `credentials.md:26-42` | `credentials.ts:106`，只在 `web/src/platform/api/domains/credentials.ts:3` 再导出，**界面没有入口** | `?force=true` 跳过引用检查 |
| GET `/api/v1/usage`（signed） | `routes/token_usage.rs:31 all_usage_route`（登记在 `app/router/backups.rs:9`）→ `retainpdf-jobs` `reports/usage/api.rs:64 all_usage_view` | `UsageSummaryView`（`reports/usage/mod.rs:94`） | `token-usage.v1`，`reports/usage/tests.rs:161` 钉键集合；前端类型来自 `@retainpdf/contracts/token-usage` | 契约 `contracts/token-usage.v1.schema.json`；`docs/api/` 下没有说明 | `frontend/packages/api/src/usage.ts:23 fetchUsageSummary`（`web/src/features/usage/ui/UsageSettingsPanel.tsx:10`） | 已经在 `spawn_blocking` 里跑 |

`docs/api/00-接口目录.md:52-66,224` 和登记表一致（同源生成），`routes.txt:62-66` 也一致。

## 问题

### 1. `GET /credentials?include_values=true` 把凭据库里所有密钥原文一次性返回；没有调用方，和登记表、文档说的「不含明文」相反
- 位置: `backend/packages/retainpdf-providers/src/credentials/service.rs:129-133`（`ListCredentialQuery`，没有 `deny_unknown_fields`）、`:174-193`（`include_values` 为真时 `view.secret = Some(credential.secret.clone())`）；`backend/api/src/routes/credentials.rs:14-25`；测试 `backend/api/src/api_tests/credentials.rs:74-98`（反过来钉住「能拿回明文」）；登记表 `app/router/credentials.rs:8`「凭据列表（不含明文）」；`backend/api/docs/api-spec/credentials.md:12-14`（「List/get responses never return the secret」）；`docs/core/api/index.md:236`；`docs/api/00-约定/02-认证与错误.md:22`（「后端响应不会回显这些明文」）。单机的跨域：`backend/api/src/app/router.rs:78-81`（单机 `CorsLayer::permissive()`）；开发栈默认 key `ops/development/dev.sh:37`、`dev_stack.py:27`（`dev-local-key`）。
- 现状:
  - 前端 2026-09-16（`1b2258e7`「keep provider keys in the browser, stop serving them back」）已经不再带这个参数（`frontend/web/src/features/credentials/domain/browser.ts:297-307` 的注释说明了原因：每个新浏览器都会把服务端明文拉回来）；`frontend/web/dist`、`frontend/desktop`、`backend/ai`、`backend/cli` 里都没有 `include_values`。现在只有 api_tests 在用。
  - 凭据库里不只有用户手动存的：建任务时内联的 `mineru_token`、`paddle_token`、`translation.api_key` 会被导入成 `managed` 凭据（`retainpdf-jobs/src/lifecycle/creation/ocr_credentials.rs:57,86,115`），所以这一条返回的是这台机器用过的全部 OCR、翻译、agent 密钥。
  - 单机模式认证只靠 `X-API-Key`，CORS 全放开；开发栈默认 key 是公开的 `dev-local-key`。任何网页在浏览器里带上这个头，就能读到 `http://127.0.0.1:<port>/api/v1/credentials?include_values=true`（未实跑；新版 Chrome 的本地网络访问提示会拦一部分，Firefox/Safari 不一定）。即使换了 key，web 端的 key 写在前端运行配置里，能打开页面的人都拿得到。
- 问题: 一个没人用的参数把「密钥只进不出」的设计整个打穿；文档、登记表、约定三处都说不回明文，测试却钉住了能回。
- 建议: 后端去掉这个分支：`include_values` 先保留字段但忽略（或直接 400），`ListCredentialQuery` 加 `deny_unknown_fields`；`api_tests/credentials.rs:74-98` 改成断言带了参数也不含 `secret`。`CredentialMetadataView.secret` 字段删掉。对前端的影响（写进整理记录）：`frontend/packages/api/src/credentials.ts:16`（`secret?: string`）、`:81-82`（`includeValues` 选项）可以删，没有调用方传它，功能无影响。另记一笔：单机 `CorsLayer::permissive()` 加默认公开 key 是这一类问题的放大器（不只这个接口），属于认证那块，这里只记录。
- 等级: 高

### 2. （跨组）`/providers/ocr` 把 `local` 的 `command` 列成选项，建任务时请求里的 `ocr.options.command` 会被原样拿去执行
- 位置: `backend/packages/retainpdf-core/src/config/provider_config.rs:188-198`、`backend/config/ocr_providers.json` 的 `local.options.command`；`backend/packages/retainpdf-data/src/worker_command/stage_specs.rs:377,465-486`（`provider_options_for_stage`：先放配置里的默认值，再用请求里的 `ocr.options` **逐个覆盖**）；`backend/pipeline/retainpdf_pipeline/ocr/ocr_provider/provider_pipeline.py:95,118`（`local_ocr_command=ocr_options.get("command")`）；`local_command_driver.py:31-36,59,88-111`（`shlex.split` 后 `subprocess.run`，环境是 `os.environ.copy()`）；建任务校验 `retainpdf-jobs/src/lifecycle/validation.rs:362-380`（只校验 Paddle 的 `transport`，`command` 不管）；多用户平台模式会 `ocr.options.clear()`（`lifecycle/launcher.rs:129`），单机不会。
- 现状: 单机下 `POST /jobs`，`ocr.provider = "local"`，`ocr.options.command = "<任意程序和参数>"`，OCR 阶段就会以后端进程的身份、带着后端全部环境变量执行它（未实跑，按代码路径推断）。`GET /providers/ocr` 还把 `command` 和 `env: RETAIN_LOCAL_OCR_COMMAND` 一起公开，看起来就是让客户端填的。同样的「请求覆盖运维配置」还有 `raw_provider`；`ocr.paddle_api_url`、`translation.base_url` 不过 `validate_provider_base_url`（探针过，建任务不过），密钥会发到请求指定的地址。
- 问题: 「运维在配置文件 / 环境变量里定的命令」和「客户端能传的选项」没分开。配合第 1 条说的单机 CORS 和默认 key，就是从网页到本机执行命令的一条路。
- 建议: 交给建任务那组定。方向：provider 选项分「客户端可填」和「只能服务器定」两类（`command`、`raw_provider`，以及配置里带 `env` 的选项默认算后者）；`provider_options_for_stage` 不让请求覆盖后一类，建任务校验遇到就 400（`ocr.options.command is server-owned`）；`/providers/ocr` 给选项加 `client_settable: false`（只增）或者干脆不列出后一类。前端不发这些字段（`git grep` 前端没有 `options.command`），无影响。
- 等级: 高

### 3. MinerU 校验把错误链丢了：Token 无效、过期、超时全都报成「没能完成 MinerU 校验，请稍后重试」
- 位置: `backend/packages/retainpdf-providers/src/provider_probe/ocr.rs:60`（`classify_probe_error(err.to_string(), …)`）、`:102-172`；`backend/packages/retainpdf-data/src/ocr_provider/mineru/client.rs:108-116`（`post_json(...).await.context("MinerU apply upload url failed")`）、`:361`（`with_context("POST … failed")`）、`mineru/response_error.rs:78-96`（真正带 `MinerU HTTP 401`、`code=A0202` 的是这一层）；测试 `provider_probe/tests.rs:62-96,501-512`（都是手写字符串，没走真实的错误链）。
- 现状: `anyhow::Error` 的 `to_string()` 只给最外层的 context，所以 `classify_probe_error` 拿到的永远是 `"MinerU apply upload url failed"`：认不出错误码、不以 `mineru http 401` 开头、也不含 `timeout`/`connection`，落进最后的 `provider_error`、`retryable: true`，`operator_hint` 是「底层错误：MinerU apply upload url failed」（未实跑）。Paddle 那边是 `downcast_ref::<PaddleProviderError>()`（`ocr.rs:179`），没有这个问题。
- 问题: 这个接口存在的意义就是区分「Token 坏了」和「网络不通」，现在两者一样，还叫人稍后重试。`docs/core/rust_api/08-Provider 校验接口.md` 承诺的 `unauthorized`、`expired`、`network_error` 实际出不来。
- 建议: 先 `err.downcast_ref::<MineruResponseError>()`（`chain()` 里找）拿 `info()`，和 Paddle 同样处理；字符串匹配退到 `format!("{err:#}")`。加一个假 MinerU 服务器的测试（像 `tests.rs:250 fake_provider`），分别回 401、`code=A0211`、挂起超时。对前端的影响: 无，字段不变，`status` 变准。
- 等级: 中

### 4. 校验接口的超时各家不一样，MinerU / Paddle 远长于前端等的 30 秒
- 位置: `backend/packages/retainpdf-core/src/config/provider.rs:118`（MinerU `request_timeout_secs` 默认 120）、`:163,168`（Paddle 120s、重试 3 次）、`:230`（DeepSeek `probe_timeout_secs` 默认 20）；`retainpdf-data/src/ocr_provider/mineru/client.rs:95-98`；`paddle/client.rs:334-370`（`send_with_retry`，超时和连不上都重试，中间还有退避）；`provider_probe/deepseek.rs:52,277`；前端 `frontend/packages/api/src/providers.ts:4-7`（注释以为「后端探针自身超时 20s」，自己等 30s）。
- 现状: 外网请求都是 `reqwest` 异步客户端、`await` 的，不占阻塞线程，这一点没问题。但对端挂起时，MinerU 要 120s、Paddle 最长约 360s 多才返回；前端 30s 就放弃并显示「检测超时」，CLI（`setup.rs`）则一直等。
- 问题: 和前端的约定对不上；Paddle 探针在「检测」这种前台操作上做重试也不合适。
- 建议: 探针单独用一个短超时：新增 `RUST_API_PROVIDER_PROBE_TIMEOUT_SECS`（默认 20，DeepSeek 的旧变量继续认），MinerU / Paddle 探针建客户端时用它，Paddle 探针不重试。对前端的影响: 无；`providers.ts:4-6` 的注释可以顺手改准。
- 等级: 中

### 5. 探针跟随重定向：Anthropic 的 `x-api-key` 会被带到重定向后的地址，重定向目标也不再过私网拦截
- 位置: `provider_probe/deepseek.rs:47-54,276-281`（`reqwest::Client::builder()` 没设 `redirect`，默认跟 10 次）、`:348-350`（`anthropic_auth` 加 `x-api-key`）；`url_policy.rs:10-62`（只检查请求前的 `base_url`）；对照 `model_executor/mod.rs:264`（`.redirect(Policy::none())`，测试 `model_executor/tests.rs:506 redirects_never_forward_credentials`）。MinerU / Paddle 客户端（`mineru/client.rs:95`、`paddle/client.rs:363`）同样默认跟随。
- 现状: reqwest 跨主机重定向时会去掉 `Authorization`，但不认识自定义的 `x-api-key`，会照样带过去；一个公网地址 302 到 `127.0.0.1`、`169.254.169.254`，探针也会跟过去，`allow_private_urls=false` 形同虚设（未实跑）。
- 问题: 模型执行器已经堵上的口子，探针这条路还开着；探针的地址又是请求体里随便填的。
- 建议: 三个探针的客户端都设 `Policy::none()`，3xx 归 `provider_error`、`summary` 写「服务商返回了重定向，请检查接口地址」。对前端的影响: 无。
- 等级: 中

### 6. 凭据的 5 个处理函数在异步线程里直接做文件 IO、文件锁、fsync 和 SQLite
- 位置: `backend/api/src/routes/credentials.rs:14-75`（都是直接调同步函数）；`credentials/service.rs:440-461`（`mutate_vault`：进程内 `std::sync::RwLock` 写锁 + `flock(LOCK_EX)` + 读写文件 + `sync_all`）、`:397-438`（删除时 `count_jobs_referencing_credential`，`database/.../jobs` 对整张 `jobs` 表做 `json_extract` 计数）、`:647-699`；对照同组的 `routes/token_usage.rs:13-19` 已经 `spawn_blocking`。
- 现状: 建任务时会拿共享锁（`retainpdf-jobs/src/lifecycle/creation/submit.rs:147`），另一个后端进程也可能拿锁；凭据写请求会在 tokio 工作线程上阻塞等 `flock`。删除时的全表扫描随任务数增长。
- 问题: 违反规矩第 3 条最后一项（异步处理函数里不直接跑 SQLite、文件 IO）。
- 建议: 五个处理函数都包进 `spawn_blocking`（照 `token_usage.rs` 的 `blocking` 写法；`deps` 里的 `data_root`、`db` 换成可搬进线程的 `PathBuf` / `Arc<Db>`）。`GET /providers/ocr` 每次也同步 `fs::read_to_string` 配置文件（`provider_config.rs:130-137`），文件很小，顺手一起包或者缓存。对前端的影响: 无。
- 等级: 中

### 7. 凭据接口的 409 没有 `error.details.reason`
- 位置: `backend/packages/retainpdf-api-base/src/error.rs:319-329`（`credential_reference`）、`:434-449`（响应用 `StructuredError::empty`，`details` 恒为 `{}`）；`credentials/service.rs:276-280`（`CREDENTIAL_REVISION_CONFLICT`）、`:424-428`（`CREDENTIAL_IN_USE`）、`:474-478`（`CREDENTIAL_VAULT_REVISION_CONFLICT`）；`docs/api/00-约定/02-认证与错误.md:36,51-68`；`backend/api/docs/api-spec/credentials.md:75-84`。
- 现状: 三种 409 只能靠 `error.code`（大写字符串码）区分；找不到凭据是通用 `NOT_FOUND`（`service.rs:204,274,412`），而文档表里列的 `CREDENTIAL_REF_NOT_FOUND` 只在建任务解析引用时出现，`credentials.md:101-102` 有一句说明。
- 问题: 和「409 带 `error.details.reason`」的规矩不一致。
- 建议: `CredentialReference` 的响应 `details` 加 `reason`（`credential_in_use`、`credential_revision_conflict`、`credential_vault_revision_conflict`），只增；错误约定的「机器可读原因」表补一节凭据。对前端的影响: 无，`frontend/packages/api/src/credentials.ts:75` 读的是顶层 `code`，照旧。
- 等级: 中

### 8. 校验接口和凭据列表不拒绝不认识的字段
- 位置: `provider_probe/types.rs:3-10,26-31,33-44`（三个请求类型都没有 `deny_unknown_fields`）；`credentials/service.rs:129-133`（`ListCredentialQuery`）。凭据的建、改、删已经有（`service.rs:91,104,121`）。
- 现状: 拼错字段（例如 `token` 代替 `mineru_token`）在 MinerU / Paddle 上会报「`mineru_token is required`」还算看得出；`base_url` 拼错则静默走默认地址，校验的是另一个服务商。
- 问题: 违反规矩第 3 条（不认识的字段 400）。
- 建议: 四个类型都加 `deny_unknown_fields`。调用方核对过：前端发 `mineru_token`；`paddle_token` + `base_url`；`api_key` / `base_url` / `model` / `api_protocol`；余额发 `api_key` / `base_url`；CLI 的余额请求发了四个字段（`setup.rs:51,56`），都在 `DeepSeekTokenValidationRequest` 里，加上不会误伤。第 1 条做完以后 `include_values` 是否保留由那一条决定。对前端的影响: 无。
- 等级: 低

### 9. 校验结果的结构：三家共用一个叫 `MineruTokenValidationView` 的类型，`status` 取值各家不一，没有契约，`provider_message` 不截断
- 位置: `provider_probe/types.rs:12-24,54-67`；各家 `status`：MinerU `valid/unauthorized/expired/network_error/provider_error`（`ocr.rs`）、Paddle `valid/unauthorized/network_error/provider_error`、翻译接口 `valid/rate_limited/unauthorized/model_unavailable/insufficient_balance/network_error/provider_error/invalid_input`（`deepseek.rs`）、余额 `available/insufficient_balance/unauthorized/network_error/provider_error/unsupported_provider`；`deepseek.rs:490-508`（`summarize_deepseek_error_payload`：非 JSON 正文原样整段返回）、`:468`、`ocr.rs:167,258`（`operator_hint` 拼「底层错误：…」，含请求地址）；前端 `providers.ts:15-29` 全是 `Promise<any>`；文档只有 `docs/core/rust_api/08-Provider 校验接口.md`（只写了 MinerU 和 5 个 status）。
- 现状: `ok: true` 也可能是 `rate_limited`（限流算可用）；服务商回一整页 HTML 错误页时，`provider_message` 就是整页。没看到密钥原文会进返回：Key 只在请求头里，`base_url` 带用户名密码会 400（`url_policy.rs:24-28`），reqwest 的错误文本只带 URL。
- 问题: 前端和 CLI 都只能按 `ok` / `summary` 用，`status` 的全集没人写下来；名字误导后来的人。
- 建议: Rust 类型改名 `ProviderProbeView`（JSON 不变）；写一份 `provider-probe.v1` 契约，列出四个接口各自的 `status` 全集；`provider_message` 截到 500 字；`08-Provider 校验接口.md` 补 Paddle、翻译接口、余额三节。对前端的影响: 无；以后可以把 `Promise<any>` 换成契约类型。
- 等级: 低

### 10. 凭据库现在没有活的前端调用方，但服务端仍存着建任务导入的明文副本，界面看不到、删不掉
- 位置: 前端 `frontend/web/src/features/credentials/domain/credential-vault.ts:20`（`createCredentialVault` 没被引用）、`browser.ts:183-184`（收了 `createCredential`、`updateCredential` 不用）、`browser.ts:297`（注释「凭据只住在这个浏览器里，一个字节都不向服务端要」）；后端导入 `retainpdf-jobs/src/lifecycle/creation/ocr_credentials.rs:57,86,115`、`actions/stage_retry_refine.rs:105`；清理 `lifecycle/managed_credential_gc.rs`（删任务、删书、原地重跑后清掉没人引用的 `managed` 凭据，`deletion.rs:86`、`retainpdf-library/src/documents.rs:194`、`actions/rerun.rs:95`）；存放 `retainpdf-data/src/credentials.rs:15-17`（`<data_root>/secrets/credentials.json`，0600、明文 JSON）。
- 现状: 用户在设置页填的 Key 存在浏览器里，提交任务时以内联字段发出，后端落库前把它导入凭据库换成引用。任务还在，明文就一直在服务端；用户删了浏览器里的 Key，服务端那份不受影响。`GET /credentials/:ref`、`DELETE /credentials/:ref` 都没有调用方。
- 问题: 界面说法和实际存放位置不一致；用户想「从这台机器上清掉 Key」没有入口（只能删任务，或者手动删文件）。`PUT` 能改已有凭据的 `kind`、`provider`（`service.rs:282-287`），被任务引用的凭据改了种类，重试时会 `CREDENTIAL_KIND_MISMATCH`。新建凭据带 vault 级 `expected_revision` 时，后台导入会让它 409（只在旧客户端写法下）。
- 建议: 接口保留（AI 引用模式、外部集成要用）。文档 `credentials.md` 补一段「建任务导入的副本什么时候清」；`PUT` 不许改 `kind`（400）；前端死代码和「只在浏览器」的说法写进整理记录交前端核对，是否要在设置页加「清除服务端保存的密钥」由前端定（后端已有 `DELETE`）。
- 等级: 低

### 11. `GET /usage`：每次全量扫盘；月份按字节切，台账里一行怪时间戳能让整个用量页 500
- 位置: `backend/packages/retainpdf-jobs/src/reports/usage/mod.rs:314`（`record.ts[..7]`，`ts` 是台账原样的字符串，只要求长度 ≥ 7 字节）、`:141-161`（`parse_ledger_line` 不校验 `ts` 格式）、`:345-381`（`summarize_jobs`、`all_job_ids`：遍历 `output/` 下每个目录，读每份台账、诊断和精修报告，没有缓存）；`routes/token_usage.rs:13-19`（panic → `JoinError` → 500 `usage summary task failed`）。
- 现状: `ts` 前 7 个字节里有多字节字符（手改过、别的工具写的台账）就切在字符中间 panic，后果是 `/usage` 和这本书的用量都 500，直到那行被删（未实跑）。正常台账是 ISO 时间，不会触发。任务多了以后每打开一次设置页都要读一遍全部任务目录。
- 问题: 一行坏数据让整份汇总失败，和 `parse_ledger_line` 注释「坏行跳过」的意图相反。
- 建议: 改成 `record.ts.get(..7)`，取不到就不计入按月；全量扫描先记录，任务数上去再加按目录修改时间的缓存。接口说明在 `docs/api/` 下没有，补一页或在接口目录里链到契约。对前端的影响: 无。
- 等级: 低

### 12. `GET /providers/ocr`：没有调用方、没有契约，还把服务器内部的东西给出去
- 位置: `retainpdf-data/src/ocr_provider/catalog.rs:86-200`；`retainpdf-core/src/models/ocr.rs:80-123`（`credential.env`、`options.*.env`、`artifact_layout`）；`backend/api/src/api_tests/providers.rs:9-60`。
- 现状: 返回服务器的环境变量名（`RETAIN_PADDLE_API_TOKEN`、`RETAIN_LOCAL_OCR_COMMAND`）、产物内部文件名；运维在 `ocr_providers.json` 里给 `command` 写的默认值（通常带本机路径）会原样给所有 `signed` 用户。整个列表不分页，`docs/api/00-约定/03-分页.md` 没写它是「不分页的小列表」（`GET /credentials` 也一样）。
- 问题: `api-spec/jobs-submission.md:276` 说它是「前端和外部集成方的发现入口」，实际前端没用，自己写死了一份（`frontend/web/src/platform/config/providers.ts`），两份会走样。
- 建议: 只增不改：先写 `ocr-providers.v1` 契约把现在的字段钉住；第 2 条做的时候给「只能服务器定」的选项去掉 `default`（或标出来）。`03-分页.md` 的说明里补一句「`/providers/ocr`、`/credentials` 不分页」。前端要不要改用这个接口由前端定，写进整理记录。
- 等级: 低

### 13. 单机下本机模型服务没法校验；私网拦截只看字面地址
- 位置: `provider_probe/url_policy.rs:30-62`；`retainpdf-core/src/config/provider.rs:10,151,174,231`（`RUST_API_ALLOW_PRIVATE_PROVIDER_URLS` 默认 false，`ops/development/`、`frontend/desktop` 都没设）；对照建任务不检查 `translation.base_url`（第 2 条）。
- 现状: 单机用户把翻译接到 `http://127.0.0.1:11434/v1`（Ollama 等）时，探针 400「localhost/private targets are blocked」，前端显示网络错误；真正建任务却能用。反过来，域名解析到私网地址（内网 DNS、`*.localtest.me`）不会被拦。
- 问题: 单机下这条拦截主要是挡用户自己；多用户下又挡不全。
- 建议: 单机模式默认放开回环和私网（就是用户自己的机器），多用户保持拦截并在连接前解析域名再判一次（多用户已冻结，只记录）。对前端的影响: 无。
- 等级: 低

## 查过、没问题的

- 路由都在 `crate::endpoints!` 表里，`routes.txt`、`00-接口目录.md` 与表一致；`/usage` 登记在 `backups.rs`，文件头注释写明了为什么。
- `credential_ref` 路径参数：每个接口先过 `is_valid_credential_ref`（`cred_` 开头、只许小写字母数字 `_` `-`、≤64），不合格 400；编号只拿来查 JSON 里的键，不拼文件路径，凭据库是固定的一个文件，不存在穿越。
- 凭据库写入：同目录临时文件 `O_NOFOLLOW` + `create_new` + 0600，写完 `fsync` 再 `rename`，目录 0700 并 `fsync`；失败删临时文件；读之前拒绝符号链接、超大文件和非 0600。删除就是从 JSON 里去掉再整份重写，不留旧行；数据库备份不含 `secrets/`。
- 建、改、删的返回、错误信息都不带密钥原文（`api_tests/credentials.rs:62-72` 断言过）；请求体解析错误走固定文案（`routes/common/extractors.rs`），不回显请求体；`TraceLayer` 默认不记请求头，路由用量统计只记路由模板。
- 删凭据默认检查任务引用（`translation.credential_ref`、`reviewer_credential_ref`、`ocr.credential_ref`）和 AI 运行时引用，409 不泄露引用它的任务；和建任务之间有进程内读写锁 + 跨进程 `flock`，不会删在「校验引用」和「任务落库」之间。`managed` 凭据删任务、删书、原地重跑后会自动回收。
- 三个校验和余额的外网请求都是异步 `reqwest`，都有超时（长短问题见第 4 条）；`base_url` 先过 `validate_provider_base_url`（scheme、内嵌账号密码、字面私网地址）。余额只在地址含 `api.deepseek.com` 时才发，发往配置里固定的余额地址，不会把 Key 发到用户填的地址。翻译接口探针的 Key 含换行等非法字符时直接给 `invalid_input`，不会发出去。
- `GET /usage` 已经在 `spawn_blocking` 里；字段和 `token-usage.v1` 由 `reports/usage/tests.rs:161` 钉住，前端类型来自契约包；单机不按账号过滤、多用户按 `owner_filter`（只看了有没有）。
- 多用户相关（`/providers/ocr` 是 `signed`，其余 `admin`；平台模式清空 `ocr.options`）只记录，没细查。
