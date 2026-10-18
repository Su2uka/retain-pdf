//! 上传，以及只做 OCR 的旧别名 `/ocr/jobs/*`（多用户下关闭，内容对应 `/jobs/*`）。

use crate::app::AppState;
use crate::routes::uploads;

crate::endpoints! {
    state: AppState;
    /// 上传一个 PDF（不限请求体大小），建任务时用返回的 upload_id
    POST "/api/v1/uploads" => uploads::upload_pdf, signed [unlimited_body];
}
