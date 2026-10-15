mod files;
mod markdown;

pub use files::{
    ai_board_file_response, ai_document_response, job_file, ocr_job_file,
};
pub use markdown::{markdown_document_response, markdown_response};
