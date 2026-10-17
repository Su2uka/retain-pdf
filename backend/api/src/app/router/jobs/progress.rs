//! 进度：跑到哪了。

use crate::app::AppState;
use crate::routes::jobs::progress;

crate::endpoints! {
    state: AppState;
    /// 事件列表（分页、按游标续读）
    GET "/api/v1/jobs/:job_id/progress/events" => progress::get_job_events, signed;
    GET "/api/v1/jobs/:job_id/events" => progress::get_job_events, signed old "/api/v1/jobs/:job_id/progress/events";
    /// 实时推送（SSE）：边翻边看的新译文
    GET "/api/v1/jobs/:job_id/progress/stream" => progress::get_live_translation_events, signed;
    GET "/api/v1/jobs/:job_id/live-events" => progress::get_live_translation_events, signed old "/api/v1/jobs/:job_id/progress/stream";
    /// 边翻边看：页面文字块的版面
    GET "/api/v1/jobs/:job_id/progress/layout" => progress::get_live_translation_layout, signed;
    GET "/api/v1/jobs/:job_id/live-translation/layout" => progress::get_live_translation_layout, signed old "/api/v1/jobs/:job_id/progress/layout";
    /// 边翻边看：一页已经出来的译文
    GET "/api/v1/jobs/:job_id/progress/pages/:page" => progress::get_live_translation_page, signed;
    GET "/api/v1/jobs/:job_id/live-translation/pages/:page_idx" => progress::get_live_translation_page, signed old "/api/v1/jobs/:job_id/progress/pages/:page";
}
