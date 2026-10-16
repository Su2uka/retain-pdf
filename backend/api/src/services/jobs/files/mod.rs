//! 文件：任务产出的文件——列出来、取一个（唯一的下载入口 `JobDownloads::open`）、是否就绪。

pub(in crate::services::jobs) mod downloads;
mod queries;
mod readiness;

pub(crate) use downloads::{
    Disposition, FileDownload, FileGroup, JobDownloads, JobFile, JobFilesView, JobScope,
};
pub(crate) use readiness::job_readiness;
