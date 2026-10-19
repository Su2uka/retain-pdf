//! AI：AI 助手写在任务目录 `ai/` 下的画板（`ai/board/`），文件名受限地读。

mod ai_board;
mod ai_carryover;
#[cfg(all(test, unix))]
mod ai_carryover_tests;
mod downloads;
