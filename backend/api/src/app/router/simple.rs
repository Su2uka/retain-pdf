//! 简化接口：另一个端口（simple_port）上只有这一个，给命令行、脚本一步提交带文件的翻译。

use crate::app::AppState;
use crate::routes::jobs;

crate::endpoints! {
    state: AppState;
    /// 带文件一步建翻译任务（不限请求体大小）
    POST "/api/v1/translate/bundle" => jobs::lifecycle::translate_bundle, closed [unlimited_body];
}
