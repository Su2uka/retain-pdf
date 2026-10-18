//! 书库：书架上的书（按任务列，旧的一套，书的新接口在 documents）。

use crate::app::AppState;
use crate::routes::library;

crate::endpoints! {
    state: AppState;
    /// 书架列表
    GET "/api/v1/library/books" => library::list_books, signed;
    /// 书架上的一本书
    GET "/api/v1/library/books/:job_id" => library::get_book, signed;
    /// 从书架删除一本书
    DELETE "/api/v1/library/books/:job_id" => library::delete_book, signed;
}
