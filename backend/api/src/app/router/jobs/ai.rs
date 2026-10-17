//! AI：AI 助手写在这本书里的东西。还没生成是正常状态：阅读路径、概念图返回 404，画板返回空列表。

use crate::app::AppState;
use crate::routes::jobs::ai;

crate::endpoints! {
    state: AppState;
    /// AI 助手生成的阅读路径
    GET "/api/v1/jobs/:job_id/ai/reading-path" => ai::download_ai_reading_path, signed;
    GET "/api/v1/jobs/:job_id/reading-path" => ai::download_ai_reading_path, signed old "/api/v1/jobs/:job_id/ai/reading-path";
    /// AI 助手生成的概念图
    GET "/api/v1/jobs/:job_id/ai/canvas" => ai::download_ai_canvas, signed;
    GET "/api/v1/jobs/:job_id/canvas" => ai::download_ai_canvas, signed old "/api/v1/jobs/:job_id/ai/canvas";
    /// 画板里的文件列表
    GET "/api/v1/jobs/:job_id/ai/board" => ai::list_ai_board, signed;
    GET "/api/v1/jobs/:job_id/board" => ai::list_ai_board, signed old "/api/v1/jobs/:job_id/ai/board";
    /// 画板里的一个文件
    GET "/api/v1/jobs/:job_id/ai/board/:name" => ai::download_ai_board_file, signed;
    GET "/api/v1/jobs/:job_id/board/:name" => ai::download_ai_board_file, signed old "/api/v1/jobs/:job_id/ai/board/:name";
}
