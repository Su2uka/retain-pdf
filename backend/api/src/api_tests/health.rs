use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::util::ServiceExt;

use crate::api_tests::jobs_common::{read_json, test_state};
use crate::app::{build_app, build_simple_app};
use crate::config::JobsRuntimeMode;
use crate::routes::common::build_health_route_deps;
use crate::runtime::{ai_supervisor, jobsd_supervisor};
use crate::services::platform::build_readiness_view;

#[tokio::test]
async fn liveness_and_readiness_are_public_on_both_routers() {
    for app in [
        build_app(test_state("health-full-router")),
        build_simple_app(test_state("health-simple-router")),
    ] {
        for uri in ["/health", "/ready"] {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri(uri)
                        .body(Body::empty())
                        .expect("health request"),
                )
                .await
                .expect("health response");
            assert_eq!(response.status(), StatusCode::OK, "{uri}");
            let body = read_json(response).await;
            assert_eq!(body["code"], 0);
        }
    }
}

#[tokio::test]
async fn simple_router_serves_only_health_and_ready() {
    // 旧的一步提交接口 `/translate/bundle` 已经删掉：另一个端口上不再有它。
    let app = build_simple_app(test_state("health-simple-only"));
    for (method, uri) in [
        ("POST", "/api/v1/translate/bundle"),
        ("GET", "/api/v1/jobs"),
        ("POST", "/api/v1/uploads"),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(uri)
                    .header("X-API-Key", "test-key")
                    .body(Body::empty())
                    .expect("simple request"),
            )
            .await
            .expect("simple response");
        assert_eq!(response.status(), StatusCode::NOT_FOUND, "{method} {uri}");
    }
}

#[test]
fn readiness_requires_only_components_selected_by_config() {
    let state = test_state("readiness-config-boundary");
    let view = build_readiness_view(
        &build_health_route_deps(&state),
        ai_supervisor::AI_STATUS_UNHEALTHY,
        jobsd_supervisor::JOBSD_STATUS_UNHEALTHY,
    );
    assert_eq!(view.status, "ready");
    assert!(view.reasons.is_empty());
    assert!(!view.components.ai_service.required);
    assert_eq!(view.components.ai_service.status, "not_required");
    assert!(!view.components.jobsd.required);
    assert_eq!(view.components.jobsd.status, "not_required");

    let mut config = (*state.config).clone();
    config.ai_service.supervise = true;
    config.jobs_service.mode = JobsRuntimeMode::Remote;
    config.jobs_service.supervise = true;
    let required_state = crate::AppState {
        config: std::sync::Arc::new(config),
        ..state
    };
    let view = build_readiness_view(
        &build_health_route_deps(&required_state),
        ai_supervisor::AI_STATUS_STARTING,
        jobsd_supervisor::JOBSD_STATUS_UNHEALTHY,
    );
    assert_eq!(view.status, "not_ready");
    assert!(!view.is_ready());
    assert_eq!(
        view.reasons,
        ["ai_service_not_healthy", "jobsd_not_healthy"]
    );
    assert!(view.components.ai_service.required);
    assert_eq!(view.components.ai_service.status, "starting");
    assert!(view.components.jobsd.required);
    assert_eq!(view.components.jobsd.status, "unhealthy");

    let view = build_readiness_view(
        &build_health_route_deps(&required_state),
        ai_supervisor::AI_STATUS_HEALTHY,
        jobsd_supervisor::JOBSD_STATUS_HEALTHY,
    );
    assert_eq!(view.status, "ready");
    assert!(view.is_ready());
    assert!(view.reasons.is_empty());
}
