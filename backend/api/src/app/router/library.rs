//! 书库：书架上的书（按任务列，旧的一套，书的新接口在 documents）。冻结：不再加字段；
//! 前端改调 `GET /jobs?job_ids=` 以后删列表，删除已经有新地址 `DELETE /jobs/:job_id`。

use crate::app::AppState;
use crate::routes::library;

crate::endpoints! {
    state: AppState;
    /// 书架列表（冻结，前端改调 GET /jobs?job_ids= 后删）
    GET "/api/v1/library/books" => library::list_books, signed;
    /// 从书架删除一本书
    DELETE "/api/v1/library/books/:job_id" => library::delete_book, signed old "/api/v1/jobs/:job_id";
}
