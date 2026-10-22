//! 合集的 agent 工作区（`/collections/:collection_id/agent-workspace`）：编号校验、合集不存在、
//! POST 物化、删合集清目录。书库接口审计 A6（docs/ops/planning/api-reorg/20261019-书库接口审计.md）。
//! 旧地址 GET 2026-10-22 删了（前端改用 POST），现在是 405。

use std::fs;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::util::ServiceExt;

use super::jobs_common::{read_json, test_state};
use crate::app::build_app;

async fn request(app: &axum::Router, method: &str, uri: &str, body: &str) -> axum::response::Response {
    app.clone()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(uri)
                .header("X-API-Key", "test-key")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap()
}

/// 回归（A6）：路径参数会被百分号解码，`%2E%2E` 进到 `collection_id` 里就是 `..`，以前会对
/// `<data_root>/ai/books` 做 `remove_dir_all` 再重建。现在编号不是 `new_collection_id` 的形状就 400，
/// 磁盘一点不碰。
///
/// 这里只用解码后仍落在数据目录里的写法（`..` → `<data_root>/ai`），没改的代码上跑也不会碰到测试目录外面。
#[tokio::test]
async fn percent_encoded_traversal_ids_are_rejected_before_touching_the_disk() {
    let state = test_state("collection-workspace-traversal");
    let data_root = state.config.data_root.clone();
    // `collections/` 已经在（建过别的合集工作区），`collections/..` 才解析得通。
    fs::create_dir_all(data_root.join("collections")).unwrap();
    let decoy = data_root.join("ai/books");
    fs::create_dir_all(&decoy).unwrap();
    fs::write(decoy.join("keep.txt"), b"agent output").unwrap();
    let app = build_app(state);

    for id in [
        "%2E%2E",
        "%2e%2e",
        "%2E%2E%2Fcollections%2F%2E%2E",
        "..%2Fcollections%2F..",
        "col-1",
        "col-20261019000000-ABCDEF",
        "col-20261019000000-abc12",
        "col-2026101900000x-abcdef",
        "col-20261019000000-abcdef%2F%2E%2E",
    ] {
        let uri = format!("/api/v1/collections/{id}/agent-workspace");
        let response = request(&app, "POST", &uri, "{}").await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "POST {uri}");
        assert!(decoy.join("keep.txt").is_file(), "POST {uri} 删掉了 collections/ 外面的 ai/books");
        assert!(!data_root.join("ai/collection.v1.json").exists(), "POST {uri} 在 collections/ 外面写了清单");
    }
    assert!(decoy.join("keep.txt").is_file());
}

/// 合集不存在返回 404，也不再凭空建出 `collections/<id>/ai/` 目录。
#[tokio::test]
async fn an_unknown_collection_is_404_and_creates_no_directory() {
    let state = test_state("collection-workspace-unknown");
    let data_root = state.config.data_root.clone();
    let app = build_app(state);

    let id = "col-20261019000000-abcdef";
    let response = request(&app, "POST", &format!("/api/v1/collections/{id}/agent-workspace"), "{}").await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert!(!data_root.join("collections").join(id).exists(), "不存在的合集建出了目录");
}

/// POST 物化工作区；删合集以后 `collections/<id>/` 整个清掉。
#[tokio::test]
async fn post_materializes_the_workspace_and_deleting_the_collection_clears_it() {
    let state = test_state("collection-workspace-roundtrip");
    let data_root = state.config.data_root.clone();
    let app = build_app(state);

    let created = request(&app, "POST", "/api/v1/collections", r#"{"name":"合集"}"#).await;
    assert_eq!(created.status(), StatusCode::OK);
    let collection_id = read_json(created).await["data"]["collection_id"].as_str().unwrap().to_string();
    let workspace_uri = format!("/api/v1/collections/{collection_id}/agent-workspace");

    let posted = request(&app, "POST", &workspace_uri, "{}").await;
    assert_eq!(posted.status(), StatusCode::OK);
    let posted = read_json(posted).await;
    assert_eq!(posted["data"]["collection_id"], collection_id.as_str());
    assert_eq!(posted["data"]["schema"], "retainpdf_collection_workspace_v1");
    let collection_dir = data_root.join("collections").join(&collection_id);
    assert!(collection_dir.join("ai/collection.v1.json").is_file());

    // agent 在工作区里留下的东西，删合集时一起清掉（目录是这个合集专属的）。
    fs::write(collection_dir.join("ai/notes.md"), "跨书笔记").unwrap();
    let deleted = request(&app, "DELETE", &format!("/api/v1/collections/{collection_id}"), "").await;
    assert_eq!(deleted.status(), StatusCode::OK);
    assert!(!collection_dir.exists(), "删了合集，collections/<id>/ 还在");

    // 删掉以后再物化是 404，不会把目录建回来。
    let after = request(&app, "POST", &workspace_uri, "{}").await;
    assert_eq!(after.status(), StatusCode::NOT_FOUND);
    assert!(!collection_dir.exists());
}

/// 旧地址 GET 2026-10-22 删了：同一个路径还挂着 POST，所以是 405 不是 404；不物化、不建目录，
/// 编号不合法的也一样先回 405（方法不对，处理函数根本不跑）。
#[tokio::test]
async fn get_on_the_workspace_is_405_and_touches_nothing() {
    let state = test_state("collection-workspace-get-gone");
    let data_root = state.config.data_root.clone();
    let app = build_app(state);

    let created = request(&app, "POST", "/api/v1/collections", r#"{"name":"合集"}"#).await;
    assert_eq!(created.status(), StatusCode::OK);
    let collection_id = read_json(created).await["data"]["collection_id"].as_str().unwrap().to_string();
    for id in [collection_id.as_str(), "col-20261019000000-abcdef", "%2E%2E"] {
        let uri = format!("/api/v1/collections/{id}/agent-workspace");
        let response = request(&app, "GET", &uri, "").await;
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED, "GET {uri}");
    }
    assert!(
        !data_root.join("collections").join(&collection_id).join("ai").exists(),
        "GET 物化了工作区"
    );
}
