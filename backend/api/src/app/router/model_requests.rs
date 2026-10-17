//! 内部：翻译工作进程调模型。两组鉴权不同，所以分两张表：
//! - 发能力令牌（`launcher`）：要应用的 API 密钥，挂在认证之内；
//! - 工作进程提交请求（`worker`）：凭只对一个任务有效的短期能力令牌，自己验，挂在认证之外。

pub(super) mod launcher {
    use crate::app::AppState;
    use crate::routes::model_requests;

    crate::endpoints! {
        state: AppState;
        /// 给一个任务的翻译工作进程发调模型的能力令牌
        POST "/api/v1/internal/model/jobs/:job_id/capability" => model_requests::issue, closed;
    }
}

pub(super) mod worker {
    use crate::app::AppState;
    use crate::routes::model_requests;

    crate::endpoints! {
        state: AppState;
        /// 工作进程提交一次模型调用（凭能力令牌）
        POST "/api/v1/internal/model/jobs/:job_id/requests" => model_requests::submit, closed;
        /// 一次模型调用的状态和结果
        GET "/api/v1/internal/model/jobs/:job_id/requests/:operation_id" => model_requests::status, closed;
        /// 取消一次模型调用
        POST "/api/v1/internal/model/jobs/:job_id/requests/:operation_id/cancel" => model_requests::cancel, closed;
    }
}
