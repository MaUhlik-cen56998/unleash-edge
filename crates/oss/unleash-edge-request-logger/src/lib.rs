use axum::body::{Body, to_bytes};
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use http::{HeaderValue, header};
use tracing::trace;

const MAX_LOGGED_PAYLOAD_LEN: usize = 1_048_576;

fn payload_to_log_string(body: &[u8]) -> String {
    if body.is_empty() {
        return "<empty>".to_string();
    }

    let payload = String::from_utf8_lossy(body);
    let mut truncated_payload = payload.chars().take(MAX_LOGGED_PAYLOAD_LEN).collect::<String>();
    if payload.chars().count() > MAX_LOGGED_PAYLOAD_LEN {
        truncated_payload.push_str("...<truncated>");
    }

    truncated_payload
}

pub async fn log_request_middleware(req: Request, next: Next) -> Response {
    let (parts, body) = req.into_parts();
    let uri = parts.uri.clone();
    let mut headers = parts.headers.clone();
    let method = parts.method.clone();

    let body_bytes = to_bytes(body, usize::MAX).await.unwrap_or_default();
    let payload = payload_to_log_string(&body_bytes);

    headers.insert(
        header::AUTHORIZATION,
        headers
            .get(header::AUTHORIZATION)
            .map(|f| {
                if f.is_empty() {
                    HeaderValue::from_str("AuthorizationEmpty").unwrap()
                } else {
                    HeaderValue::from_str("***redacted***").unwrap()
                }
            })
            .unwrap_or_else(|| HeaderValue::from_str("No authorization header").unwrap()),
    );

    let req = Request::from_parts(parts, Body::from(body_bytes));
    let res = next.run(req).await;
    trace!(
        "Request: uri=[{}], method=[{}], headers=[{:?}], payload=[{}], status=[{:?}]",
        uri,
        method,
        headers,
        payload,
        res.status()
    );
    res
}
