use askama::Template;
use axum::{response::Html, routing::get, Router};
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "bokuga=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    let app = Router::new()
        .nest_service("/public", ServeDir::new("public"))
        .route("/", get(index))
        .route("/greet", axum::routing::post(greet_handler))
        .fallback(static_files);

    let addr = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    tracing::debug!("listening on {:?}", addr);
    axum::serve(addr, app).await.unwrap();
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    title: String,
}

async fn index() -> Html<String> {
    let template = IndexTemplate {
        title: "Hello, World!".to_string(),
    };
    Html(template.render().unwrap())
}

async fn greet_handler() -> Html<String> {
    Html("<h2>Hello from Rust via HTMX</h2>".to_string())
}

async fn static_files(uri: axum::http::Uri) -> impl axum::response::IntoResponse {
    let path = uri.path().trim_start_matches('/');
    let file_path = format!("./public/{}", path);

    match tokio::fs::read_to_string(&file_path).await {
        Ok(content) => {
            let mime_type = mime_guess::from_path(&file_path).first_or_text_plain();
            (
                axum::http::StatusCode::OK,
                [(axum::http::header::CONTENT_TYPE, mime_type.to_string())],
                content,
            )
        }
        Err(_) => (
            axum::http::status::StatusCode::NOT_FOUND,
            [(axum::http::header::CONTENT_TYPE, "text/plain".to_string())],
            "File not found".to_string(),
        ),
    }
}
