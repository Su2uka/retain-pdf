//! 请求日志不记查询串：助手终端握手可以把 key 放在 `?api_key=…` 里，开 `RUST_LOG=debug` 时不能写进日志。

use std::io::Write;
use std::sync::{Arc, Mutex};

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::util::ServiceExt;

use super::jobs_common::test_state;
use crate::app::build_app;

#[derive(Clone, Default)]
struct Captured(Arc<Mutex<Vec<u8>>>);

impl Write for Captured {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn debug_request_logs_keep_the_path_but_not_the_query_string() {
    let captured = Captured::default();
    let writer = captured.clone();
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_ansi(false)
        .with_writer(move || writer.clone())
        .finish();
    let _guard = tracing::subscriber::set_default(subscriber);

    // tracing 的级别和兴趣缓存是全进程共享的，别的测试线程同时在建、丢订阅者时，偶尔会让这边一次请求一条都抓不到；
    // 抓到为止，最多试 20 次。要守的是"抓到的日志里没有 key"。
    let app = build_app(test_state("request-log-query"));
    for _ in 0..20 {
        tracing::callsite::rebuild_interest_cache();
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/health?api_key=query-secret-0123456789")
                    .header("X-API-Key", "test-key")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        if !captured.0.lock().unwrap().is_empty() {
            break;
        }
    }

    let logs = String::from_utf8(captured.0.lock().unwrap().clone()).unwrap();
    assert!(logs.contains("/health"), "request span should still name the path: {logs}");
    assert!(!logs.contains("query-secret"), "query string leaked into logs: {logs}");
}
