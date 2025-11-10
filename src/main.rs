mod utils;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::task;
use uuid::Uuid;
use tower_http::services::ServeDir;

#[derive(Clone)]
struct AppState {
    tasks: Arc<Mutex<HashMap<String, TaskStatus>>>,
}

#[derive(Clone, Serialize)]
struct TaskStatus {
    ready: bool,
    results: Option<Vec<utils::CheckResult>>,
}

#[derive(Deserialize)]
struct CrawlerRequest {
    url: String,
}

#[derive(Serialize)]
struct StartResponse {
    success: bool,
    task_id: String,
}

#[tokio::main]
async fn main() {
    let state = AppState {
        tasks: Arc::new(Mutex::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/", get(serve_html))
        .route("/run-crawler", post(run_crawler))
        .route("/crawler-result/{task_id}", get(get_crawler_result))
        .nest_service(
            "/assets",
            ServeDir::new("assets")
        )
        .with_state(state);

    let mut listener = tokio::net::TcpListener::bind("0.0.0.0:10000")
        .await
        .unwrap();

    if cfg!(debug_assertions) {
        listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
            .await
            .unwrap();
    }
    
    axum::serve(listener, app).await.unwrap();
}

async fn serve_html() -> impl IntoResponse {
    let html_content = include_str!("../index.html");
    Html(html_content)
}

async fn run_crawler(
    State(state): State<AppState>,
    Json(payload): Json<CrawlerRequest>,
) -> impl IntoResponse {
    let task_id = Uuid::new_v4().to_string();

    {
        let mut tasks = state.tasks.lock().unwrap();
        tasks.insert(
            task_id.clone(),
            TaskStatus {
                ready: false,
                results: None,
            },
        );
    }

    let task_id_clone = task_id.clone();
    let state_clone = state.clone();
    let url = payload.url.clone();
    
    task::spawn(async move {
        let results = utils::run_crawler(&url).await;
        
        let mut tasks = state_clone.tasks.lock().unwrap();
        tasks.insert(
            task_id_clone,
            TaskStatus {
                ready: true,
                results: Some(results),
            },
        );
    });

    (
        StatusCode::OK,
        Json(StartResponse {
            success: true,
            task_id,
        }),
    )
}

async fn get_crawler_result(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    let tasks = state.tasks.lock().unwrap();
    
    match tasks.get(&task_id) {
        Some(status) => (StatusCode::OK, Json(status.clone())),
        None => (
            StatusCode::NOT_FOUND,
            Json(TaskStatus {
                ready: false,
                results: None,
            }),
        ),
    }
}