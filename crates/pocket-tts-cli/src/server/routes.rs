use crate::server::handlers;
use crate::server::state::AppState;
use axum::Router;
use axum::routing::{get, post};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

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
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    #[cfg(feature = "web-ui")]
    let router = router.route("/wasm/pkg/*path", get(handlers::serve_wasm_pkg));

    #[cfg(feature = "web-ui")]
    let router = router.fallback(handlers::serve_static);

    router
}
