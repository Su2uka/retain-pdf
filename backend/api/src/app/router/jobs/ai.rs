//! AI：AI 助手写在这本书里的东西（画板）。还没放过东西是正常状态：列表返回空。

use crate::app::AppState;
use crate::routes::jobs::ai;

crate::endpoints! {
    state: AppState;
    /// 画板里的文件列表
    GET "/api/v1/jobs/:job_id/ai/board" => ai::list_ai_board, signed;
    /// 画板里的一个文件
    GET "/api/v1/jobs/:job_id/ai/board/:name" => ai::download_ai_board_file, signed;
}
