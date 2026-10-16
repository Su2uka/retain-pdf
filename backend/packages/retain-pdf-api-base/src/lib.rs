//! API 各领域包共用的底座，在 `services` 下面一层：
//!
//! - `error`：错误类型 `AppError`，以及它对外的 HTTP 错误契约（状态码、`code`、结构化原因）；
//! - `job_loading`：按编号读出任务、确认格式受支持，以及合并阅读的虚拟任务。
//!
//! 以后按领域拆出去的包都往下依赖这里。它不依赖 `retain-pdf-api`，也就用不了 `services`、`routes`
//! 和 `AppState`，这一条由编译器保证。

pub mod error;
pub mod job_loading;
