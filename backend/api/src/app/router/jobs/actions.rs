//! 操作：能对它做什么、去做。

use crate::app::AppState;
use crate::routes::jobs::actions;

crate::endpoints! {
    state: AppState;
    /// 现在能做哪些操作（每个阶段能不能重试、重试的请求怎么写）
    GET "/api/v1/jobs/:job_id/actions" => actions::get_stage_actions, signed;
    GET "/api/v1/jobs/:job_id/stage-actions" => actions::get_stage_actions, signed old "/api/v1/jobs/:job_id/actions";
    /// 续跑会从哪里开始
    GET "/api/v1/jobs/:job_id/actions/resume" => actions::get_resume_plan, signed;
    /// 从断点续跑
    POST "/api/v1/jobs/:job_id/actions/resume" => actions::resume_job, signed;
    GET "/api/v1/jobs/:job_id/resume-plan" => actions::get_resume_plan, signed old "/api/v1/jobs/:job_id/actions/resume";
    POST "/api/v1/jobs/:job_id/resume" => actions::resume_job, signed old "/api/v1/jobs/:job_id/actions/resume";
    /// 取消任务
    POST "/api/v1/jobs/:job_id/actions/cancel" => actions::cancel_job, signed;
    POST "/api/v1/jobs/:job_id/cancel" => actions::cancel_job, signed old "/api/v1/jobs/:job_id/actions/cancel";
    /// 整个任务重跑
    POST "/api/v1/jobs/:job_id/actions/rerun" => actions::rerun_job, signed;
    POST "/api/v1/jobs/:job_id/rerun" => actions::rerun_job, signed old "/api/v1/jobs/:job_id/actions/rerun";
    /// 重试某一个阶段（OCR、翻译、渲染、精修）
    POST "/api/v1/jobs/:job_id/actions/retry-stage" => actions::retry_stage, signed;
    POST "/api/v1/jobs/:job_id/retry-stage" => actions::retry_stage, signed old "/api/v1/jobs/:job_id/actions/retry-stage";
    /// OCR 拿不准时由用户选
    POST "/api/v1/jobs/:job_id/actions/resolve-ocr-ambiguity" => actions::resolve_ocr_ambiguity, signed;
    POST "/api/v1/jobs/:job_id/ocr/resolve-ambiguity" => actions::resolve_ocr_ambiguity, signed old "/api/v1/jobs/:job_id/actions/resolve-ocr-ambiguity";
}
