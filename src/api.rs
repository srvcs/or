use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use utoipa::{OpenApi, ToSchema};

/// This service's identity. `srvcs-or` is a leaf: it depends on no other
/// service, computing the boolean OR of its two operands directly.
pub const SERVICE: &str = "srvcs-or";
pub const CONCERN: &str = "logic: boolean OR";
pub const DEPENDS_ON: &[&str] = &[];

#[derive(Serialize, ToSchema)]
pub struct Info {
    pub service: &'static str,
    pub concern: &'static str,
    pub depends_on: Vec<&'static str>,
}

/// `GET /` — service identity (srvcs service standard).
#[utoipa::path(get, path = "/", responses((status = 200, body = Info)))]
pub async fn index() -> Json<Info> {
    Json(Info {
        service: SERVICE,
        concern: CONCERN,
        depends_on: DEPENDS_ON.to_vec(),
    })
}

#[derive(Deserialize, ToSchema)]
pub struct EvalRequest {
    /// The first operand. Must be a JSON boolean.
    #[schema(value_type = Object)]
    pub a: Value,
    /// The second operand. Must be a JSON boolean.
    #[schema(value_type = Object)]
    pub b: Value,
}

#[derive(Serialize, ToSchema)]
pub struct OrResponse {
    #[schema(value_type = Object)]
    pub a: Value,
    #[schema(value_type = Object)]
    pub b: Value,
    pub result: bool,
}

/// The single concern: the boolean OR of two operands.
pub fn or(a: bool, b: bool) -> bool {
    a || b
}

fn ok(a: Value, b: Value, result: bool) -> Response {
    (
        StatusCode::OK,
        Json(json!({ "a": a, "b": b, "result": result })),
    )
        .into_response()
}

fn invalid(reason: &str) -> Response {
    (
        StatusCode::UNPROCESSABLE_ENTITY,
        Json(json!({ "error": reason })),
    )
        .into_response()
}

/// `POST /` — compute `a || b`.
///
/// Both operands must be JSON booleans; anything else is a client error. This
/// is a self-contained leaf and performs no I/O.
#[utoipa::path(
    post,
    path = "/",
    request_body = EvalRequest,
    responses(
        (status = 200, body = OrResponse),
        (status = 422, description = "an operand is not a boolean")
    )
)]
pub async fn evaluate(Json(req): Json<EvalRequest>) -> Response {
    let Some(a) = req.a.as_bool() else {
        return invalid("a is not a boolean");
    };
    let Some(b) = req.b.as_bool() else {
        return invalid("b is not a boolean");
    };
    ok(req.a, req.b, or(a, b))
}

#[derive(OpenApi)]
#[openapi(
    paths(index, evaluate),
    components(schemas(Info, EvalRequest, OrResponse))
)]
pub struct ApiDoc;

/// Serve OpenAPI document
pub async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn openapi_documents_routes() {
        let doc = ApiDoc::openapi();
        let root = doc.paths.paths.get("/").expect("path / present");
        assert!(root.get.is_some(), "GET / documented");
        assert!(root.post.is_some(), "POST / documented");
    }

    #[test]
    fn or_truth_table() {
        assert!(or(true, true));
        assert!(or(true, false));
        assert!(or(false, true));
        assert!(!or(false, false));
    }

    #[tokio::test]
    async fn evaluate_accepts_booleans() {
        let res = evaluate(Json(EvalRequest {
            a: json!(true),
            b: json!(false),
        }))
        .await;
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn evaluate_rejects_non_boolean_operands() {
        let res = evaluate(Json(EvalRequest {
            a: json!(true),
            b: json!("nope"),
        }))
        .await;
        assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);

        let res = evaluate(Json(EvalRequest {
            a: json!(1),
            b: json!(false),
        }))
        .await;
        assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn index_reports_identity() {
        let Json(info) = index().await;
        assert_eq!(info.service, "srvcs-or");
        assert!(info.depends_on.is_empty());
    }
}
