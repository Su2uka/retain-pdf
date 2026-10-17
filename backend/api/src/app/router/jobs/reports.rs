//! 报告：跑完结果怎么样。

use crate::app::AppState;
use crate::routes::jobs::reports;

crate::endpoints! {
    state: AppState;
    /// 质量体检卡
    GET "/api/v1/jobs/:job_id/reports/quality" => reports::quality_summary_route, signed;
    GET "/api/v1/jobs/:job_id/quality-summary" => reports::quality_summary_route, signed old "/api/v1/jobs/:job_id/reports/quality";
    /// 质量问题逐条清单
    GET "/api/v1/jobs/:job_id/reports/quality/items" => reports::quality_items_route, signed;
    GET "/api/v1/jobs/:job_id/quality-items" => reports::quality_items_route, signed old "/api/v1/jobs/:job_id/reports/quality/items";
    /// 翻译质检（AI 助手、命令行在用）
    GET "/api/v1/jobs/:job_id/reports/translation-qa" => reports::get_translation_qa, signed;
    GET "/api/v1/jobs/:job_id/translation/qa" => reports::get_translation_qa, signed old "/api/v1/jobs/:job_id/reports/translation-qa";
    /// 翻译诊断
    GET "/api/v1/jobs/:job_id/reports/translation" => reports::get_translation_diagnostics, signed;
    GET "/api/v1/jobs/:job_id/translation/diagnostics" => reports::get_translation_diagnostics, signed old "/api/v1/jobs/:job_id/reports/translation";
    /// 排版缩放情况
    GET "/api/v1/jobs/:job_id/reports/layout-fit" => reports::get_fit_report, signed;
    GET "/api/v1/jobs/:job_id/render/fit-report" => reports::get_fit_report, signed old "/api/v1/jobs/:job_id/reports/layout-fit";
    /// 精修报告
    GET "/api/v1/jobs/:job_id/reports/refine" => reports::get_refine_report, signed;
    GET "/api/v1/jobs/:job_id/translation/refine-report" => reports::get_refine_report, signed old "/api/v1/jobs/:job_id/reports/refine";
    /// 失败原因
    GET "/api/v1/jobs/:job_id/reports/failure" => reports::get_job_diagnostics, signed;
    GET "/api/v1/jobs/:job_id/diagnostics" => reports::get_job_diagnostics, signed old "/api/v1/jobs/:job_id/reports/failure";
    /// 模型用量
    GET "/api/v1/jobs/:job_id/reports/usage" => reports::job_usage_route, signed;
    GET "/api/v1/jobs/:job_id/usage" => reports::job_usage_route, signed old "/api/v1/jobs/:job_id/reports/usage";
}
