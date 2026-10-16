//! 业务逻辑按领域分成 9 个目录，一个领域以后拆成一个包（拆包规划第 3 节）。
//!
//! | 领域 | 管什么 |
//! |---|---|
//! | `accounts` | 账号、登录会话、页额度、邀请码、管理后台、平台设置 |
//! | `providers` | 模型和 OCR 服务商、凭据库、模型调用、连通性测试 |
//! | `ingest` | 上传 |
//! | `files` | 产物清单和打包、派生产物生成、下载文件名 |
//! | `jobs` | 任务：按 9 类分（见 `jobs/mod.rs`），含质量报告、模型用量 |
//! | `library` | 书、书架、合集、全文检索、合并阅读 |
//! | `glossaries` | 术语表 |
//! | `ai` | AI 问答、Agent、计算、文档操作 |
//! | `platform` | 同步、备份、字体、健康检查 |
//!
//! 每个领域的 `mod.rs` 是它唯一的门面：子模块私有，领域外只能用门面导出的东西。领域之间只能往下
//! 依赖，允许的方向登记在 `scripts/check_architecture.py` 的 `ALLOWED_DOMAIN_DEPENDENCIES`。

// 账号已经拆成独立的包，这里接上。
pub(crate) use retain_pdf_accounts as accounts;
pub(crate) mod ai;
pub(crate) mod files;
// 术语表已经拆成独立的包，这里接上，`crate::services::glossaries::…` 的写法不变。
pub(crate) use retain_pdf_glossaries as glossaries;
// 入口已经拆成独立的包，这里接上。
pub(crate) use retain_pdf_ingest as ingest;
pub(crate) mod jobs;
pub(crate) mod library;
// 平台已经拆成独立的包，这里接上。
pub(crate) use retain_pdf_platform as platform;
// 服务商已经拆成独立的包，这里接上。
pub(crate) use retain_pdf_providers as providers;
