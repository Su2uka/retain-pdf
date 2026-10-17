//! 文件：产出了哪些文件、取一个。按"给谁用"分三组：output 成品 / pages 译文页预览 /
//! internal 中间产物（只给管理员）。旧地址和新文件名不是一一对应，单列在后面，
//! 对照表见 docs/api/07-产物下载/01-下载总览.md。

use crate::app::AppState;
use crate::routes::jobs::files;

crate::endpoints! {
    state: AppState;
    /// 文件清单：按 output、pages、internal 三组列出这个任务的文件
    GET "/api/v1/jobs/:job_id/files" => files::list_job_files, signed;
    /// 取一个成品：translated.pdf、side-by-side.pdf、translated.docx、ocr.md、ocr.zip、ocr/images/…、all.zip
    GET "/api/v1/jobs/:job_id/files/output/*name" => files::job_output_file, signed;
    /// 译文某一页的预览图（pages/3.jpg）
    GET "/api/v1/jobs/:job_id/files/pages/:page" => files::job_page_file, signed;
    /// 取一个中间产物（规范化文档、翻译清单、请求日志……），多用户下只给管理员
    GET "/api/v1/jobs/:job_id/files/internal/*name" => files::job_internal_file, admin;
    /// 译文 PDF
    GET "/api/v1/jobs/:job_id/pdf" => files::download_pdf, signed old "/api/v1/jobs/:job_id/files/output/*name";
    /// 中英对照 PDF
    GET "/api/v1/jobs/:job_id/pdf/side-by-side" => files::download_side_by_side_pdf, signed old "/api/v1/jobs/:job_id/files/output/*name";
    /// 译文 Word
    GET "/api/v1/jobs/:job_id/docx" => files::download_layout_docx, signed old "/api/v1/jobs/:job_id/files/output/*name";
    /// 原文 Markdown（`?raw=true` 原样，否则 JSON）
    GET "/api/v1/jobs/:job_id/markdown" => files::download_markdown, signed old "/api/v1/jobs/:job_id/files/output/*name";
    /// Markdown 引用的图片
    GET "/api/v1/jobs/:job_id/markdown/images/*path" => files::download_markdown_image, signed old "/api/v1/jobs/:job_id/files/output/*name";
    /// 全部成品打包
    GET "/api/v1/jobs/:job_id/download" => files::download_bundle, signed old "/api/v1/jobs/:job_id/files/output/*name";
    /// 单页预览图（`?kind=source` 原文页在书下面，`translated` 是 pages/）
    GET "/api/v1/jobs/:job_id/preview/pages/:page" => files::download_page_preview, signed old "/api/v1/jobs/:job_id/files/pages/:page";
    /// 封面（现在在书下面：/documents/:id/files/cover.jpg）
    GET "/api/v1/jobs/:job_id/cover" => files::download_cover, signed old "/api/v1/documents/:document_id/files/:name";
    /// 缩略图（现在在书下面：/documents/:id/files/thumbnail.jpg）
    GET "/api/v1/jobs/:job_id/thumbnail" => files::download_thumbnail, signed old "/api/v1/documents/:document_id/files/:name";
    /// 规范化文档
    GET "/api/v1/jobs/:job_id/normalized-document" => files::download_normalized_document, signed old "/api/v1/jobs/:job_id/files/internal/*name";
    /// 规范化报告
    GET "/api/v1/jobs/:job_id/normalization-report" => files::download_normalization_report, signed old "/api/v1/jobs/:job_id/files/internal/*name";
    /// 产物列表
    GET "/api/v1/jobs/:job_id/artifacts" => files::get_job_artifacts, signed old "/api/v1/jobs/:job_id/files";
    /// 产物清单
    GET "/api/v1/jobs/:job_id/artifacts-manifest" => files::get_job_artifacts_manifest, signed old "/api/v1/jobs/:job_id/files";
    /// 按旧产物名取一个文件
    GET "/api/v1/jobs/:job_id/artifacts/:artifact_key" => files::download_artifact_by_key, signed old "/api/v1/jobs/:job_id/files/internal/*name";
}
