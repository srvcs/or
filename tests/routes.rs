use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use srvcs_or::{health, router, telemetry};
use tower::ServiceExt;

async fn status_of(uri: &str) -> StatusCode {
    let app = router(telemetry::metrics_handle_for_tests());
    app.oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap()
        .status()
}

/// POST a JSON body to `/` and return (status, parsed JSON response).
async fn post_eval(body: serde_json::Value) -> (StatusCode, serde_json::Value) {
    let app = router(telemetry::metrics_handle_for_tests());
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = res.status();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let json = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
    (status, json)
}

#[tokio::test]
async fn index_ok() {
    assert_eq!(status_of("/").await, StatusCode::OK);
}

#[tokio::test]
async fn healthz_ok() {
    assert_eq!(status_of("/healthz").await, StatusCode::OK);
}

#[tokio::test]
async fn readyz_reflects_state() {
    health::set_ready(true);
    assert_eq!(status_of("/readyz").await, StatusCode::OK);
}

#[tokio::test]
async fn metrics_ok() {
    assert_eq!(status_of("/metrics").await, StatusCode::OK);
}

#[tokio::test]
async fn openapi_ok() {
    assert_eq!(status_of("/openapi.json").await, StatusCode::OK);
}

#[tokio::test]
async fn post_true_or_false_is_true() {
    let (status, body) = post_eval(serde_json::json!({ "a": true, "b": false })).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["result"], true);
    assert_eq!(body["a"], true);
    assert_eq!(body["b"], false);
}

#[tokio::test]
async fn post_false_or_false_is_false() {
    let (status, body) = post_eval(serde_json::json!({ "a": false, "b": false })).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["result"], false);
}

#[tokio::test]
async fn post_non_boolean_operand_is_rejected() {
    // A non-boolean operand is a client error, not a 500.
    let (status, _) = post_eval(serde_json::json!({ "a": true, "b": "false" })).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    let (status, _) = post_eval(serde_json::json!({ "a": 1, "b": false })).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn post_malformed_body_is_rejected() {
    // Missing the `b` field is a client error, not a 500.
    let (status, _) = post_eval(serde_json::json!({ "a": true })).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn generates_request_id_when_absent() {
    let app = router(telemetry::metrics_handle_for_tests());
    let res = app
        .oneshot(
            Request::builder()
                .uri("/healthz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(
        res.headers().contains_key("x-request-id"),
        "response must carry a generated x-request-id"
    );
}
