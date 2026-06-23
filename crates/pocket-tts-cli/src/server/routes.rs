use crate::server::handlers;
use crate::server::state::AppState;
use axum::{
    Router,
    extract::State,
    http::{HeaderMap, StatusCode, header},
    middleware,
    response::Response,
    routing::{get, post},
};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

async fn auth_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    req: axum::http::Request<axum::body::Body>,
    next: middleware::Next,
) -> Response {
    let path = req.uri().path();

    if state.api_key.is_none() {
        return next.run(req).await;
    }

    if path == "/health" || path == "/" {
        return next.run(req).await;
    }

    let expected = state.api_key.as_deref().unwrap();
    let auth_header = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());

    let valid = auth_header.is_some_and(|h| {
        h.strip_prefix("Bearer ")
            .or_else(|| h.strip_prefix("bearer "))
            .is_some_and(|token| token == expected)
    });

    if !valid {
        let mut resp = Response::new(axum::body::Body::from(
            r#"{"error":"unauthorized"}"#,
        ));
        *resp.status_mut() = StatusCode::UNAUTHORIZED;
        return resp;
    }

    next.run(req).await
}

pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let router = Router::new()
        .route("/health", get(handlers::health_check))
        .route("/generate", post(handlers::generate))
        .route("/stream", post(handlers::generate_stream))
        .route("/tts", post(handlers::tts_form))
        .route("/v1/audio/speech", post(handlers::openai_speech))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    #[cfg(feature = "web-ui")]
    let router = router.route("/wasm/pkg/*path", get(handlers::serve_wasm_pkg));

    #[cfg(feature = "web-ui")]
    let router = router.fallback(handlers::serve_static);

    router
}
