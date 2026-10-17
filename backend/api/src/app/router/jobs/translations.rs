//! 译文：逐条看、人工改、重翻一条。

use crate::app::AppState;
use crate::routes::jobs::translations;

crate::endpoints! {
    state: AppState;
    /// 译文条目列表
    GET "/api/v1/jobs/:job_id/translations" => translations::list_translation_items, signed;
    GET "/api/v1/jobs/:job_id/translation/items" => translations::list_translation_items, signed old "/api/v1/jobs/:job_id/translations";
    /// 一条译文
    GET "/api/v1/jobs/:job_id/translations/:item_id" => translations::get_translation_item, signed;
    /// 人工修改一条译文
    PATCH "/api/v1/jobs/:job_id/translations/:item_id" => translations::revise_translation_item_route, signed;
    GET "/api/v1/jobs/:job_id/translation/items/:item_id" => translations::get_translation_item, signed old "/api/v1/jobs/:job_id/translations/:item_id";
    PATCH "/api/v1/jobs/:job_id/translation/items/:item_id" => translations::revise_translation_item_route, signed old "/api/v1/jobs/:job_id/translations/:item_id";
    /// 一条译文的修改历史
    GET "/api/v1/jobs/:job_id/translations/:item_id/revisions" => translations::get_translation_item_revisions, signed;
    GET "/api/v1/jobs/:job_id/translation/items/:item_id/revisions" => translations::get_translation_item_revisions, signed old "/api/v1/jobs/:job_id/translations/:item_id/revisions";
    /// 重翻一条
    POST "/api/v1/jobs/:job_id/translations/:item_id/replay" => translations::replay_translation_item_route, signed;
    POST "/api/v1/jobs/:job_id/translation/items/:item_id/replay" => translations::replay_translation_item_route, signed old "/api/v1/jobs/:job_id/translations/:item_id/replay";
}
